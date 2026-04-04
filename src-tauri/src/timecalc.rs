use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use rusqlite::{params, Connection, OptionalExtension};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResolvedSchedule {
    pub jornada_id: Option<i64>,
    pub jornada_nome: String,
    pub tipo_jornada: String,
    pub tolerancia_entrada_minutos: i64,
    pub tolerancia_saida_minutos: i64,
    pub tolerancia_intervalo_minutos: i64,
    pub exige_marcacao_intervalo: bool,
    pub expected_minutes: i64,
    pub entrada_1: Option<String>,
    pub saida_1: Option<String>,
    pub entrada_2: Option<String>,
    pub saida_2: Option<String>,
    pub is_day_off: bool,
    pub is_holiday: bool,
    pub holiday_label: Option<String>,
    pub holiday_compensation: Option<String>,
    pub holiday_jornada_rule: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DailyCalculation {
    pub expected_minutes: i64,
    pub worked_minutes: i64,
    pub saldo_minutes: i64,
    pub atraso_minutes: i64,
    pub extra_minutes: i64,
    pub saida_antecipada_minutos: i64,
    pub inconsistente: bool,
    pub mensagens: Vec<String>,
}

#[derive(Debug, Clone)]
struct HolidayContext {
    descricao: String,
    regra_compensacao: Option<String>,
    regra_jornada: Option<String>,
}

#[derive(Debug, Clone)]
struct FlexSwapCandidate {
    date_iso: String,
    expected_minutes: i64,
    entrada_1: Option<String>,
    saida_1: Option<String>,
    entrada_2: Option<String>,
    saida_2: Option<String>,
}

fn date_week_bounds(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let weekday = i64::from(date.weekday().number_from_monday()) - 1;
    let start = date - chrono::Duration::days(weekday);
    let end = start + chrono::Duration::days(6);
    (start, end)
}

fn find_flex_swap_candidate(
    conn: &Connection,
    employee_id: i64,
    current_date: NaiveDate,
    current_weekday: u32,
    scheduled_is_day_off: bool,
) -> Result<Option<FlexSwapCandidate>, String> {
    let (week_start, week_end) = date_week_bounds(current_date);
    let mut stmt = conn
        .prepare(
            "SELECT jd.dia_semana, jd.entrada_1, jd.saida_1, jd.entrada_2, jd.saida_2,
                COALESCE(jd.carga_prevista_minutos, 0), COALESCE(jd.folga, 0)
         FROM funcionarios f
         INNER JOIN jornada_dias jd ON jd.jornada_id = f.jornada_id
         WHERE f.id = ?1",
        )
        .map_err(|err| format!("Falha ao preparar dias para heurística flexível: {err}"))?;

    let rows = stmt
        .query_map(params![employee_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|err| format!("Falha ao consultar dias da jornada flexível: {err}"))?;

    for row in rows {
        let (dia_semana, entrada_1, saida_1, entrada_2, saida_2, carga, folga) =
            row.map_err(|err| format!("Falha ao ler dia da jornada flexível: {err}"))?;
        let offset = dia_semana - 1;
        let candidate_date = week_start + chrono::Duration::days(offset);
        if candidate_date < week_start
            || candidate_date > week_end
            || candidate_date == current_date
        {
            continue;
        }
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM batidas WHERE funcionario_id = ?1 AND data_referencia = ?2",
                params![employee_id, candidate_date.format("%Y-%m-%d").to_string()],
                |row| row.get(0),
            )
            .map_err(|err| format!("Falha ao contar batidas da heurística flexível: {err}"))?;
        if scheduled_is_day_off {
            if folga == 0 && count == 0 {
                return Ok(Some(FlexSwapCandidate {
                    date_iso: candidate_date.format("%Y-%m-%d").to_string(),
                    expected_minutes: if carga > 0 {
                        carga
                    } else {
                        derive_minutes_from_pairs(
                            entrada_1.as_deref(),
                            saida_1.as_deref(),
                            entrada_2.as_deref(),
                            saida_2.as_deref(),
                        )
                    },
                    entrada_1,
                    saida_1,
                    entrada_2,
                    saida_2,
                }));
            }
        } else if dia_semana != i64::from(current_weekday) && folga == 1 && count > 0 {
            return Ok(Some(FlexSwapCandidate {
                date_iso: candidate_date.format("%Y-%m-%d").to_string(),
                expected_minutes: 0,
                entrada_1: None,
                saida_1: None,
                entrada_2: None,
                saida_2: None,
            }));
        }
    }
    Ok(None)
}

pub fn parse_hhmm_minutes(value: &str) -> Option<i64> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .ok()
        .map(|time| i64::from(time.hour() as i32) * 60 + i64::from(time.minute() as i32))
}

pub fn parse_iso_date(date: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|err| format!("Data inválida ({date}): {err}"))
}

fn derive_minutes_from_pairs(
    entrada_1: Option<&str>,
    saida_1: Option<&str>,
    entrada_2: Option<&str>,
    saida_2: Option<&str>,
) -> i64 {
    let mut total = 0i64;
    if let (Some(start), Some(end)) = (
        entrada_1.and_then(parse_hhmm_minutes),
        saida_1.and_then(parse_hhmm_minutes),
    ) {
        if end >= start {
            total += end - start;
        }
    }
    if let (Some(start), Some(end)) = (
        entrada_2.and_then(parse_hhmm_minutes),
        saida_2.and_then(parse_hhmm_minutes),
    ) {
        if end >= start {
            total += end - start;
        }
    }
    total
}

fn day_is_active(days: Option<String>, weekday: u32) -> bool {
    match days {
        Some(raw) => raw
            .split(',')
            .filter_map(|item| item.trim().parse::<u32>().ok())
            .any(|day| day == weekday),
        None => true,
    }
}

fn load_holiday_for_employee(
    conn: &Connection,
    employee_id: i64,
    date_iso: &str,
) -> Result<Option<HolidayContext>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT
                    f.descricao,
                    f.regra_compensacao,
                    f.regra_jornada,
                    CASE f.contexto_tipo
                        WHEN 'departamento' THEN 3
                        WHEN 'operacional' THEN 2
                        WHEN 'empresa' THEN 1
                        ELSE 0
                    END AS prioridade
             FROM feriados f
             INNER JOIN funcionarios fu ON fu.id = ?2
             LEFT JOIN feriados_empresas fe ON fe.feriado_id = f.id
             LEFT JOIN feriados_departamentos fd ON fd.feriado_id = f.id
             WHERE f.ativo = 1
               AND f.data = ?1
               AND (
                    f.contexto_tipo = 'global'
                    OR (
                        f.contexto_tipo = 'empresa'
                        AND (
                            (f.empresa_id IS NOT NULL AND fu.empresa_id = f.empresa_id)
                            OR (fe.empresa_id IS NOT NULL AND fu.empresa_id = fe.empresa_id)
                        )
                    )
                    OR (
                        f.contexto_tipo = 'departamento'
                        AND (
                            (f.departamento_id IS NOT NULL AND fu.departamento_id = f.departamento_id)
                            OR (fd.departamento_id IS NOT NULL AND fu.departamento_id = fd.departamento_id)
                        )
                    )
                    OR (
                        f.contexto_tipo = 'operacional'
                        AND (
                            (
                                (f.empresa_id IS NULL AND fe.empresa_id IS NULL)
                                OR fu.empresa_id = f.empresa_id
                                OR fu.empresa_id = fe.empresa_id
                            )
                            AND (
                                (f.departamento_id IS NULL AND fd.departamento_id IS NULL)
                                OR fu.departamento_id = f.departamento_id
                                OR fu.departamento_id = fd.departamento_id
                            )
                        )
                    )
               )
             ORDER BY prioridade DESC, f.id DESC
             LIMIT 1",
        )
        .map_err(|err| format!("Falha ao preparar leitura de feriado contextual: {err}"))?;

    stmt.query_row(params![date_iso, employee_id], |row| {
        Ok(HolidayContext {
            descricao: row.get(0)?,
            regra_compensacao: row.get(1)?,
            regra_jornada: row.get(2)?,
        })
    })
    .optional()
    .map_err(|err| format!("Falha ao consultar feriado contextual: {err}"))
}

pub fn resolve_schedule_for_employee(
    conn: &Connection,
    employee_id: i64,
    date_iso: &str,
) -> Result<ResolvedSchedule, String> {
    let date = parse_iso_date(date_iso)?;
    let weekday = date.weekday().number_from_monday();

    let mut stmt = conn
        .prepare(
            "SELECT f.jornada_id,
                    jt.descricao,
                    COALESCE(jt.tipo_jornada, 'fixa'),
                    jt.perfil_flexivel,
                    COALESCE(jt.permite_folga_movel, 0),
                    COALESCE(jt.permite_meia_folga, 0),
                    jt.dia_folga_base,
                    jt.periodo_meia_folga,
                    COALESCE(jt.heuristica_troca_folga, 1),
                    COALESCE(jt.tolerancia_entrada_minutos, 5),
                    COALESCE(jt.tolerancia_saida_minutos, 5),
                    COALESCE(jt.tolerancia_intervalo_minutos, 5),
                    COALESCE(jt.exige_marcacao_intervalo, 1),
                    jd.entrada_1,
                    jd.saida_1,
                    jd.entrada_2,
                    jd.saida_2,
                    COALESCE(jd.carga_prevista_minutos, 0),
                    COALESCE(jd.folga, 0),
                    h.descricao,
                    h.entrada_1,
                    h.saida_1,
                    h.entrada_2,
                    h.saida_2,
                    COALESCE(h.carga_horaria_minutos, 480),
                    es.dias_ativos
             FROM funcionarios f
             LEFT JOIN jornadas_trabalho jt ON jt.id = f.jornada_id
             LEFT JOIN jornada_dias jd ON jd.jornada_id = f.jornada_id AND jd.dia_semana = ?1
             LEFT JOIN horarios h ON h.id = f.horario_id
             LEFT JOIN escalas es ON es.id = f.escala_id
             WHERE f.id = ?2 LIMIT 1",
        )
        .map_err(|err| format!("Falha ao preparar consulta de jornada do funcionário: {err}"))?;

    let row = stmt
        .query_row(params![weekday, employee_id], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, i64>(16)?,
                row.get::<_, i64>(17)?,
                row.get::<_, Option<String>>(18)?,
                row.get::<_, Option<String>>(19)?,
                row.get::<_, Option<String>>(20)?,
                row.get::<_, Option<String>>(21)?,
                row.get::<_, Option<String>>(22)?,
                row.get::<_, i64>(23)?,
                row.get::<_, Option<String>>(24)?,
            ))
        })
        .optional()
        .map_err(|err| format!("Falha ao consultar jornada do funcionário: {err}"))?
        .ok_or_else(|| "Funcionário não encontrado para resolver jornada.".to_string())?;

    let holiday_context = load_holiday_for_employee(conn, employee_id, date_iso)?;

    let (
        jornada_id,
        jornada_nome,
        tipo_jornada,
        perfil_flexivel,
        permite_folga_movel,
        _permite_meia_folga,
        _dia_folga_base,
        _periodo_meia_folga,
        heuristica_troca_folga,
        tolerancia_entrada,
        tolerancia_saida,
        tolerancia_intervalo,
        exige_marcacao_intervalo,
        jd_entrada_1,
        jd_saida_1,
        jd_entrada_2,
        jd_saida_2,
        jd_carga,
        jd_folga,
        horario_nome,
        h_entrada_1,
        h_saida_1,
        h_entrada_2,
        h_saida_2,
        h_carga,
        escala_dias,
    ) = row;

    if jornada_id.is_some() {
        let mut expected_minutes = jd_carga;
        if expected_minutes <= 0 && jd_folga == 0 {
            expected_minutes = derive_minutes_from_pairs(
                jd_entrada_1.as_deref(),
                jd_saida_1.as_deref(),
                jd_entrada_2.as_deref(),
                jd_saida_2.as_deref(),
            );
        }

        let has_punches_today: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM batidas WHERE funcionario_id = ?1 AND data_referencia = ?2",
                params![employee_id, date_iso],
                |row| row.get(0),
            )
            .map_err(|err| format!("Falha ao verificar batidas da jornada flexível: {err}"))?;

        if permite_folga_movel == 1 && heuristica_troca_folga == 1 {
            if jd_folga == 1 && has_punches_today > 0 {
                if let Some(candidate) =
                    find_flex_swap_candidate(conn, employee_id, date, weekday, true)?
                {
                    expected_minutes = candidate.expected_minutes;
                    return Ok(ResolvedSchedule {
                        jornada_id,
                        jornada_nome: format!(
                            "{} • folga móvel",
                            jornada_nome
                                .clone()
                                .unwrap_or_else(|| "Jornada".to_string())
                        ),
                        tipo_jornada: perfil_flexivel
                            .clone()
                            .unwrap_or_else(|| "flexivel".to_string()),
                        tolerancia_entrada_minutos: tolerancia_entrada,
                        tolerancia_saida_minutos: tolerancia_saida,
                        tolerancia_intervalo_minutos: tolerancia_intervalo,
                        exige_marcacao_intervalo: exige_marcacao_intervalo == 1,
                        expected_minutes,
                        entrada_1: candidate.entrada_1,
                        saida_1: candidate.saida_1,
                        entrada_2: candidate.entrada_2,
                        saida_2: candidate.saida_2,
                        is_day_off: false,
                        is_holiday: false,
                        holiday_label: Some(format!("Folga movida para {}", candidate.date_iso)),
                        holiday_compensation: None,
                        holiday_jornada_rule: Some("swap_week_day_off".to_string()),
                    });
                }
            }
            if jd_folga == 0 && has_punches_today == 0 {
                if let Some(candidate) =
                    find_flex_swap_candidate(conn, employee_id, date, weekday, false)?
                {
                    return Ok(ResolvedSchedule {
                        jornada_id,
                        jornada_nome: format!(
                            "{} • folga móvel",
                            jornada_nome
                                .clone()
                                .unwrap_or_else(|| "Jornada".to_string())
                        ),
                        tipo_jornada: perfil_flexivel
                            .clone()
                            .unwrap_or_else(|| "flexivel".to_string()),
                        tolerancia_entrada_minutos: tolerancia_entrada,
                        tolerancia_saida_minutos: tolerancia_saida,
                        tolerancia_intervalo_minutos: tolerancia_intervalo,
                        exige_marcacao_intervalo: exige_marcacao_intervalo == 1,
                        expected_minutes: 0,
                        entrada_1: None,
                        saida_1: None,
                        entrada_2: None,
                        saida_2: None,
                        is_day_off: true,
                        is_holiday: false,
                        holiday_label: Some(format!("Folga compensada em {}", candidate.date_iso)),
                        holiday_compensation: None,
                        holiday_jornada_rule: Some("swap_week_day_off".to_string()),
                    });
                }
            }
        }

        if let Some(holiday) = holiday_context.clone() {
            return Ok(ResolvedSchedule {
                jornada_id,
                jornada_nome: holiday.descricao.clone(),
                tipo_jornada: "feriado".to_string(),
                tolerancia_entrada_minutos: tolerancia_entrada,
                tolerancia_saida_minutos: tolerancia_saida,
                tolerancia_intervalo_minutos: tolerancia_intervalo,
                exige_marcacao_intervalo: exige_marcacao_intervalo == 1,
                expected_minutes: 0,
                entrada_1: None,
                saida_1: None,
                entrada_2: None,
                saida_2: None,
                is_day_off: true,
                is_holiday: true,
                holiday_label: Some(holiday.descricao),
                holiday_compensation: holiday.regra_compensacao,
                holiday_jornada_rule: holiday.regra_jornada,
            });
        }

        return Ok(ResolvedSchedule {
            jornada_id,
            jornada_nome: jornada_nome.unwrap_or_else(|| "Jornada".to_string()),
            tipo_jornada,
            tolerancia_entrada_minutos: tolerancia_entrada,
            tolerancia_saida_minutos: tolerancia_saida,
            tolerancia_intervalo_minutos: tolerancia_intervalo,
            exige_marcacao_intervalo: exige_marcacao_intervalo == 1,
            expected_minutes: if jd_folga == 1 { 0 } else { expected_minutes },
            entrada_1: jd_entrada_1,
            saida_1: jd_saida_1,
            entrada_2: jd_entrada_2,
            saida_2: jd_saida_2,
            is_day_off: jd_folga == 1,
            is_holiday: false,
            holiday_label: None,
            holiday_compensation: None,
            holiday_jornada_rule: None,
        });
    }

    let active = day_is_active(escala_dias, weekday);
    let mut expected_minutes = if active { h_carga } else { 0 };
    if expected_minutes <= 0 && active {
        expected_minutes = derive_minutes_from_pairs(
            h_entrada_1.as_deref(),
            h_saida_1.as_deref(),
            h_entrada_2.as_deref(),
            h_saida_2.as_deref(),
        );
    }

    if let Some(holiday) = holiday_context {
        return Ok(ResolvedSchedule {
            jornada_id: None,
            jornada_nome: holiday.descricao.clone(),
            tipo_jornada: "feriado".to_string(),
            tolerancia_entrada_minutos: 5,
            tolerancia_saida_minutos: 5,
            tolerancia_intervalo_minutos: 5,
            exige_marcacao_intervalo: true,
            expected_minutes: 0,
            entrada_1: None,
            saida_1: None,
            entrada_2: None,
            saida_2: None,
            is_day_off: true,
            is_holiday: true,
            holiday_label: Some(holiday.descricao),
            holiday_compensation: holiday.regra_compensacao,
            holiday_jornada_rule: holiday.regra_jornada,
        });
    }

    Ok(ResolvedSchedule {
        jornada_id: None,
        jornada_nome: horario_nome.unwrap_or_else(|| "Horário padrão".to_string()),
        tipo_jornada: if active {
            "horario".to_string()
        } else {
            "folga".to_string()
        },
        tolerancia_entrada_minutos: 5,
        tolerancia_saida_minutos: 5,
        tolerancia_intervalo_minutos: 5,
        exige_marcacao_intervalo: true,
        expected_minutes,
        entrada_1: h_entrada_1,
        saida_1: h_saida_1,
        entrada_2: h_entrada_2,
        saida_2: h_saida_2,
        is_day_off: !active,
        is_holiday: false,
        holiday_label: None,
        holiday_compensation: None,
        holiday_jornada_rule: None,
    })
}

pub fn calculate_day(schedule: &ResolvedSchedule, batidas: &[String]) -> DailyCalculation {
    let mut worked_minutes = 0i64;
    let mut inconsistente = !batidas.len().is_multiple_of(2);
    let mut mensagens: Vec<String> = Vec::new();

    let mut index = 0usize;
    while index + 1 < batidas.len() {
        match (
            parse_hhmm_minutes(&batidas[index]),
            parse_hhmm_minutes(&batidas[index + 1]),
        ) {
            (Some(start), Some(end)) if end >= start => worked_minutes += end - start,
            _ => inconsistente = true,
        }
        index += 2;
    }

    let mut atraso_minutos = 0i64;
    if let (Some(first), Some(expected_start)) = (
        batidas.first().and_then(|value| parse_hhmm_minutes(value)),
        schedule.entrada_1.as_deref().and_then(parse_hhmm_minutes),
    ) {
        let tolerated = expected_start + schedule.tolerancia_entrada_minutos;
        if first > tolerated {
            atraso_minutos = first - expected_start;
            mensagens.push(format!("Atraso identificado: {atraso_minutos} minuto(s)."));
        }
    }

    let mut saida_antecipada_minutos = 0i64;
    if let (Some(last), Some(expected_end)) = (
        batidas.last().and_then(|value| parse_hhmm_minutes(value)),
        schedule
            .saida_2
            .as_deref()
            .and_then(parse_hhmm_minutes)
            .or_else(|| schedule.saida_1.as_deref().and_then(parse_hhmm_minutes)),
    ) {
        let tolerated = expected_end - schedule.tolerancia_saida_minutos;
        if last < tolerated {
            saida_antecipada_minutos = expected_end - last;
            mensagens.push(format!(
                "Saída antecipada identificada: {saida_antecipada_minutos} minuto(s)."
            ));
        }
    }

    let saldo_minutes = worked_minutes - schedule.expected_minutes;
    let extra_minutes = saldo_minutes.max(0);

    if inconsistente {
        mensagens.push("Quantidade de batidas ímpar ou horários inconsistentes.".to_string());
    }

    DailyCalculation {
        expected_minutes: schedule.expected_minutes,
        worked_minutes,
        saldo_minutes,
        atraso_minutes: atraso_minutos,
        extra_minutes,
        saida_antecipada_minutos,
        inconsistente,
        mensagens,
    }
}

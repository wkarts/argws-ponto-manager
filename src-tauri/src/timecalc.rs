use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use rusqlite::{params, Connection, OptionalExtension};

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

pub fn parse_hhmm_minutes(value: &str) -> Option<i64> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .ok()
        .map(|time| i64::from(time.hour() as i32) * 60 + i64::from(time.minute() as i32))
}

pub fn parse_iso_date(date: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|err| format!("Data inválida ({date}): {err}"))
}

fn derive_minutes_from_pairs(entrada_1: Option<&str>, saida_1: Option<&str>, entrada_2: Option<&str>, saida_2: Option<&str>) -> i64 {
    let mut total = 0i64;
    if let (Some(start), Some(end)) = (entrada_1.and_then(parse_hhmm_minutes), saida_1.and_then(parse_hhmm_minutes)) {
        if end >= start {
            total += end - start;
        }
    }
    if let (Some(start), Some(end)) = (entrada_2.and_then(parse_hhmm_minutes), saida_2.and_then(parse_hhmm_minutes)) {
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
             WHERE f.id = ?2 LIMIT 1"
        )
        .map_err(|err| format!("Falha ao preparar consulta de jornada do funcionário: {err}"))?;

    let row = stmt
        .query_row(params![weekday, employee_id], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, i64>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, Option<String>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
                row.get::<_, i64>(18)?,
                row.get::<_, Option<String>>(19)?,
            ))
        })
        .optional()
        .map_err(|err| format!("Falha ao consultar jornada do funcionário: {err}"))?
        .ok_or_else(|| "Funcionário não encontrado para resolver jornada.".to_string())?;

    let (
        jornada_id,
        jornada_nome,
        tipo_jornada,
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
        if expected_minutes <= 0 {
            expected_minutes = derive_minutes_from_pairs(
                jd_entrada_1.as_deref(),
                jd_saida_1.as_deref(),
                jd_entrada_2.as_deref(),
                jd_saida_2.as_deref(),
            );
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

    Ok(ResolvedSchedule {
        jornada_id: None,
        jornada_nome: horario_nome.unwrap_or_else(|| "Horário padrão".to_string()),
        tipo_jornada: if active { "horario".to_string() } else { "folga".to_string() },
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
    })
}

pub fn calculate_day(schedule: &ResolvedSchedule, batidas: &[String]) -> DailyCalculation {
    let mut worked_minutes = 0i64;
    let mut inconsistente = batidas.len() % 2 != 0;
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
    let expected_end = schedule
        .saida_2
        .as_deref()
        .and_then(parse_hhmm_minutes)
        .or_else(|| schedule.saida_1.as_deref().and_then(parse_hhmm_minutes));

    if let (Some(last), Some(expected_end)) = (
        batidas.last().and_then(|value| parse_hhmm_minutes(value)),
        expected_end,
    ) {
        let tolerated = expected_end - schedule.tolerancia_saida_minutos;
        if last < tolerated {
            saida_antecipada_minutos = expected_end - last;
            mensagens.push(format!(
                "Saída antecipada identificada: {saida_antecipada_minutos} minuto(s)."
            ));
        }
    }

    if schedule.exige_marcacao_intervalo
        && schedule.saida_1.is_some()
        && schedule.entrada_2.is_some()
        && batidas.len() >= 4
    {
        if let (Some(actual_out), Some(actual_in), Some(expected_out), Some(expected_in)) = (
            parse_hhmm_minutes(&batidas[1]),
            parse_hhmm_minutes(&batidas[2]),
            schedule.saida_1.as_deref().and_then(parse_hhmm_minutes),
            schedule.entrada_2.as_deref().and_then(parse_hhmm_minutes),
        ) {
            let expected_interval = expected_in.saturating_sub(expected_out);
            let actual_interval = actual_in.saturating_sub(actual_out);
            if actual_interval + schedule.tolerancia_intervalo_minutos < expected_interval {
                mensagens.push(format!(
                    "Intervalo inferior ao previsto: esperado {} min, realizado {} min.",
                    expected_interval, actual_interval
                ));
            }
        }
    }

    if schedule.is_day_off && !batidas.is_empty() {
        mensagens.push("Dia marcado como folga com presença de batidas.".to_string());
    }

    if schedule.expected_minutes == 0 && !schedule.is_day_off && batidas.is_empty() {
        mensagens.push("Jornada sem carga horária prevista para o dia.".to_string());
    }

    let saldo_minutes = worked_minutes - schedule.expected_minutes;
    let extra_minutes = saldo_minutes.max(0);

    DailyCalculation {
        expected_minutes: schedule.expected_minutes,
        worked_minutes,
        saldo_minutes,
        atraso_minutos,
        extra_minutes,
        saida_antecipada_minutos,
        inconsistente,
        mensagens,
    }
}

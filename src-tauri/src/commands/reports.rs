use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::{params, params_from_iter, OptionalExtension};
use std::{collections::BTreeSet, fs, path::PathBuf};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::open_connection,
    models::{ApuracaoDia, ApuracaoRequest, ApuracaoResumo, PunchFilters},
    timecalc::{calculate_day, parse_iso_date, resolve_schedule_for_employee},
};

#[derive(Debug)]
struct FuncionarioApuracaoBase {
    id: i64,
    nome: String,
}

#[derive(Debug, Default)]
struct DayOccurrenceData {
    labels: Vec<String>,
    minutos_abonados: i64,
    abonar_dia: bool,
}

fn build_punch_query(filters: &PunchFilters) -> (String, Vec<rusqlite::types::Value>) {
    let mut sql = String::from(
        "SELECT b.id,
                f.nome AS funcionario_nome,
                COALESCE(e.descricao, '') AS equipamento_nome,
                b.data_referencia,
                b.hora,
                b.tipo,
                b.origem
         FROM batidas b
         INNER JOIN funcionarios f ON f.id = b.funcionario_id
         LEFT JOIN equipamentos e ON e.id = b.equipamento_id
         WHERE 1 = 1",
    );

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(funcionario_id) = filters.funcionario_id {
        sql.push_str(" AND b.funcionario_id = ?");
        params.push(rusqlite::types::Value::Integer(funcionario_id));
    }

    if let Some(data_inicial) = &filters.data_inicial {
        sql.push_str(" AND b.data_referencia >= ?");
        params.push(rusqlite::types::Value::Text(data_inicial.clone()));
    }

    if let Some(data_final) = &filters.data_final {
        sql.push_str(" AND b.data_referencia <= ?");
        params.push(rusqlite::types::Value::Text(data_final.clone()));
    }

    sql.push_str(" ORDER BY b.data_referencia ASC, f.nome ASC, b.hora ASC");
    (sql, params)
}

#[tauri::command]
pub fn exportar_batidas_csv(
    state: State<'_, SharedState>,
    filters: PunchFilters,
) -> Result<String, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let (sql, params) = build_punch_query(&filters);

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar exportação CSV: {err}"))?;

    let rows = stmt
        .query_map(params_from_iter(params.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|err| format!("Falha ao executar exportação CSV: {err}"))?;

    let mut csv = String::from("id,funcionario,equipamento,data,hora,tipo,origem\n");
    for row in rows {
        let (id, funcionario, equipamento, data, hora, tipo, origem) =
            row.map_err(|err| format!("Falha ao mapear linha do CSV: {err}"))?;
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            id,
            escape_csv(&funcionario),
            escape_csv(&equipamento),
            data,
            hora,
            escape_csv(&tipo),
            escape_csv(&origem)
        ));
    }

    let export_dir = db_path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Diretório do banco inválido para exportação.".to_string())?
        .join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|err| format!("Falha ao criar diretório de exportação: {err}"))?;

    let file_name = format!(
        "batidas_{}.csv",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file_path = export_dir.join(file_name);
    fs::write(&file_path, csv).map_err(|err| format!("Falha ao gravar CSV: {err}"))?;

    Ok(file_path.to_string_lossy().to_string())
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn load_day_occurrences(
    conn: &rusqlite::Connection,
    funcionario_id: i64,
    data: &str,
) -> Result<DayOccurrenceData, String> {
    let mut stmt = conn
        .prepare(
            "SELECT o.tipo,
                    o.abonar_dia,
                    COALESCE(o.minutos_abonados, 0),
                    COALESCE(j.descricao, ''),
                    COALESCE(o.observacao, ''),
                    COALESCE(o.anexo_nome, '')
             FROM ocorrencias_ponto o
             LEFT JOIN justificativas j ON j.id = o.justificativa_id
             WHERE o.funcionario_id = ?1 AND o.data_referencia = ?2
             ORDER BY o.id ASC",
        )
        .map_err(|err| format!("Falha ao preparar ocorrências do dia: {err}"))?;

    let rows = stmt
        .query_map(params![funcionario_id, data], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|err| format!("Falha ao consultar ocorrências do dia: {err}"))?;

    let mut data_occ = DayOccurrenceData::default();
    for row in rows {
        let (tipo, abonar_dia, minutos_abonados, justificativa, observacao, anexo_nome) =
            row.map_err(|err| format!("Falha ao mapear ocorrência do dia: {err}"))?;

        let mut label = if justificativa.trim().is_empty() {
            tipo.replace('_', " ")
        } else {
            justificativa
        };
        if minutos_abonados > 0 {
            label.push_str(&format!(" ({} min)", minutos_abonados));
        }
        if !anexo_nome.trim().is_empty() {
            label.push_str(" [anexo]");
        }
        if !observacao.trim().is_empty() {
            label.push_str(&format!(": {}", observacao));
        }

        data_occ.labels.push(label);
        data_occ.minutos_abonados += minutos_abonados;
        if abonar_dia == 1 {
            data_occ.abonar_dia = true;
        }
    }

    Ok(data_occ)
}

fn has_manual_adjustment(
    conn: &rusqlite::Connection,
    funcionario_id: i64,
    data: &str,
) -> Result<bool, String> {
    let count: Option<i64> = conn
        .query_row(
            "SELECT COUNT(*) FROM batidas WHERE funcionario_id = ?1 AND data_referencia = ?2 AND manual_ajuste = 1",
            params![funcionario_id, data],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar ajustes manuais do dia: {err}"))?;
    Ok(count.unwrap_or(0) > 0)
}

pub fn apurar_periodo_internal(
    conn: &rusqlite::Connection,
    payload: &ApuracaoRequest,
) -> Result<ApuracaoResumo, String> {
    let data_inicial = payload
        .data_inicial
        .clone()
        .ok_or_else(|| "dataInicial é obrigatória.".to_string())?;
    let data_final = payload
        .data_final
        .clone()
        .ok_or_else(|| "dataFinal é obrigatória.".to_string())?;

    let inicio = parse_iso_date(&data_inicial)?;
    let fim = parse_iso_date(&data_final)?;
    if fim < inicio {
        return Err("A data final deve ser maior ou igual à data inicial.".to_string());
    }

    let mut funcionarios_sql = String::from(
        "SELECT f.id, f.nome
         FROM funcionarios f
         WHERE f.ativo = 1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(funcionario_id) = payload.funcionario_id {
        funcionarios_sql.push_str(" AND f.id = ?");
        params_vec.push(rusqlite::types::Value::Integer(funcionario_id));
    }
    funcionarios_sql.push_str(" ORDER BY f.nome ASC");

    let mut stmt = conn
        .prepare(&funcionarios_sql)
        .map_err(|err| format!("Falha ao preparar funcionários da apuração: {err}"))?;

    let funcionario_rows = stmt
        .query_map(params_from_iter(params_vec.iter()), |row| {
            Ok(FuncionarioApuracaoBase {
                id: row.get(0)?,
                nome: row.get(1)?,
            })
        })
        .map_err(|err| format!("Falha ao executar funcionários da apuração: {err}"))?;

    let funcionarios: Result<Vec<_>, _> = funcionario_rows.collect();
    let funcionarios =
        funcionarios.map_err(|err| format!("Falha ao mapear funcionários da apuração: {err}"))?;

    let mut rows: Vec<ApuracaoDia> = Vec::new();
    let mut total_esperado = 0i64;
    let mut total_trabalhado = 0i64;
    let mut total_saldo = 0i64;
    let mut total_atraso = 0i64;
    let mut total_extra = 0i64;

    for funcionario in &funcionarios {
        let mut date: NaiveDate = inicio;
        while date <= fim {
            let current_date = date.format("%Y-%m-%d").to_string();
            let mut punch_stmt = conn
                .prepare(
                    "SELECT hora
                     FROM batidas
                     WHERE funcionario_id = ?1 AND data_referencia = ?2
                     ORDER BY hora ASC, id ASC",
                )
                .map_err(|err| format!("Falha ao preparar batidas da apuração: {err}"))?;

            let punches = punch_stmt
                .query_map(params![funcionario.id, current_date], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|err| format!("Falha ao executar batidas da apuração: {err}"))?;

            let batidas: Result<Vec<_>, _> = punches.collect();
            let batidas =
                batidas.map_err(|err| format!("Falha ao mapear batidas da apuração: {err}"))?;

            let schedule = resolve_schedule_for_employee(conn, funcionario.id, &current_date)?;
            let mut calc = calculate_day(&schedule, &batidas);
            let occurrence_data = load_day_occurrences(conn, funcionario.id, &current_date)?;

            if occurrence_data.abonar_dia {
                if calc.worked_minutes < calc.expected_minutes {
                    calc.worked_minutes = calc.expected_minutes;
                }
                if calc.saldo_minutes < 0 {
                    calc.saldo_minutes = 0;
                }
                calc.atraso_minutes = 0;
                calc.saida_antecipada_minutos = 0;
                calc.inconsistente = false;
                calc.mensagens
                    .push("Dia abonado por justificativa/atestado.".to_string());
            } else if occurrence_data.minutos_abonados > 0 && calc.saldo_minutes < 0 {
                let deficit = calc.saldo_minutes.abs();
                let compensado = occurrence_data.minutos_abonados.min(deficit);
                calc.worked_minutes += compensado;
                calc.saldo_minutes += compensado;
                if calc.saldo_minutes >= 0 {
                    calc.atraso_minutes = 0;
                    calc.saida_antecipada_minutos = 0;
                }
                calc.mensagens
                    .push(format!("Abono parcial aplicado: {} minuto(s).", compensado));
            }

            if has_manual_adjustment(conn, funcionario.id, &current_date)? {
                calc.mensagens
                    .push("Contém batida lançada/ajustada manualmente.".to_string());
            }

            if calc.expected_minutes > 0
                || !batidas.is_empty()
                || !calc.mensagens.is_empty()
                || !occurrence_data.labels.is_empty()
                || (schedule.is_day_off && !batidas.is_empty())
            {
                if schedule.is_day_off && !batidas.is_empty() {
                    calc.mensagens
                        .push("Batidas registradas em dia configurado como folga.".to_string());
                }

                total_esperado += calc.expected_minutes;
                total_trabalhado += calc.worked_minutes;
                total_saldo += calc.saldo_minutes;
                total_atraso += calc.atraso_minutes;
                total_extra += calc.extra_minutes;

                rows.push(ApuracaoDia {
                    funcionario_id: funcionario.id,
                    funcionario_nome: funcionario.nome.clone(),
                    data: current_date,
                    jornada_nome: schedule.jornada_nome,
                    tipo_jornada: if schedule.is_day_off {
                        format!("folga-{}", date.weekday().number_from_monday())
                    } else {
                        schedule.tipo_jornada
                    },
                    horario_esperado_minutos: calc.expected_minutes,
                    trabalhado_minutos: calc.worked_minutes,
                    saldo_minutos: calc.saldo_minutes,
                    atraso_minutos: calc.atraso_minutes,
                    extra_minutos: calc.extra_minutes,
                    saida_antecipada_minutos: calc.saida_antecipada_minutos,
                    mensagens: calc.mensagens,
                    batidas,
                    ocorrencias: occurrence_data.labels,
                    minutos_abonados: occurrence_data.minutos_abonados,
                    abonado: occurrence_data.abonar_dia,
                    inconsistente: calc.inconsistente,
                });
            }

            date += Duration::days(1);
        }
    }

    let mut funcionarios_set = BTreeSet::new();
    for row in &rows {
        funcionarios_set.insert(row.funcionario_id);
    }

    Ok(ApuracaoResumo {
        total_funcionarios: funcionarios_set.len(),
        total_dias: rows.len(),
        total_esperado_minutos: total_esperado,
        total_trabalhado_minutos: total_trabalhado,
        total_saldo_minutos: total_saldo,
        total_atraso_minutos: total_atraso,
        total_extra_minutos: total_extra,
        rows,
    })
}

#[tauri::command]
pub fn apurar_periodo(
    state: State<'_, SharedState>,
    payload: ApuracaoRequest,
) -> Result<ApuracaoResumo, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    apurar_periodo_internal(&conn, &payload)
}

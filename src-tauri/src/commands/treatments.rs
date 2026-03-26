use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use chrono::{NaiveDate, Utc};
use rusqlite::{params, params_from_iter, OptionalExtension};
use serde_json::{json, Map, Value};
use std::{fs, path::PathBuf};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::ApuracaoRequest,
};

use super::reports::apurar_periodo_internal;

fn get_string(payload: &Map<String, Value>, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|value| match value {
            Value::String(text) => Some(text.trim().to_string()),
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(flag) => Some(if *flag {
                "1".to_string()
            } else {
                "0".to_string()
            }),
            _ => None,
        })
        .filter(|value| !value.is_empty())
}

fn get_bool(payload: &Map<String, Value>, key: &str, default: bool) -> i64 {
    match payload.get(key) {
        Some(Value::Bool(flag)) => {
            if *flag {
                1
            } else {
                0
            }
        }
        Some(Value::Number(number)) => {
            if number.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Some(Value::String(text)) => {
            if matches!(text.trim(), "1" | "true" | "TRUE" | "sim" | "SIM") {
                1
            } else {
                0
            }
        }
        _ => {
            if default {
                1
            } else {
                0
            }
        }
    }
}

fn get_i64(payload: &Map<String, Value>, key: &str) -> Option<i64> {
    payload.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn get_id(payload: &Map<String, Value>) -> Option<i64> {
    get_i64(payload, "id")
}

fn validate_iso_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

#[tauri::command]
pub fn ocorrencia_list(
    state: State<'_, SharedState>,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = String::from(
        "SELECT o.id,
                o.funcionario_id,
                f.nome AS funcionario_nome,
                o.data_referencia,
                o.justificativa_id,
                COALESCE(j.descricao, '') AS justificativa_nome,
                o.tipo,
                o.abonar_dia,
                o.minutos_abonados,
                o.observacao,
                o.anexo_nome,
                o.anexo_mime,
                o.created_at,
                o.updated_at
         FROM ocorrencias_ponto o
         INNER JOIN funcionarios f ON f.id = o.funcionario_id
         LEFT JOIN justificativas j ON j.id = o.justificativa_id
         WHERE 1 = 1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(funcionario_id) = get_i64(&filters, "funcionarioId") {
        sql.push_str(" AND o.funcionario_id = ?");
        params_vec.push(rusqlite::types::Value::Integer(funcionario_id));
    }

    if let Some(date) = get_string(&filters, "dataInicial") {
        sql.push_str(" AND o.data_referencia >= ?");
        params_vec.push(rusqlite::types::Value::Text(date));
    }

    if let Some(date) = get_string(&filters, "dataFinal") {
        sql.push_str(" AND o.data_referencia <= ?");
        params_vec.push(rusqlite::types::Value::Text(date));
    }

    sql.push_str(" ORDER BY o.data_referencia DESC, f.nome ASC, o.id DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de ocorrências: {err}"))?;

    let rows = stmt
        .query_map(params_from_iter(params_vec.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao consultar ocorrências: {err}"))?;
    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear ocorrências: {err}"))
}

#[tauri::command]
pub fn ocorrencia_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let id = get_id(&payload);

    let funcionario_id = get_i64(&payload, "funcionario_id")
        .ok_or_else(|| "funcionario_id é obrigatório.".to_string())?;
    let data_referencia = get_string(&payload, "data_referencia")
        .ok_or_else(|| "data_referencia é obrigatória.".to_string())?;
    let tipo = get_string(&payload, "tipo")
        .ok_or_else(|| "tipo da ocorrência é obrigatório.".to_string())?;

    if !validate_iso_date(&data_referencia) {
        return Err("Data da ocorrência inválida. Utilize YYYY-MM-DD.".to_string());
    }

    let funcionario_exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM funcionarios WHERE id = ?1 LIMIT 1",
            [funcionario_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar funcionário da ocorrência: {err}"))?;
    if funcionario_exists.is_none() {
        return Err("Funcionário da ocorrência não existe.".to_string());
    }

    let justificativa_id = get_i64(&payload, "justificativa_id");
    if let Some(justificativa_id) = justificativa_id {
        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM justificativas WHERE id = ?1 LIMIT 1",
                [justificativa_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| format!("Falha ao validar justificativa da ocorrência: {err}"))?;
        if exists.is_none() {
            return Err("Justificativa informada não existe.".to_string());
        }
    }

    let mut abonar_dia = get_bool(&payload, "abonar_dia", false);
    if abonar_dia == 0 {
        if let Some(justificativa_id) = justificativa_id {
            let justificativa_abono: Option<i64> = conn
                .query_row(
                    "SELECT abono FROM justificativas WHERE id = ?1 LIMIT 1",
                    [justificativa_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| format!("Falha ao ler abono padrão da justificativa: {err}"))?;
            if justificativa_abono.unwrap_or(0) == 1
                && matches!(tipo.as_str(), "atestado" | "falta_justificada" | "abono")
            {
                abonar_dia = 1;
            }
        }
    }

    let minutos_abonados = get_i64(&payload, "minutos_abonados").unwrap_or(0).max(0);
    let observacao = get_string(&payload, "observacao");
    let anexo_nome = get_string(&payload, "anexo_nome");
    let anexo_mime = get_string(&payload, "anexo_mime");
    let anexo_base64 = get_string(&payload, "anexo_base64");

    let record_id = if let Some(existing_id) = id {
        conn.execute(
            "UPDATE ocorrencias_ponto
             SET funcionario_id = ?1,
                 data_referencia = ?2,
                 justificativa_id = ?3,
                 tipo = ?4,
                 abonar_dia = ?5,
                 minutos_abonados = ?6,
                 observacao = ?7,
                 anexo_nome = ?8,
                 anexo_mime = ?9,
                 anexo_base64 = ?10,
                 updated_at = ?11
             WHERE id = ?12",
            params![
                funcionario_id,
                data_referencia,
                justificativa_id,
                tipo,
                abonar_dia,
                minutos_abonados,
                observacao,
                anexo_nome,
                anexo_mime,
                anexo_base64,
                now,
                existing_id,
            ],
        )
        .map_err(|err| format!("Falha ao atualizar ocorrência: {err}"))?;
        existing_id
    } else {
        conn.execute(
            "INSERT INTO ocorrencias_ponto (
                funcionario_id, data_referencia, justificativa_id, tipo, abonar_dia,
                minutos_abonados, observacao, anexo_nome, anexo_mime, anexo_base64,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![
                funcionario_id,
                data_referencia,
                justificativa_id,
                tipo,
                abonar_dia,
                minutos_abonados,
                observacao,
                anexo_nome,
                anexo_mime,
                anexo_base64,
                now,
            ],
        )
        .map_err(|err| format!("Falha ao inserir ocorrência: {err}"))?;
        conn.last_insert_rowid()
    };

    let saved = conn
        .query_row(
            "SELECT o.id,
                    o.funcionario_id,
                    f.nome AS funcionario_nome,
                    o.data_referencia,
                    o.justificativa_id,
                    COALESCE(j.descricao, '') AS justificativa_nome,
                    o.tipo,
                    o.abonar_dia,
                    o.minutos_abonados,
                    o.observacao,
                    o.anexo_nome,
                    o.anexo_mime,
                    o.created_at,
                    o.updated_at
             FROM ocorrencias_ponto o
             INNER JOIN funcionarios f ON f.id = o.funcionario_id
             LEFT JOIN justificativas j ON j.id = o.justificativa_id
             WHERE o.id = ?1",
            [record_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler ocorrência salva: {err}"))?
        .ok_or_else(|| "Ocorrência salva não encontrada.".to_string())?;

    let action_name = if id.is_some() { "update" } else { "create" };
    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "ocorrencias_ponto",
        action_name,
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "ocorrencias_ponto",
        action_name,
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn ocorrencia_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let affected = conn
        .execute("DELETE FROM ocorrencias_ponto WHERE id = ?1", params![id])
        .map_err(|err| format!("Falha ao excluir ocorrência: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id });
        write_audit(&conn, "ocorrencias_ponto", "delete", Some(id), &payload)?;
        enqueue_sync(&conn, "ocorrencias_ponto", "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

#[tauri::command]
pub fn ocorrencia_exportar_anexo(state: State<'_, SharedState>, id: i64) -> Result<String, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT COALESCE(anexo_nome, ''), COALESCE(anexo_base64, '')
             FROM ocorrencias_ponto
             WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar anexo da ocorrência: {err}"))?;

    let (file_name, base64_data) = row.ok_or_else(|| "Ocorrência não encontrada.".to_string())?;
    if file_name.trim().is_empty() || base64_data.trim().is_empty() {
        return Err("Esta ocorrência não possui anexo.".to_string());
    }

    let payload = if let Some((_, rest)) = base64_data.split_once(',') {
        rest.to_string()
    } else {
        base64_data
    };

    let bytes = BASE64_STANDARD
        .decode(payload.as_bytes())
        .map_err(|err| format!("Falha ao decodificar anexo: {err}"))?;

    let export_dir = db_path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Diretório do banco inválido para exportação.".to_string())?
        .join("exports")
        .join("anexos");
    fs::create_dir_all(&export_dir)
        .map_err(|err| format!("Falha ao criar diretório de anexos exportados: {err}"))?;

    let file_path = export_dir.join(format!("{}_{}", id, sanitize_file_name(&file_name)));
    fs::write(&file_path, bytes)
        .map_err(|err| format!("Falha ao gravar anexo exportado: {err}"))?;
    Ok(file_path.to_string_lossy().to_string())
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_minutes(minutos: i64) -> String {
    let sinal = if minutos < 0 { "-" } else { "" };
    let abs = minutos.abs();
    let horas = abs / 60;
    let mins = abs % 60;
    format!("{}{:02}:{:02}", sinal, horas, mins)
}

#[tauri::command]
pub fn fechamento_list(
    state: State<'_, SharedState>,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = String::from(
        "SELECT fm.id,
                fm.funcionario_id,
                f.nome AS funcionario_nome,
                fm.empresa_id,
                COALESCE(e.nome, '') AS empresa_nome,
                fm.ano,
                fm.mes,
                fm.data_inicial,
                fm.data_final,
                fm.total_esperado_minutos,
                fm.total_trabalhado_minutos,
                fm.total_saldo_minutos,
                fm.total_atraso_minutos,
                fm.total_extra_minutos,
                fm.total_banco_horas_minutos,
                fm.relatorio_path,
                fm.status,
                fm.created_at,
                fm.updated_at
         FROM fechamentos_mensais fm
         INNER JOIN funcionarios f ON f.id = fm.funcionario_id
         LEFT JOIN empresas e ON e.id = fm.empresa_id
         WHERE 1 = 1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(funcionario_id) = get_i64(&filters, "funcionarioId") {
        sql.push_str(" AND fm.funcionario_id = ?");
        params_vec.push(rusqlite::types::Value::Integer(funcionario_id));
    }

    if let Some(ano) = get_i64(&filters, "ano") {
        sql.push_str(" AND fm.ano = ?");
        params_vec.push(rusqlite::types::Value::Integer(ano));
    }

    if let Some(mes) = get_i64(&filters, "mes") {
        sql.push_str(" AND fm.mes = ?");
        params_vec.push(rusqlite::types::Value::Integer(mes));
    }

    sql.push_str(" ORDER BY fm.ano DESC, fm.mes DESC, f.nome ASC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de fechamentos: {err}"))?;
    let rows = stmt
        .query_map(params_from_iter(params_vec.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao consultar fechamentos: {err}"))?;
    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear fechamentos: {err}"))
}

#[tauri::command]
pub fn fechamento_gerar_relatorio(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();

    let funcionario_id = get_i64(&payload, "funcionarioId")
        .ok_or_else(|| "funcionarioId é obrigatório para gerar o fechamento.".to_string())?;
    let ano = get_i64(&payload, "ano").ok_or_else(|| "ano é obrigatório.".to_string())? as i32;
    let mes = get_i64(&payload, "mes").ok_or_else(|| "mes é obrigatório.".to_string())? as u32;

    let inicio = NaiveDate::from_ymd_opt(ano, mes, 1)
        .ok_or_else(|| "Período do fechamento inválido.".to_string())?;
    let proximo_mes = if mes == 12 {
        NaiveDate::from_ymd_opt(ano + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(ano, mes + 1, 1)
    }
    .ok_or_else(|| "Período do fechamento inválido.".to_string())?;
    let fim = proximo_mes
        .pred_opt()
        .ok_or_else(|| "Falha ao calcular data final do mês.".to_string())?;

    let apuracao = apurar_periodo_internal(
        &conn,
        &ApuracaoRequest {
            funcionario_id: Some(funcionario_id),
            data_inicial: Some(inicio.format("%Y-%m-%d").to_string()),
            data_final: Some(fim.format("%Y-%m-%d").to_string()),
        },
    )?;

    let (funcionario_nome, empresa_id, empresa_nome, documento, matricula, jornada_nome): (
        String,
        Option<i64>,
        String,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT f.nome,
                    f.empresa_id,
                    COALESCE(e.nome, ''),
                    COALESCE(f.documento, ''),
                    COALESCE(f.matricula, ''),
                    COALESCE(jt.descricao, '')
             FROM funcionarios f
             LEFT JOIN empresas e ON e.id = f.empresa_id
             LEFT JOIN jornadas_trabalho jt ON jt.id = f.jornada_id
             WHERE f.id = ?1",
            [funcionario_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar funcionário do fechamento: {err}"))?
        .ok_or_else(|| "Funcionário não encontrado para o fechamento.".to_string())?;

    let total_banco_horas: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(minutos), 0)
             FROM banco_horas_lancamentos
             WHERE funcionario_id = ?1 AND data_referencia >= ?2 AND data_referencia <= ?3",
            params![
                funcionario_id,
                inicio.format("%Y-%m-%d").to_string(),
                fim.format("%Y-%m-%d").to_string()
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao totalizar banco de horas do fechamento: {err}"))?
        .unwrap_or(0);

    let mut linhas = String::new();
    for row in &apuracao.rows {
        let batidas = if row.batidas.is_empty() {
            "-".to_string()
        } else {
            row.batidas.join(" | ")
        };
        let ocorrencias = if row.ocorrencias.is_empty() {
            "-".to_string()
        } else {
            row.ocorrencias.join(" | ")
        };
        let mensagens = if row.mensagens.is_empty() {
            "-".to_string()
        } else {
            row.mensagens.join(" | ")
        };

        linhas.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            escape_html(&row.data),
            escape_html(&batidas),
            escape_html(&row.jornada_nome),
            format_minutes(row.horario_esperado_minutos),
            format_minutes(row.trabalhado_minutos),
            format_minutes(row.saldo_minutos),
            format_minutes(row.atraso_minutos),
            format_minutes(row.extra_minutos),
            escape_html(&ocorrencias),
            escape_html(&mensagens),
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
        <html lang="pt-BR">
        <head>
          <meta charset="utf-8" />
          <title>Fechamento de ponto {mes:02}/{ano} - {funcionario}</title>
          <style>
            body {{ font-family: Arial, sans-serif; margin: 24px; color: #111827; }}
            h1, h2, h3 {{ margin: 0 0 8px; }}
            .meta {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; margin-bottom: 16px; }}
            .box {{ border: 1px solid #d1d5db; border-radius: 8px; padding: 10px 12px; }}
            table {{ width: 100%; border-collapse: collapse; margin-top: 12px; font-size: 12px; }}
            th, td {{ border: 1px solid #d1d5db; padding: 6px 8px; vertical-align: top; }}
            th {{ background: #f3f4f6; text-align: left; }}
            .totais {{ margin-top: 16px; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }}
            .assinaturas {{ margin-top: 42px; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 36px; }}
            .assinatura-linha {{ margin-top: 56px; border-top: 1px solid #111827; padding-top: 8px; text-align: center; }}
            .rodape {{ margin-top: 20px; font-size: 11px; color: #6b7280; }}
          </style>
        </head>
        <body>
          <h1>Espelho mensal de ponto para fechamento</h1>
          <h2>{funcionario}</h2>
          <div class="meta">
            <div class="box"><strong>Empresa:</strong> {empresa}<br /><strong>Período:</strong> {inicio} a {fim}<br /><strong>Jornada principal:</strong> {jornada}</div>
            <div class="box"><strong>Matrícula:</strong> {matricula}<br /><strong>Documento:</strong> {documento}<br /><strong>Gerado em:</strong> {gerado}</div>
          </div>

          <table>
            <thead>
              <tr>
                <th>Data</th>
                <th>Batidas</th>
                <th>Jornada</th>
                <th>Previsto</th>
                <th>Computado</th>
                <th>Saldo</th>
                <th>Atraso</th>
                <th>Extra</th>
                <th>Ocorrências</th>
                <th>Observações</th>
              </tr>
            </thead>
            <tbody>
              {linhas}
            </tbody>
          </table>

          <div class="totais">
            <div class="box"><strong>Total previsto:</strong> {previsto}<br /><strong>Total computado:</strong> {computado}</div>
            <div class="box"><strong>Saldo do período:</strong> {saldo}<br /><strong>Atrasos:</strong> {atraso}<br /><strong>Extras:</strong> {extra}</div>
            <div class="box"><strong>Banco de horas no período:</strong> {banco}<br /><strong>Dias apurados:</strong> {dias}</div>
          </div>

          <div class="assinaturas">
            <div class="assinatura-linha">Assinatura do colaborador</div>
            <div class="assinatura-linha">Assinatura do empregador / responsável</div>
          </div>

          <div class="rodape">
            Relatório interno de fechamento mensal para conferência e assinatura. O tratamento do período considera batidas importadas/manuais, justificativas, atestados, abonos, ajustes autorizados e saldo de banco de horas processado no sistema.
          </div>
        </body>
        </html>"#,
        mes = mes,
        ano = ano,
        funcionario = escape_html(&funcionario_nome),
        empresa = escape_html(&empresa_nome),
        inicio = inicio.format("%d/%m/%Y"),
        fim = fim.format("%d/%m/%Y"),
        jornada = escape_html(&jornada_nome),
        matricula = escape_html(&matricula),
        documento = escape_html(&documento),
        gerado = escape_html(&chrono::Local::now().format("%d/%m/%Y %H:%M").to_string()),
        linhas = linhas,
        previsto = format_minutes(apuracao.total_esperado_minutos),
        computado = format_minutes(apuracao.total_trabalhado_minutos),
        saldo = format_minutes(apuracao.total_saldo_minutos),
        atraso = format_minutes(apuracao.total_atraso_minutos),
        extra = format_minutes(apuracao.total_extra_minutos),
        banco = format_minutes(total_banco_horas),
        dias = apuracao.total_dias,
    );

    let export_dir = db_path
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "Diretório do banco inválido para exportação.".to_string())?
        .join("exports")
        .join("fechamentos");
    fs::create_dir_all(&export_dir)
        .map_err(|err| format!("Falha ao criar diretório de fechamentos: {err}"))?;

    let file_name = format!(
        "fechamento_{:04}_{:02}_{}_{}.html",
        ano,
        mes,
        funcionario_id,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file_path = export_dir.join(file_name);
    fs::write(&file_path, html)
        .map_err(|err| format!("Falha ao gravar relatório de fechamento: {err}"))?;

    conn.execute(
        "INSERT INTO fechamentos_mensais (
            funcionario_id, empresa_id, ano, mes, data_inicial, data_final,
            total_esperado_minutos, total_trabalhado_minutos, total_saldo_minutos,
            total_atraso_minutos, total_extra_minutos, total_banco_horas_minutos,
            resumo_json, relatorio_path, status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'fechado', ?15, ?15)
         ON CONFLICT(funcionario_id, ano, mes)
         DO UPDATE SET
            empresa_id = excluded.empresa_id,
            data_inicial = excluded.data_inicial,
            data_final = excluded.data_final,
            total_esperado_minutos = excluded.total_esperado_minutos,
            total_trabalhado_minutos = excluded.total_trabalhado_minutos,
            total_saldo_minutos = excluded.total_saldo_minutos,
            total_atraso_minutos = excluded.total_atraso_minutos,
            total_extra_minutos = excluded.total_extra_minutos,
            total_banco_horas_minutos = excluded.total_banco_horas_minutos,
            resumo_json = excluded.resumo_json,
            relatorio_path = excluded.relatorio_path,
            status = excluded.status,
            updated_at = excluded.updated_at",
        params![
            funcionario_id,
            empresa_id,
            ano,
            mes,
            inicio.format("%Y-%m-%d").to_string(),
            fim.format("%Y-%m-%d").to_string(),
            apuracao.total_esperado_minutos,
            apuracao.total_trabalhado_minutos,
            apuracao.total_saldo_minutos,
            apuracao.total_atraso_minutos,
            apuracao.total_extra_minutos,
            total_banco_horas,
            serde_json::to_string(&apuracao)
                .map_err(|err| format!("Falha ao serializar resumo do fechamento: {err}"))?,
            file_path.to_string_lossy().to_string(),
            now,
        ],
    )
    .map_err(|err| format!("Falha ao gravar fechamento mensal: {err}"))?;

    let fechamento_id: i64 = conn
        .query_row(
            "SELECT id FROM fechamentos_mensais WHERE funcionario_id = ?1 AND ano = ?2 AND mes = ?3 LIMIT 1",
            params![funcionario_id, ano, mes],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao reler fechamento mensal: {err}"))?;

    let result = conn
        .query_row(
            "SELECT fm.id,
                    fm.funcionario_id,
                    f.nome AS funcionario_nome,
                    fm.empresa_id,
                    COALESCE(e.nome, '') AS empresa_nome,
                    fm.ano,
                    fm.mes,
                    fm.data_inicial,
                    fm.data_final,
                    fm.total_esperado_minutos,
                    fm.total_trabalhado_minutos,
                    fm.total_saldo_minutos,
                    fm.total_atraso_minutos,
                    fm.total_extra_minutos,
                    fm.total_banco_horas_minutos,
                    fm.relatorio_path,
                    fm.status,
                    fm.created_at,
                    fm.updated_at
             FROM fechamentos_mensais fm
             INNER JOIN funcionarios f ON f.id = fm.funcionario_id
             LEFT JOIN empresas e ON e.id = fm.empresa_id
             WHERE fm.id = ?1",
            [fechamento_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao montar retorno do fechamento: {err}"))?
        .ok_or_else(|| "Fechamento gerado, mas não encontrado para retorno.".to_string())?;

    let payload_value = Value::Object(result.clone());
    write_audit(
        &conn,
        "fechamentos_mensais",
        "generate",
        Some(fechamento_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "fechamentos_mensais",
        "generate",
        Some(fechamento_id),
        &payload_value,
    )?;

    Ok(result)
}

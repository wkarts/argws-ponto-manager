use rusqlite::{params, OptionalExtension};
use serde_json::{Map, Value};
use tauri::State;

use crate::{app_state::SharedState, db::open_connection};

fn brand_header(brand: &str) -> &'static str {
    match brand {
        "henry" => "HENRY",
        "evo" => "EVO",
        "blue" => "BLUE",
        "dimep" => "DIMEP",
        _ => "REPGEN",
    }
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn export_company_line(brand: &str, row: &Map<String, Value>) -> String {
    let id = row.get("id").and_then(Value::as_i64).unwrap_or_default();
    let nome = row.get("nome").and_then(Value::as_str).unwrap_or("");
    let documento = row.get("documento").and_then(Value::as_str).unwrap_or("");
    let cidade = row.get("cidade").and_then(Value::as_str).unwrap_or("");
    let estado = row.get("estado").and_then(Value::as_str).unwrap_or("");
    let prefix = brand_header(brand);
    format!(
        "{prefix}|EMPRESA|{id}|{}|{}|{}|{}",
        only_digits(documento),
        nome.replace('|', " "),
        cidade.replace('|', " "),
        estado.replace('|', " "),
    )
}

fn export_employee_line(brand: &str, row: &Map<String, Value>) -> String {
    let prefix = brand_header(brand);
    let empresa_id = row.get("empresa_id").and_then(Value::as_i64).unwrap_or_default();
    let matricula = row.get("matricula").and_then(Value::as_str).unwrap_or("");
    let nome = row.get("nome").and_then(Value::as_str).unwrap_or("");
    let cpf = row.get("documento").and_then(Value::as_str).unwrap_or("");
    let pis = row.get("pis").and_then(Value::as_str).unwrap_or("");
    match brand {
        "henry" => format!(
            "{prefix}|FUNC|{empresa_id}|{}|{}|{}|{}",
            matricula,
            nome.replace('|', " "),
            only_digits(cpf),
            only_digits(pis)
        ),
        "evo" => format!(
            "{prefix};COLAB;{empresa_id};{};{};{};{}",
            matricula,
            nome.replace(';', " "),
            only_digits(cpf),
            only_digits(pis)
        ),
        "blue" => format!(
            "{prefix},{empresa_id},{},{},{},{}",
            matricula,
            nome.replace(',', " "),
            only_digits(cpf),
            only_digits(pis)
        ),
        "dimep" => format!(
            "{prefix}\tFUNC\t{empresa_id}\t{}\t{}\t{}\t{}",
            matricula,
            nome.replace('\t', " "),
            only_digits(cpf),
            only_digits(pis)
        ),
        _ => format!(
            "{prefix}|FUNC|{empresa_id}|{}|{}|{}|{}",
            matricula,
            nome.replace('|', " "),
            only_digits(cpf),
            only_digits(pis)
        ),
    }
}

#[tauri::command]
pub fn rep_export_empresa_txt(
    state: State<'_, SharedState>,
    brand: String,
    empresa_id: i64,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let record = conn
        .query_row(
            "SELECT id, nome, documento, cidade, estado FROM empresas WHERE id = ?1 LIMIT 1",
            [empresa_id],
            crate::db::row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar empresa para exportação REP: {err}"))?
        .ok_or_else(|| "Empresa não encontrada para exportação REP.".to_string())?;

    let content = export_company_line(&brand.to_lowercase(), &record);
    let mut response = Map::new();
    response.insert("brand".to_string(), Value::from(brand.to_lowercase()));
    response.insert("empresa_id".to_string(), Value::from(empresa_id));
    response.insert("file_name".to_string(), Value::from(format!("rep_empresa_{}_{}.txt", brand.to_lowercase(), empresa_id)));
    response.insert("content".to_string(), Value::from(content));
    Ok(response)
}

#[tauri::command]
pub fn rep_export_funcionarios_txt(
    state: State<'_, SharedState>,
    brand: String,
    empresa_id: i64,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let empresa_nome: String = conn
        .query_row("SELECT nome FROM empresas WHERE id = ?1 LIMIT 1", [empresa_id], |row| row.get(0))
        .optional()
        .map_err(|err| format!("Falha ao consultar empresa do REP: {err}"))?
        .ok_or_else(|| "Empresa não encontrada para exportação REP.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, empresa_id, matricula, nome, documento, pis
             FROM funcionarios
             WHERE empresa_id = ?1 AND COALESCE(ativo, 1) = 1
             ORDER BY nome ASC",
        )
        .map_err(|err| format!("Falha ao preparar funcionários para exportação REP: {err}"))?;

    let rows = stmt
        .query_map(params![empresa_id], crate::db::row_to_json_map)
        .map_err(|err| format!("Falha ao consultar funcionários para exportação REP: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear funcionários para exportação REP: {err}"))?;

    let mut lines = vec![format!("{}|EMPRESA|{}", brand_header(&brand.to_lowercase()), empresa_nome.replace('|', " "))];
    for row in &rows {
        lines.push(export_employee_line(&brand.to_lowercase(), row));
    }

    let mut response = Map::new();
    response.insert("brand".to_string(), Value::from(brand.to_lowercase()));
    response.insert("empresa_id".to_string(), Value::from(empresa_id));
    response.insert("total".to_string(), Value::from(rows.len() as i64));
    response.insert(
        "file_name".to_string(),
        Value::from(format!("rep_funcionarios_{}_{}.txt", brand.to_lowercase(), empresa_id)),
    );
    response.insert("content".to_string(), Value::from(lines.join("\r\n")));
    Ok(response)
}

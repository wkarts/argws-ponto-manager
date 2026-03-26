use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
};

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

fn get_id(payload: &Map<String, Value>) -> Option<i64> {
    payload.get("id").and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn normalize_upper_uf(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.to_uppercase())
        .filter(|text| !text.is_empty())
}

fn validate_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.contains('@') && email.contains('.')
}

fn is_valid_cpf(value: &str) -> bool {
    let digits = only_digits(value);
    if digits.len() != 11 {
        return false;
    }

    let bytes = digits.as_bytes();
    if bytes.iter().all(|b| *b == bytes[0]) {
        return false;
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    let sum1: u32 = nums
        .iter()
        .take(9)
        .enumerate()
        .map(|(idx, n)| n * (10 - idx as u32))
        .sum();
    let rem1 = sum1 % 11;
    let dv1 = if rem1 < 2 { 0 } else { 11 - rem1 };
    if nums[9] != dv1 {
        return false;
    }

    let sum2: u32 = nums
        .iter()
        .take(10)
        .enumerate()
        .map(|(idx, n)| n * (11 - idx as u32))
        .sum();
    let rem2 = sum2 % 11;
    let dv2 = if rem2 < 2 { 0 } else { 11 - rem2 };
    nums[10] == dv2
}

fn is_valid_cnpj(value: &str) -> bool {
    let digits = only_digits(value);
    if digits.len() != 14 {
        return false;
    }

    let bytes = digits.as_bytes();
    if bytes.iter().all(|b| *b == bytes[0]) {
        return false;
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];

    let sum1: u32 = nums
        .iter()
        .take(12)
        .zip(weights1.iter())
        .map(|(n, w)| n * w)
        .sum();
    let rem1 = sum1 % 11;
    let dv1 = if rem1 < 2 { 0 } else { 11 - rem1 };
    if nums[12] != dv1 {
        return false;
    }

    let sum2: u32 = nums
        .iter()
        .take(13)
        .zip(weights2.iter())
        .map(|(n, w)| n * w)
        .sum();
    let rem2 = sum2 % 11;
    let dv2 = if rem2 < 2 { 0 } else { 11 - rem2 };
    nums[13] == dv2
}

fn validate_company_document(document: &str) -> bool {
    let digits = only_digits(document);
    match digits.len() {
        11 => is_valid_cpf(&digits),
        14 => is_valid_cnpj(&digits),
        _ => false,
    }
}

fn company_select_sql() -> &'static str {
    "SELECT id, nome, nome_fantasia, documento, inscricao_estadual, inscricao_municipal, telefone, email,
            responsavel_nome, responsavel_telefone, cep, endereco, numero, complemento, bairro, cidade,
            estado, observacoes, ativo, created_at, updated_at
     FROM empresas"
}

#[tauri::command]
pub fn company_list(
    state: State<'_, SharedState>,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let search = get_string(&filters, "search").unwrap_or_default();
    let only_active = get_bool(&filters, "onlyActive", false) == 1;

    let mut sql = format!("{} WHERE 1=1", company_select_sql());
    let mut values: Vec<rusqlite::types::Value> = Vec::new();

    if !search.is_empty() {
        sql.push_str(
            " AND (nome LIKE ? OR nome_fantasia LIKE ? OR documento LIKE ? OR cidade LIKE ?)",
        );
        let wildcard = format!("%{}%", search);
        let doc_wildcard = format!("%{}%", only_digits(&search));
        values.push(rusqlite::types::Value::Text(wildcard.clone()));
        values.push(rusqlite::types::Value::Text(wildcard.clone()));
        values.push(rusqlite::types::Value::Text(
            if only_digits(&search).is_empty() {
                wildcard.clone()
            } else {
                doc_wildcard
            },
        ));
        values.push(rusqlite::types::Value::Text(wildcard));
    }

    if only_active {
        sql.push_str(" AND ativo = 1");
    }

    sql.push_str(" ORDER BY nome ASC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de empresas: {err}"))?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(values.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de empresas: {err}"))?;

    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear empresas: {err}"))
}

#[tauri::command]
pub fn company_get(state: State<'_, SharedState>, id: i64) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let sql = format!("{} WHERE id = ?1", company_select_sql());

    conn.query_row(&sql, [id], row_to_json_map)
        .optional()
        .map_err(|err| format!("Falha ao consultar empresa: {err}"))?
        .ok_or_else(|| "Empresa não encontrada.".to_string())
}

#[tauri::command]
pub fn company_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let id = get_id(&payload);
    let now = Utc::now().to_rfc3339();

    let nome = get_string(&payload, "nome")
        .ok_or_else(|| "Informe a razão social da empresa.".to_string())?;
    let documento_raw = get_string(&payload, "documento")
        .ok_or_else(|| "Informe o CNPJ/CPF da empresa usuária.".to_string())?;
    let documento = only_digits(&documento_raw);

    if !validate_company_document(&documento) {
        return Err("Documento inválido para empresa usuária.".to_string());
    }

    let email = get_string(&payload, "email");
    if let Some(value) = email.as_deref() {
        if !validate_email(value) {
            return Err("E-mail da empresa é inválido.".to_string());
        }
    }

    let estado = normalize_upper_uf(get_string(&payload, "estado"));
    if let Some(uf) = &estado {
        if uf.len() != 2 {
            return Err("UF da empresa deve conter 2 caracteres.".to_string());
        }
    }

    let duplicate_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM empresas WHERE documento = ?1 AND (?2 IS NULL OR id <> ?2) LIMIT 1",
            params![documento, id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar documento da empresa: {err}"))?;

    if duplicate_id.is_some() {
        return Err("Já existe uma empresa usuária com este documento.".to_string());
    }

    let nome_fantasia = get_string(&payload, "nome_fantasia");
    let inscricao_estadual =
        get_string(&payload, "inscricao_estadual").map(|value| only_digits(&value));
    let inscricao_municipal = get_string(&payload, "inscricao_municipal");
    let telefone = get_string(&payload, "telefone").map(|value| only_digits(&value));
    let responsavel_nome = get_string(&payload, "responsavel_nome");
    let responsavel_telefone =
        get_string(&payload, "responsavel_telefone").map(|value| only_digits(&value));
    let cep = get_string(&payload, "cep").map(|value| only_digits(&value));
    let endereco = get_string(&payload, "endereco");
    let numero = get_string(&payload, "numero");
    let complemento = get_string(&payload, "complemento");
    let bairro = get_string(&payload, "bairro");
    let cidade = get_string(&payload, "cidade");
    let observacoes = get_string(&payload, "observacoes");
    let ativo = get_bool(&payload, "ativo", true);

    let record_id = if let Some(existing_id) = id {
        conn.execute(
            "UPDATE empresas
             SET nome = ?1,
                 nome_fantasia = ?2,
                 documento = ?3,
                 inscricao_estadual = ?4,
                 inscricao_municipal = ?5,
                 telefone = ?6,
                 email = ?7,
                 responsavel_nome = ?8,
                 responsavel_telefone = ?9,
                 cep = ?10,
                 endereco = ?11,
                 numero = ?12,
                 complemento = ?13,
                 bairro = ?14,
                 cidade = ?15,
                 estado = ?16,
                 observacoes = ?17,
                 ativo = ?18,
                 updated_at = ?19
             WHERE id = ?20",
            params![
                nome,
                nome_fantasia,
                documento,
                inscricao_estadual,
                inscricao_municipal,
                telefone,
                email,
                responsavel_nome,
                responsavel_telefone,
                cep,
                endereco,
                numero,
                complemento,
                bairro,
                cidade,
                estado,
                observacoes,
                ativo,
                now,
                existing_id,
            ],
        )
        .map_err(|err| format!("Falha ao atualizar empresa: {err}"))?;
        existing_id
    } else {
        conn.execute(
            "INSERT INTO empresas (
                nome, nome_fantasia, documento, inscricao_estadual, inscricao_municipal, telefone, email,
                responsavel_nome, responsavel_telefone, cep, endereco, numero, complemento, bairro,
                cidade, estado, observacoes, ativo, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?19)",
            params![
                nome,
                nome_fantasia,
                documento,
                inscricao_estadual,
                inscricao_municipal,
                telefone,
                email,
                responsavel_nome,
                responsavel_telefone,
                cep,
                endereco,
                numero,
                complemento,
                bairro,
                cidade,
                estado,
                observacoes,
                ativo,
                now,
            ],
        )
        .map_err(|err| format!("Falha ao inserir empresa: {err}"))?;
        conn.last_insert_rowid()
    };

    let saved = conn
        .query_row(
            &format!("{} WHERE id = ?1", company_select_sql()),
            [record_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler empresa salva: {err}"))?
        .ok_or_else(|| "Empresa salva não encontrada.".to_string())?;

    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "empresas",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "empresas",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn company_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let employee_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM funcionarios WHERE empresa_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar vínculos da empresa: {err}"))?;

    if employee_count > 0 {
        return Err(
            "Não é possível excluir a empresa porque existem funcionários vinculados.".to_string(),
        );
    }

    let equipment_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM equipamentos WHERE empresa_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar equipamentos da empresa: {err}"))?;

    if equipment_count > 0 {
        return Err(
            "Não é possível excluir a empresa porque existem equipamentos vinculados.".to_string(),
        );
    }

    let affected = conn
        .execute("DELETE FROM empresas WHERE id = ?1", [id])
        .map_err(|err| format!("Falha ao excluir empresa: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id, "entity": "empresas" });
        write_audit(&conn, "empresas", "delete", Some(id), &payload)?;
        enqueue_sync(&conn, "empresas", "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

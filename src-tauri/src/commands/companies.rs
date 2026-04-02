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

fn get_setting_url(
    conn: &rusqlite::Connection,
    key: &str,
    default_value: &str,
) -> Result<String, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT valor FROM app_settings WHERE chave = ?1 LIMIT 1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar configuração {key}: {err}"))?;

    Ok(raw
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_value.to_string()))
}

fn optional_json_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|item| match item {
        Value::String(text) => Some(text.trim().to_string()).filter(|text| !text.is_empty()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

fn optional_json_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(|item| match item {
        Value::Bool(flag) => Some(*flag),
        Value::Number(number) => Some(number.as_i64().unwrap_or_default() != 0),
        Value::String(text) => Some(matches!(
            text.trim(),
            "1" | "true" | "TRUE" | "sim" | "SIM" | "ativo" | "ATIVO"
        )),
        _ => None,
    })
}

fn parse_publica_ws_response(
    body: &str,
    uf_hint: Option<&str>,
) -> Result<Map<String, Value>, String> {
    let parsed: Value = serde_json::from_str(body)
        .map_err(|err| format!("Falha ao interpretar resposta do CNPJ.ws: {err}"))?;

    let root = parsed
        .as_object()
        .ok_or_else(|| "Resposta inválida do CNPJ.ws.".to_string())?;
    let estabelecimento = root
        .get("estabelecimento")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "Resposta do CNPJ.ws sem estabelecimento.".to_string())?;

    let estado_obj = estabelecimento
        .get("estado")
        .and_then(|value| value.as_object());
    let cidade_obj = estabelecimento
        .get("cidade")
        .and_then(|value| value.as_object());
    let current_uf = uf_hint
        .map(|value| value.trim().to_uppercase())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            optional_json_string(estado_obj.and_then(|value| value.get("sigla")))
                .map(|value| value.to_uppercase())
        });

    let mut inscricao_estadual = None;
    if let Some(list) = estabelecimento
        .get("inscricoes_estaduais")
        .and_then(|value| value.as_array())
    {
        if let Some(target_uf) = current_uf.as_deref() {
            for item in list {
                let state_sigla = item
                    .get("estado")
                    .and_then(|value| value.get("sigla"))
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_uppercase())
                    .unwrap_or_default();
                let ativo = optional_json_bool(item.get("ativo"));
                let ie = optional_json_string(item.get("inscricao_estadual"));
                if state_sigla == target_uf && ativo.unwrap_or(true) {
                    inscricao_estadual = ie;
                    break;
                }
            }
        }
        if inscricao_estadual.is_none() {
            inscricao_estadual = list
                .iter()
                .find_map(|item| optional_json_string(item.get("inscricao_estadual")));
        }
    }

    let mut result = Map::new();
    result.insert(
        "nome".to_string(),
        Value::String(optional_json_string(root.get("razao_social")).unwrap_or_default()),
    );
    result.insert(
        "nome_fantasia".to_string(),
        Value::String(
            optional_json_string(estabelecimento.get("nome_fantasia")).unwrap_or_default(),
        ),
    );
    result.insert(
        "documento".to_string(),
        Value::String(
            optional_json_string(root.get("cnpj_raiz")).unwrap_or_default()
                + &optional_json_string(root.get("cnpj_ordem")).unwrap_or_default()
                + &optional_json_string(root.get("cnpj_digito_verificador")).unwrap_or_default(),
        ),
    );
    result.insert(
        "inscricao_estadual".to_string(),
        Value::String(inscricao_estadual.unwrap_or_else(|| "ISENTO".to_string())),
    );
    result.insert(
        "telefone".to_string(),
        Value::String(
            format!(
                "{}{}",
                optional_json_string(estabelecimento.get("ddd1")).unwrap_or_default(),
                optional_json_string(estabelecimento.get("telefone1")).unwrap_or_default(),
            )
            .trim()
            .to_string(),
        ),
    );
    result.insert(
        "email".to_string(),
        Value::String(optional_json_string(estabelecimento.get("email")).unwrap_or_default()),
    );
    result.insert(
        "cep".to_string(),
        Value::String(optional_json_string(estabelecimento.get("cep")).unwrap_or_default()),
    );
    result.insert(
        "endereco".to_string(),
        Value::String(optional_json_string(estabelecimento.get("logradouro")).unwrap_or_default()),
    );
    result.insert(
        "numero".to_string(),
        Value::String(optional_json_string(estabelecimento.get("numero")).unwrap_or_default()),
    );
    result.insert(
        "complemento".to_string(),
        Value::String(optional_json_string(estabelecimento.get("complemento")).unwrap_or_default()),
    );
    result.insert(
        "bairro".to_string(),
        Value::String(optional_json_string(estabelecimento.get("bairro")).unwrap_or_default()),
    );
    result.insert(
        "cidade".to_string(),
        Value::String(
            optional_json_string(cidade_obj.and_then(|value| value.get("nome")))
                .unwrap_or_default(),
        ),
    );
    result.insert(
        "estado".to_string(),
        Value::String(current_uf.unwrap_or_default()),
    );
    result.insert(
        "inscricao_municipal".to_string(),
        Value::String(
            optional_json_string(estabelecimento.get("inscricao_municipal")).unwrap_or_default(),
        ),
    );
    result.insert(
        "source".to_string(),
        Value::String("publica_cnpj_ws".to_string()),
    );
    Ok(result)
}

fn parse_receita_ws_response(body: &str) -> Result<Map<String, Value>, String> {
    let parsed: Value = serde_json::from_str(body)
        .map_err(|err| format!("Falha ao interpretar resposta do ReceitaWS: {err}"))?;

    let root = parsed
        .as_object()
        .ok_or_else(|| "Resposta inválida do ReceitaWS.".to_string())?;

    if let Some(status) = optional_json_string(root.get("status")) {
        if status.eq_ignore_ascii_case("ERROR") {
            return Err(optional_json_string(root.get("message"))
                .unwrap_or_else(|| "ReceitaWS retornou erro.".to_string()));
        }
    }

    let mut result = Map::new();
    result.insert(
        "nome".to_string(),
        Value::String(optional_json_string(root.get("nome")).unwrap_or_default()),
    );
    result.insert(
        "nome_fantasia".to_string(),
        Value::String(optional_json_string(root.get("fantasia")).unwrap_or_default()),
    );
    result.insert(
        "documento".to_string(),
        Value::String(optional_json_string(root.get("cnpj")).unwrap_or_default()),
    );
    result.insert(
        "inscricao_estadual".to_string(),
        Value::String(optional_json_string(root.get("ie")).unwrap_or_else(|| "ISENTO".to_string())),
    );
    result.insert(
        "telefone".to_string(),
        Value::String(optional_json_string(root.get("telefone")).unwrap_or_default()),
    );
    result.insert(
        "email".to_string(),
        Value::String(optional_json_string(root.get("email")).unwrap_or_default()),
    );
    result.insert(
        "cep".to_string(),
        Value::String(optional_json_string(root.get("cep")).unwrap_or_default()),
    );
    result.insert(
        "endereco".to_string(),
        Value::String(optional_json_string(root.get("logradouro")).unwrap_or_default()),
    );
    result.insert(
        "numero".to_string(),
        Value::String(optional_json_string(root.get("numero")).unwrap_or_default()),
    );
    result.insert(
        "complemento".to_string(),
        Value::String(optional_json_string(root.get("complemento")).unwrap_or_default()),
    );
    result.insert(
        "bairro".to_string(),
        Value::String(optional_json_string(root.get("bairro")).unwrap_or_default()),
    );
    result.insert(
        "cidade".to_string(),
        Value::String(optional_json_string(root.get("municipio")).unwrap_or_default()),
    );
    result.insert(
        "estado".to_string(),
        Value::String(
            optional_json_string(root.get("uf"))
                .unwrap_or_default()
                .to_uppercase(),
        ),
    );
    result.insert(
        "source".to_string(),
        Value::String("receita_ws".to_string()),
    );
    Ok(result)
}

async fn fetch_json_text(url: String) -> Result<String, String> {
    reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|err| format!("Falha ao consultar serviço público: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Serviço público retornou erro: {err}"))?
        .text()
        .await
        .map_err(|err| format!("Falha ao ler resposta do serviço público: {err}"))
}

#[tauri::command]
pub async fn company_lookup_cnpj(
    state: State<'_, SharedState>,
    documento: String,
    uf: Option<String>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let digits = only_digits(&documento);
    if digits.len() != 14 {
        return Err("Informe um CNPJ válido com 14 dígitos para consulta.".to_string());
    }

    let publica_base = get_setting_url(
        &conn,
        "company_lookup_publica_url",
        "https://publica.cnpj.ws/cnpj/",
    )?;
    let receita_base = get_setting_url(
        &conn,
        "company_lookup_receita_url",
        "https://www.receitaws.com.br/v1/cnpj/",
    )?;
    let uf_hint = uf
        .map(|value| value.trim().to_uppercase())
        .filter(|value| !value.is_empty());

    let publica_url = format!("{}/{}", publica_base.trim_end_matches('/'), digits);
    let publica_error = match fetch_json_text(publica_url)
        .await
        .and_then(|body| parse_publica_ws_response(&body, uf_hint.as_deref()))
    {
        Ok(result) => return Ok(result),
        Err(err) => err,
    };

    let receita_url = format!("{}/{}", receita_base.trim_end_matches('/'), digits);
    match fetch_json_text(receita_url)
        .await
        .and_then(|body| parse_receita_ws_response(&body))
    {
        Ok(result) => Ok(result),
        Err(err) => Err(format!(
            "Falha ao consultar CNPJ nos serviços públicos. Último erro: {}; erro anterior: {}",
            err, publica_error
        )),
    }
}

#[tauri::command]
pub async fn company_lookup_ie(
    state: State<'_, SharedState>,
    documento: String,
    uf: Option<String>,
) -> Result<Map<String, Value>, String> {
    let mut payload = company_lookup_cnpj(state, documento, uf.clone()).await?;
    if optional_json_string(payload.get("inscricao_estadual"))
        .unwrap_or_default()
        .is_empty()
    {
        payload.insert(
            "inscricao_estadual".to_string(),
            Value::String("ISENTO".to_string()),
        );
    }
    if let Some(value) = uf.filter(|value| !value.trim().is_empty()) {
        payload.insert(
            "estado".to_string(),
            Value::String(value.trim().to_uppercase()),
        );
    }
    Ok(payload)
}

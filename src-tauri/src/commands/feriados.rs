use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
};

fn parse_i64(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(number)) => number.as_i64(),
        Some(Value::String(text)) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn parse_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

fn parse_bool(value: Option<&Value>, default: bool) -> i64 {
    match value {
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

fn parse_i64_vec(value: Option<&Value>) -> Vec<i64> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Value::Number(number) => number.as_i64(),
                Value::String(text) => text.trim().parse::<i64>().ok(),
                _ => None,
            })
            .collect(),
        Some(Value::String(text)) => text
            .split(',')
            .filter_map(|item| item.trim().parse::<i64>().ok())
            .collect(),
        _ => Vec::new(),
    }
}

fn validate_iso_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

fn dedup_sorted_ids(ids: Vec<i64>) -> Vec<i64> {
    let mut set = BTreeSet::new();
    for id in ids {
        if id > 0 {
            set.insert(id);
        }
    }
    set.into_iter().collect()
}

fn merge_primary_with_relation(primary: Option<i64>, relation_ids: &[i64]) -> Vec<i64> {
    let mut combined = relation_ids.to_vec();
    if let Some(id) = primary {
        combined.push(id);
    }
    dedup_sorted_ids(combined)
}

fn first_or_none(ids: &[i64]) -> Option<i64> {
    ids.first().copied()
}

fn load_relation_ids(
    conn: &rusqlite::Connection,
    table: &str,
    foreign_key: &str,
    feriado_id: i64,
) -> Result<Vec<i64>, String> {
    let sql = format!(
        "SELECT {foreign_key} FROM {table} WHERE feriado_id = ?1 ORDER BY {foreign_key} ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar leitura de {table}: {err}"))?;
    let rows = stmt
        .query_map([feriado_id], |row| row.get::<_, i64>(0))
        .map_err(|err| format!("Falha ao executar leitura de {table}: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear leitura de {table}: {err}"))
}

fn sync_relation_ids(
    conn: &rusqlite::Connection,
    table: &str,
    foreign_key: &str,
    feriado_id: i64,
    ids: &[i64],
    now: &str,
) -> Result<(), String> {
    let delete_sql = format!("DELETE FROM {table} WHERE feriado_id = ?1");
    conn.execute(&delete_sql, [feriado_id])
        .map_err(|err| format!("Falha ao limpar relacionamentos de {table}: {err}"))?;

    let insert_sql =
        format!("INSERT INTO {table} (feriado_id, {foreign_key}, created_at) VALUES (?1, ?2, ?3)");
    for id in ids {
        conn.execute(&insert_sql, params![feriado_id, id, now])
            .map_err(|err| format!("Falha ao gravar relacionamento em {table}: {err}"))?;
    }

    Ok(())
}

fn feriado_payload_by_id(
    conn: &rusqlite::Connection,
    feriado_id: i64,
) -> Result<Map<String, Value>, String> {
    let mut record = conn
        .query_row(
            "SELECT id,
                    data,
                    descricao,
                    contexto_tipo,
                    empresa_id,
                    departamento_id,
                    regra_jornada,
                    regra_compensacao,
                    observacoes,
                    ativo,
                    created_at,
                    updated_at
             FROM feriados
             WHERE id = ?1 LIMIT 1",
            [feriado_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler feriado salvo: {err}"))?
        .ok_or_else(|| "Feriado não encontrado após gravação.".to_string())?;

    let primary_empresa_id = record.get("empresa_id").and_then(Value::as_i64);
    let primary_departamento_id = record.get("departamento_id").and_then(Value::as_i64);

    let empresa_ids = merge_primary_with_relation(
        primary_empresa_id,
        &load_relation_ids(conn, "feriados_empresas", "empresa_id", feriado_id)?,
    );
    let departamento_ids = merge_primary_with_relation(
        primary_departamento_id,
        &load_relation_ids(
            conn,
            "feriados_departamentos",
            "departamento_id",
            feriado_id,
        )?,
    );

    let empresas_labels = {
        let mut stmt = conn
            .prepare(
                "SELECT e.nome
                 FROM empresas e
                 WHERE e.id = (SELECT empresa_id FROM feriados WHERE id = ?1)
                    OR EXISTS (
                        SELECT 1
                        FROM feriados_empresas fe
                        WHERE fe.feriado_id = ?1
                          AND fe.empresa_id = e.id
                    )
                 ORDER BY e.nome ASC",
            )
            .map_err(|err| format!("Falha ao preparar empresas do feriado: {err}"))?;
        let rows = stmt
            .query_map([feriado_id], |row| row.get::<_, String>(0))
            .map_err(|err| format!("Falha ao executar empresas do feriado: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("Falha ao mapear empresas do feriado: {err}"))?
    };

    let departamentos_labels = {
        let mut stmt = conn
            .prepare(
                "SELECT d.descricao
                 FROM departamentos d
                 WHERE d.id = (SELECT departamento_id FROM feriados WHERE id = ?1)
                    OR EXISTS (
                        SELECT 1
                        FROM feriados_departamentos fd
                        WHERE fd.feriado_id = ?1
                          AND fd.departamento_id = d.id
                    )
                 ORDER BY d.descricao ASC",
            )
            .map_err(|err| format!("Falha ao preparar departamentos do feriado: {err}"))?;
        let rows = stmt
            .query_map([feriado_id], |row| row.get::<_, String>(0))
            .map_err(|err| format!("Falha ao executar departamentos do feriado: {err}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("Falha ao mapear departamentos do feriado: {err}"))?
    };

    let empresa_ids_value = empresa_ids.iter().copied().map(Value::from).collect();
    let departamento_ids_value = departamento_ids.iter().copied().map(Value::from).collect();

    record.insert("empresa_ids".to_string(), Value::Array(empresa_ids_value));
    record.insert(
        "departamento_ids".to_string(),
        Value::Array(departamento_ids_value),
    );
    record.insert(
        "empresas_labels".to_string(),
        Value::Array(empresas_labels.iter().cloned().map(Value::from).collect()),
    );
    record.insert(
        "departamentos_labels".to_string(),
        Value::Array(
            departamentos_labels
                .iter()
                .cloned()
                .map(Value::from)
                .collect(),
        ),
    );
    record.insert(
        "empresa_nome".to_string(),
        Value::from(empresas_labels.first().cloned().unwrap_or_default()),
    );
    record.insert(
        "departamento_nome".to_string(),
        Value::from(departamentos_labels.first().cloned().unwrap_or_default()),
    );
    record.insert(
        "empresas_count".to_string(),
        Value::from(empresa_ids.len() as i64),
    );
    record.insert(
        "departamentos_count".to_string(),
        Value::from(departamento_ids.len() as i64),
    );

    Ok(record)
}

#[tauri::command]
pub fn feriado_list(
    state: State<'_, SharedState>,
    search: Option<String>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let mut sql = String::from(
        "SELECT f.id,
                f.data,
                f.descricao,
                f.contexto_tipo,
                f.empresa_id,
                f.departamento_id,
                f.regra_jornada,
                f.regra_compensacao,
                f.observacoes,
                f.ativo,
                COALESCE(e.nome, '') AS empresa_nome,
                COALESCE(d.descricao, '') AS departamento_nome,
                (SELECT COUNT(*) FROM feriados_empresas fe WHERE fe.feriado_id = f.id) AS empresas_count,
                (SELECT COUNT(*) FROM feriados_departamentos fd WHERE fd.feriado_id = f.id) AS departamentos_count,
                f.created_at,
                f.updated_at
         FROM feriados f
         LEFT JOIN empresas e ON e.id = f.empresa_id
         LEFT JOIN departamentos d ON d.id = f.departamento_id
         WHERE 1 = 1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(search) = search
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        sql.push_str(
            " AND (
                f.data LIKE ?1 OR
                f.descricao LIKE ?1 OR
                f.contexto_tipo LIKE ?1 OR
                COALESCE(e.nome, '') LIKE ?1 OR
                COALESCE(d.descricao, '') LIKE ?1
            )",
        );
        params_vec.push(rusqlite::types::Value::Text(format!("%{search}%")));
    }

    sql.push_str(" ORDER BY f.data DESC, f.descricao ASC, f.id DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de feriados: {err}"))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter()),
            row_to_json_map,
        )
        .map_err(|err| format!("Falha ao consultar listagem de feriados: {err}"))?;

    let base_rows = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear listagem de feriados: {err}"))?;

    let mut enriched = Vec::with_capacity(base_rows.len());
    for mut row in base_rows {
        let feriado_id = row.get("id").and_then(Value::as_i64).unwrap_or_default();
        let payload = feriado_payload_by_id(&conn, feriado_id)?;

        row.insert(
            "empresa_nome".to_string(),
            payload
                .get("empresa_nome")
                .cloned()
                .unwrap_or_else(|| Value::from("")),
        );
        row.insert(
            "departamento_nome".to_string(),
            payload
                .get("departamento_nome")
                .cloned()
                .unwrap_or_else(|| Value::from("")),
        );
        row.insert(
            "empresas_count".to_string(),
            payload
                .get("empresas_count")
                .cloned()
                .unwrap_or_else(|| Value::from(0)),
        );
        row.insert(
            "departamentos_count".to_string(),
            payload
                .get("departamentos_count")
                .cloned()
                .unwrap_or_else(|| Value::from(0)),
        );
        row.insert(
            "empresas_labels".to_string(),
            payload
                .get("empresas_labels")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
        );
        row.insert(
            "departamentos_labels".to_string(),
            payload
                .get("departamentos_labels")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
        );
        enriched.push(row);
    }

    Ok(enriched)
}

#[tauri::command]
pub fn feriado_get(state: State<'_, SharedState>, id: i64) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    feriado_payload_by_id(&conn, id)
}

#[tauri::command]
pub fn feriado_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let id = parse_i64(payload.get("id"));

    let data = parse_string(payload.get("data"))
        .ok_or_else(|| "Data do feriado é obrigatória.".to_string())?;
    let descricao = parse_string(payload.get("descricao"))
        .ok_or_else(|| "Descrição do feriado é obrigatória.".to_string())?;
    let contexto_tipo =
        parse_string(payload.get("contexto_tipo")).unwrap_or_else(|| "global".to_string());

    if !validate_iso_date(&data) {
        return Err("Data do feriado inválida. Utilize YYYY-MM-DD.".to_string());
    }

    let empresa_id_input = parse_i64(payload.get("empresa_id"));
    let departamento_id_input = parse_i64(payload.get("departamento_id"));
    let mut empresa_ids =
        merge_primary_with_relation(empresa_id_input, &parse_i64_vec(payload.get("empresa_ids")));
    let mut departamento_ids = merge_primary_with_relation(
        departamento_id_input,
        &parse_i64_vec(payload.get("departamento_ids")),
    );
    let regra_jornada = parse_string(payload.get("regra_jornada"));
    let regra_compensacao = parse_string(payload.get("regra_compensacao"));
    let observacoes = parse_string(payload.get("observacoes"));
    let ativo = parse_bool(payload.get("ativo"), true);

    let (empresa_id, departamento_id) = match contexto_tipo.as_str() {
        "global" => {
            empresa_ids.clear();
            departamento_ids.clear();
            (None, None)
        }
        "empresa" => {
            departamento_ids.clear();
            if empresa_ids.is_empty() {
                return Err("Informe ao menos uma empresa para o feriado por empresa.".to_string());
            }
            (first_or_none(&empresa_ids), None)
        }
        "departamento" => {
            empresa_ids.clear();
            if departamento_ids.is_empty() {
                return Err(
                    "Informe ao menos um departamento para o feriado por departamento.".to_string(),
                );
            }
            (None, first_or_none(&departamento_ids))
        }
        "operacional" => {
            if empresa_ids.is_empty() && departamento_ids.is_empty() {
                return Err(
                    "Informe ao menos empresa ou departamento para o feriado operacional."
                        .to_string(),
                );
            }
            (
                first_or_none(&empresa_ids),
                first_or_none(&departamento_ids),
            )
        }
        other => {
            return Err(format!("Contexto de feriado inválido: {other}"));
        }
    };

    let record_id = if let Some(existing_id) = id {
        conn.execute(
            "UPDATE feriados
             SET data = ?1,
                 descricao = ?2,
                 contexto_tipo = ?3,
                 empresa_id = ?4,
                 departamento_id = ?5,
                 regra_jornada = ?6,
                 regra_compensacao = ?7,
                 observacoes = ?8,
                 ativo = ?9,
                 updated_at = ?10
             WHERE id = ?11",
            params![
                data,
                descricao,
                contexto_tipo,
                empresa_id,
                departamento_id,
                regra_jornada,
                regra_compensacao,
                observacoes,
                ativo,
                now,
                existing_id,
            ],
        )
        .map_err(|err| format!("Falha ao atualizar feriado: {err}"))?;
        existing_id
    } else {
        conn.execute(
            "INSERT INTO feriados (
                data,
                descricao,
                contexto_tipo,
                empresa_id,
                departamento_id,
                regra_jornada,
                regra_compensacao,
                observacoes,
                ativo,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                data,
                descricao,
                contexto_tipo,
                empresa_id,
                departamento_id,
                regra_jornada,
                regra_compensacao,
                observacoes,
                ativo,
                now,
            ],
        )
        .map_err(|err| format!("Falha ao inserir feriado: {err}"))?;
        conn.last_insert_rowid()
    };

    sync_relation_ids(
        &conn,
        "feriados_empresas",
        "empresa_id",
        record_id,
        &empresa_ids,
        &now,
    )?;
    sync_relation_ids(
        &conn,
        "feriados_departamentos",
        "departamento_id",
        record_id,
        &departamento_ids,
        &now,
    )?;

    let saved = feriado_payload_by_id(&conn, record_id)?;
    let payload_value = Value::Object(saved.clone());
    let action_name = if id.is_some() { "update" } else { "create" };
    write_audit(
        &conn,
        "feriados",
        action_name,
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "feriados",
        action_name,
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn feriado_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let payload = feriado_payload_by_id(&conn, id).ok();
    let affected = conn
        .execute("DELETE FROM feriados WHERE id = ?1", [id])
        .map_err(|err| format!("Falha ao excluir feriado: {err}"))?;

    if affected > 0 {
        let value = payload
            .map(Value::Object)
            .unwrap_or_else(|| json!({ "id": id }));
        write_audit(&conn, "feriados", "delete", Some(id), &value)?;
        enqueue_sync(&conn, "feriados", "delete", Some(id), &value)?;
    }

    Ok(affected > 0)
}

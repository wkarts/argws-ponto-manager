use chrono::Utc;
use rusqlite::{params, params_from_iter, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::PunchFilters,
};

fn json_to_sql_value(value: &Value) -> rusqlite::types::Value {
    match value {
        Value::Null => rusqlite::types::Value::Null,
        Value::Bool(v) => rusqlite::types::Value::Integer(if *v { 1 } else { 0 }),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = v.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        Value::String(v) if v.trim().is_empty() => rusqlite::types::Value::Null,
        Value::String(v) => rusqlite::types::Value::Text(v.to_string()),
        _ => rusqlite::types::Value::Text(value.to_string()),
    }
}

fn parse_id(payload: &Map<String, Value>, field: &str) -> Option<i64> {
    payload.get(field).and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
    })
}

fn parse_bool(payload: &Map<String, Value>, field: &str, default: bool) -> i64 {
    match payload.get(field) {
        Some(Value::Bool(v)) => {
            if *v {
                1
            } else {
                0
            }
        }
        Some(Value::Number(v)) => {
            if v.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Some(Value::String(v)) => {
            if matches!(v.trim(), "1" | "true" | "TRUE" | "sim" | "SIM") {
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

#[tauri::command]
pub fn batidas_list(
    state: State<'_, SharedState>,
    filters: PunchFilters,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = String::from(
        "SELECT b.id,
                b.funcionario_id,
                f.nome AS funcionario_nome,
                b.equipamento_id,
                COALESCE(e.descricao, '') AS equipamento_nome,
                b.justificativa_id,
                COALESCE(j.descricao, '') AS justificativa_nome,
                b.manual_ajuste,
                b.validado,
                b.data_referencia,
                b.hora,
                b.nsr,
                b.origem,
                b.observacao,
                b.tipo
         FROM batidas b
         INNER JOIN funcionarios f ON f.id = b.funcionario_id
         LEFT JOIN equipamentos e ON e.id = b.equipamento_id
         LEFT JOIN justificativas j ON j.id = b.justificativa_id
         WHERE 1 = 1",
    );

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(funcionario_id) = filters.funcionario_id {
        sql.push_str(" AND b.funcionario_id = ?");
        params.push(rusqlite::types::Value::Integer(funcionario_id));
    }

    if let Some(data_inicial) = filters.data_inicial {
        sql.push_str(" AND b.data_referencia >= ?");
        params.push(rusqlite::types::Value::Text(data_inicial));
    }

    if let Some(data_final) = filters.data_final {
        sql.push_str(" AND b.data_referencia <= ?");
        params.push(rusqlite::types::Value::Text(data_final));
    }

    sql.push_str(" ORDER BY b.data_referencia DESC, b.hora DESC, b.id DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de batidas: {err}"))?;

    let mapped = stmt
        .query_map(params_from_iter(params.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de batidas: {err}"))?;

    let rows: Result<Vec<_>, _> = mapped.collect();
    rows.map_err(|err| format!("Falha ao mapear batidas: {err}"))
}

#[tauri::command]
pub fn batida_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();

    let funcionario_id = parse_id(&payload, "funcionario_id")
        .ok_or_else(|| "funcionario_id é obrigatório.".to_string())?;

    let data_referencia = payload
        .get("data_referencia")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "data_referencia é obrigatória.".to_string())?
        .to_string();

    let hora = payload
        .get("hora")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "hora é obrigatória.".to_string())?
        .to_string();

    let equipamento_id = parse_id(&payload, "equipamento_id");
    let justificativa_id = parse_id(&payload, "justificativa_id");
    let manual_ajuste = parse_bool(&payload, "manual_ajuste", false);
    let validado = parse_bool(&payload, "validado", true);
    let id = parse_id(&payload, "id");

    let origem_value = payload.get("origem").cloned().unwrap_or_else(|| {
        Value::String(if manual_ajuste == 1 {
            "ajuste_manual".to_string()
        } else {
            "manual".to_string()
        })
    });

    let values = vec![
        rusqlite::types::Value::Integer(funcionario_id),
        match equipamento_id {
            Some(v) => rusqlite::types::Value::Integer(v),
            None => rusqlite::types::Value::Null,
        },
        match justificativa_id {
            Some(v) => rusqlite::types::Value::Integer(v),
            None => rusqlite::types::Value::Null,
        },
        rusqlite::types::Value::Integer(manual_ajuste),
        rusqlite::types::Value::Integer(validado),
        rusqlite::types::Value::Text(data_referencia),
        rusqlite::types::Value::Text(hora),
        json_to_sql_value(payload.get("nsr").unwrap_or(&Value::Null)),
        json_to_sql_value(&origem_value),
        json_to_sql_value(payload.get("observacao").unwrap_or(&Value::Null)),
        json_to_sql_value(
            payload
                .get("tipo")
                .unwrap_or(&Value::String("entrada".to_string())),
        ),
    ];

    let record_id = if let Some(existing_id) = id {
        let mut params = values.clone();
        params.push(rusqlite::types::Value::Text(now.clone()));
        params.push(rusqlite::types::Value::Integer(existing_id));

        conn.execute(
            "UPDATE batidas
             SET funcionario_id = ?, equipamento_id = ?, justificativa_id = ?, manual_ajuste = ?, validado = ?, data_referencia = ?, hora = ?, nsr = ?, origem = ?, observacao = ?, tipo = ?, updated_at = ?
             WHERE id = ?",
            params_from_iter(params.iter()),
        )
        .map_err(|err| format!("Falha ao atualizar batida: {err}"))?;

        existing_id
    } else {
        let mut params = values.clone();
        params.push(rusqlite::types::Value::Text(now.clone()));
        params.push(rusqlite::types::Value::Text(now.clone()));

        conn.execute(
            "INSERT INTO batidas
             (funcionario_id, equipamento_id, justificativa_id, manual_ajuste, validado, data_referencia, hora, nsr, origem, observacao, tipo, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params_from_iter(params.iter()),
        )
        .map_err(|err| format!("Falha ao inserir batida: {err}"))?;

        conn.last_insert_rowid()
    };

    let saved = conn
        .query_row(
            "SELECT b.id,
                    b.funcionario_id,
                    f.nome AS funcionario_nome,
                    b.equipamento_id,
                    COALESCE(e.descricao, '') AS equipamento_nome,
                    b.justificativa_id,
                    COALESCE(j.descricao, '') AS justificativa_nome,
                    b.manual_ajuste,
                    b.validado,
                    b.data_referencia,
                    b.hora,
                    b.nsr,
                    b.origem,
                    b.observacao,
                    b.tipo
             FROM batidas b
             INNER JOIN funcionarios f ON f.id = b.funcionario_id
             LEFT JOIN equipamentos e ON e.id = b.equipamento_id
             LEFT JOIN justificativas j ON j.id = b.justificativa_id
             WHERE b.id = ?1",
            [record_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler batida salva: {err}"))?
        .ok_or_else(|| "Batida salva não encontrada.".to_string())?;

    let action_name = if id.is_some() { "update" } else { "create" };
    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "batidas",
        action_name,
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "batidas",
        action_name,
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn batida_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let affected = conn
        .execute("DELETE FROM batidas WHERE id = ?1", params![id])
        .map_err(|err| format!("Falha ao excluir batida: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id });
        write_audit(&conn, "batidas", "delete", Some(id), &payload)?;
        enqueue_sync(&conn, "batidas", "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

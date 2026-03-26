use rusqlite::{params, types::ValueRef, Connection};
use serde_json::{json, Map, Value};
use std::path::Path;

pub fn open_connection(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|err| format!("Falha ao abrir banco: {err}"))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|err| format!("Falha ao ativar foreign_keys: {err}"))?;
    Ok(conn)
}

pub fn row_to_json_map(row: &rusqlite::Row<'_>) -> rusqlite::Result<Map<String, Value>> {
    let row_ref = row.as_ref();
    let count = row_ref.column_count();
    let mut result = Map::new();

    for idx in 0..count {
        let key = row_ref
            .column_name(idx)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| format!("col_{idx}"));

        let value = match row.get_ref(idx)? {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(v) => Value::from(v),
            ValueRef::Real(v) => Value::from(v),
            ValueRef::Text(v) => Value::from(String::from_utf8_lossy(v).to_string()),
            ValueRef::Blob(v) => Value::from(format!("<blob:{} bytes>", v.len())),
        };

        result.insert(key, value);
    }

    Ok(result)
}

pub fn count_table(conn: &Connection, table: &str) -> Result<i64, String> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get::<_, i64>(0))
        .map_err(|err| format!("Falha ao contar tabela {table}: {err}"))
}

pub fn enqueue_sync(
    conn: &Connection,
    entity_name: &str,
    action_name: &str,
    record_id: Option<i64>,
    payload: &Value,
) -> Result<(), String> {
    let payload_json = payload.to_string();
    conn.execute(
        "INSERT INTO sync_queue (entity_name, action_name, record_id, payload_json, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', datetime('now'), datetime('now'))",
        params![entity_name, action_name, record_id, payload_json],
    )
    .map_err(|err| format!("Falha ao enfileirar sincronização: {err}"))?;
    Ok(())
}

pub fn write_audit(
    conn: &Connection,
    entity_name: &str,
    action_name: &str,
    record_id: Option<i64>,
    payload: &Value,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO audit_logs (entity_name, action_name, record_id, payload_json, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![entity_name, action_name, record_id, payload.to_string()],
    )
    .map_err(|err| format!("Falha ao gravar auditoria: {err}"))?;
    Ok(())
}

#[allow(dead_code)]
pub fn simple_change_payload(record_id: Option<i64>, description: &str) -> Value {
    json!({
        "record_id": record_id,
        "description": description,
    })
}

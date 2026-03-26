use tauri::State;

use crate::{app_state::SharedState, db::open_connection, models::SyncQueueItem};

#[tauri::command]
pub fn sync_queue_list(state: State<'_, SharedState>) -> Result<Vec<SyncQueueItem>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, entity_name, action_name, record_id, status, payload_json, created_at
             FROM sync_queue
             ORDER BY id DESC",
        )
        .map_err(|err| format!("Falha ao preparar listagem da fila de sincronização: {err}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SyncQueueItem {
                id: row.get(0)?,
                entity_name: row.get(1)?,
                action_name: row.get(2)?,
                record_id: row.get(3)?,
                status: row.get(4)?,
                payload_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|err| format!("Falha ao executar listagem da fila de sincronização: {err}"))?;

    let result: Result<Vec<_>, _> = rows.collect();
    result.map_err(|err| format!("Falha ao mapear fila de sincronização: {err}"))
}

#[tauri::command]
pub fn sync_queue_mark_synced(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let affected = conn
        .execute(
            "UPDATE sync_queue SET status = 'synced', updated_at = datetime('now') WHERE id = ?1",
            [id],
        )
        .map_err(|err| format!("Falha ao atualizar item da fila de sincronização: {err}"))?;

    Ok(affected > 0)
}

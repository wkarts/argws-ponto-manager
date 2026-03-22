use std::collections::BTreeMap;

use serde_json::Value;
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{count_table, open_connection},
};

#[tauri::command]
pub fn app_bootstrap(state: State<'_, SharedState>) -> Result<BTreeMap<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut payload = BTreeMap::new();
    payload.insert("db_path".to_string(), Value::String(db_path.to_string_lossy().to_string()));
    payload.insert("empresas".to_string(), Value::from(count_table(&conn, "empresas")?));
    payload.insert("usuarios".to_string(), Value::from(count_table(&conn, "usuarios")?));
    payload.insert("funcionarios".to_string(), Value::from(count_table(&conn, "funcionarios")?));
    payload.insert("equipamentos".to_string(), Value::from(count_table(&conn, "equipamentos")?));
    payload.insert("horarios".to_string(), Value::from(count_table(&conn, "horarios")?));
    payload.insert("batidas".to_string(), Value::from(count_table(&conn, "batidas")?));
    payload.insert("jornadas".to_string(), Value::from(count_table(&conn, "jornadas_trabalho")?));
    payload.insert("afd_importacoes".to_string(), Value::from(count_table(&conn, "afd_importacoes")?));
    payload.insert("banco_horas".to_string(), Value::from(count_table(&conn, "banco_horas_lancamentos")?));
    payload.insert(
        "sync_pendente".to_string(),
        Value::from(
            conn.query_row(
                "SELECT COUNT(*) FROM sync_queue WHERE status = 'pending'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| format!("Falha ao contar fila de sincronização: {err}"))?,
        ),
    );
    payload.insert(
        "carga_padrao_minutos".to_string(),
        Value::from(
            conn.query_row(
                "SELECT COALESCE(valor, '480') FROM configuracoes WHERE nome = 'carga_padrao_minutos' LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .map(|value| value.parse::<i64>().unwrap_or(480))
            .unwrap_or(480),
        ),
    );

    Ok(payload)
}

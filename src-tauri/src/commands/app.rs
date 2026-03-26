use std::{collections::BTreeMap, fs, path::PathBuf};

use serde_json::{json, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{count_table, open_connection},
};

fn build_hash() -> String {
    option_env!("BUILD_HASH")
        .or(option_env!("GITHUB_SHA"))
        .map(|value| value.chars().take(8).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "dev".to_string())
}

fn export_dir_for(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("exports")
}

#[tauri::command]
pub fn app_bootstrap(state: State<'_, SharedState>) -> Result<BTreeMap<String, Value>, String> {
    let db_path = state.db_path()?;
    let data_dir = state.data_dir()?;
    let conn = open_connection(&db_path)?;

    let mut payload = BTreeMap::new();
    payload.insert(
        "db_path".to_string(),
        Value::String(db_path.to_string_lossy().to_string()),
    );
    payload.insert(
        "data_dir".to_string(),
        Value::String(data_dir.to_string_lossy().to_string()),
    );
    payload.insert(
        "exports_dir".to_string(),
        Value::String(export_dir_for(&data_dir).to_string_lossy().to_string()),
    );
    payload.insert(
        "empresas".to_string(),
        Value::from(count_table(&conn, "empresas")?),
    );
    payload.insert(
        "usuarios".to_string(),
        Value::from(count_table(&conn, "usuarios")?),
    );
    payload.insert(
        "funcionarios".to_string(),
        Value::from(count_table(&conn, "funcionarios")?),
    );
    payload.insert(
        "equipamentos".to_string(),
        Value::from(count_table(&conn, "equipamentos")?),
    );
    payload.insert(
        "horarios".to_string(),
        Value::from(count_table(&conn, "horarios")?),
    );
    payload.insert(
        "batidas".to_string(),
        Value::from(count_table(&conn, "batidas")?),
    );
    payload.insert(
        "jornadas".to_string(),
        Value::from(count_table(&conn, "jornadas_trabalho")?),
    );
    payload.insert(
        "afd_importacoes".to_string(),
        Value::from(count_table(&conn, "afd_importacoes")?),
    );
    payload.insert(
        "banco_horas".to_string(),
        Value::from(count_table(&conn, "banco_horas_lancamentos")?),
    );
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

#[tauri::command]
pub fn app_meta() -> Result<BTreeMap<String, Value>, String> {
    let mut payload = BTreeMap::new();
    payload.insert(
        "version".to_string(),
        Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );
    payload.insert("build_hash".to_string(), Value::String(build_hash()));
    payload.insert(
        "product_name".to_string(),
        Value::String("Ponto Manager".to_string()),
    );
    Ok(payload)
}

#[tauri::command]
pub fn system_info(state: State<'_, SharedState>) -> Result<BTreeMap<String, Value>, String> {
    let db_path = state.db_path()?;
    let data_dir = state.data_dir()?;
    let bootstrap_path = SharedState::bootstrap_config_path()?;
    let mut payload = app_meta()?;
    payload.insert(
        "db_path".to_string(),
        Value::String(db_path.to_string_lossy().to_string()),
    );
    payload.insert(
        "data_dir".to_string(),
        Value::String(data_dir.to_string_lossy().to_string()),
    );
    payload.insert(
        "exports_dir".to_string(),
        Value::String(export_dir_for(&data_dir).to_string_lossy().to_string()),
    );
    payload.insert(
        "bootstrap_config".to_string(),
        Value::String(bootstrap_path.to_string_lossy().to_string()),
    );
    Ok(payload)
}

#[tauri::command]
pub fn system_set_data_dir(
    state: State<'_, SharedState>,
    data_dir: String,
) -> Result<BTreeMap<String, Value>, String> {
    let target_dir = PathBuf::from(data_dir.trim());
    if data_dir.trim().is_empty() {
        return Err("Informe um diretório válido para os parâmetros/dados.".to_string());
    }
    fs::create_dir_all(&target_dir)
        .map_err(|err| format!("Falha ao criar diretório informado: {err}"))?;

    let current_db = state.db_path()?;
    let current_data_dir = state.data_dir()?;
    let new_db = target_dir.join("pontos.db");

    if current_db.exists() && current_db != new_db {
        if let Some(parent) = new_db.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Falha ao preparar diretório do novo banco: {err}"))?;
        }
        fs::copy(&current_db, &new_db)
            .map_err(|err| format!("Falha ao copiar banco para o novo local: {err}"))?;

        let old_exports = export_dir_for(&current_data_dir);
        let new_exports = export_dir_for(&target_dir);
        if old_exports.exists() {
            fs::create_dir_all(&new_exports)
                .map_err(|err| format!("Falha ao preparar diretório de exportações: {err}"))?;
            for entry in fs::read_dir(&old_exports)
                .map_err(|err| format!("Falha ao ler exportações atuais: {err}"))?
            {
                let entry = entry.map_err(|err| format!("Falha ao iterar exportações: {err}"))?;
                let target = new_exports.join(entry.file_name());
                if entry.path().is_file() {
                    let _ = fs::copy(entry.path(), target);
                }
            }
        }
    }

    let cfg = json!({ "data_dir_override": target_dir.to_string_lossy().to_string() });
    SharedState::save_bootstrap_config(&cfg)?;
    state.reconfigure_data_dir(target_dir)?;
    system_info(state)
}

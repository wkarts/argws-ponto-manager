use std::{collections::BTreeMap, fs, path::PathBuf};

use super::auth::require_session_by_token;

use serde_json::{json, Map, Value};
use tauri::State;
#[cfg(feature = "desktop")]
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

use crate::{
    app_state::SharedState,
    db::{
        app_log_file_path, count_table, open_connection, row_to_json_map, write_app_log,
        AppLogInput,
    },
};

fn build_hash() -> String {
    option_env!("BUILD_HASH")
        .or(option_env!("GITHUB_SHA"))
        .map(|value| value.chars().take(8).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "dev".to_string())
}

fn runtime_app_name() -> String {
    std::env::var("ARGWS_PONTO_MANAGER_NAME")
        .or_else(|_| std::env::var("APP_NAME"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Ponto Manager".to_string())
}

fn runtime_app_identifier() -> String {
    std::env::var("ARGWS_PONTO_MANAGER_IDENTIFIER")
        .or_else(|_| std::env::var("APP_IDENTIFIER"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "br.com.argws.pontomanager".to_string())
}

fn export_dir_for(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("exports")
}

#[cfg(feature = "desktop")]
fn sanitize_print_file_name(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            output.push(ch);
        } else if ch.is_whitespace() {
            output.push('_');
        }
    }
    let trimmed = output.trim_matches('_').trim_matches('.');
    if trimmed.is_empty() {
        "relatorio".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

#[cfg(feature = "desktop")]
fn html_with_external_print_script(html: &str) -> String {
    let script = r#"<script>
(function () {
  function requestPrint() {
    setTimeout(function () { window.focus(); window.print(); }, 350);
  }
  if (document.readyState === 'complete') requestPrint();
  else window.addEventListener('load', requestPrint, { once: true });
})();
</script>"#;

    if html.to_lowercase().contains("window.print") {
        return html.to_string();
    }
    if let Some(index) = html.to_lowercase().rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script.len());
        result.push_str(&html[..index]);
        result.push_str(script);
        result.push_str(&html[index..]);
        result
    } else {
        format!("{html}{script}")
    }
}

#[tauri::command]
pub fn app_bootstrap(state: State<'_, SharedState>) -> Result<BTreeMap<String, Value>, String> {
    let db_path = state.db_path()?;
    let data_dir = state.data_dir()?;
    app_bootstrap_from_paths(&db_path, &data_dir)
}

pub(crate) fn app_bootstrap_from_paths(
    db_path: &std::path::Path,
    data_dir: &std::path::Path,
) -> Result<BTreeMap<String, Value>, String> {
    let conn = open_connection(db_path)?;

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
        Value::String(export_dir_for(data_dir).to_string_lossy().to_string()),
    );
    payload.insert(
        "empresas".to_string(),
        Value::from(count_table(&conn, "empresas")?),
    );
    payload.insert(
        "usuarios".to_string(),
        Value::from(count_table(&conn, "usuarios")?),
    );
    for (key, table) in [
        ("perfis", "perfis_acesso"),
        ("funcionarios", "funcionarios"),
        ("equipamentos", "equipamentos"),
        ("horarios", "horarios"),
        ("batidas", "batidas"),
        ("jornadas", "jornadas_trabalho"),
        ("afd_importacoes", "afd_importacoes"),
        ("banco_horas", "banco_horas_lancamentos"),
    ] {
        payload.insert(key.to_string(), Value::from(count_table(&conn, table)?));
    }
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
        "logs_error_today".to_string(),
        Value::from(
            conn.query_row(
                "SELECT COUNT(*) FROM app_logs WHERE level IN ('error','critical') AND substr(created_at, 1, 10) = date('now')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0),
        ),
    );
    payload.insert(
        "integrations_total".to_string(),
        Value::from(
            conn.query_row("SELECT COUNT(*) FROM integration_configs", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0),
        ),
    );
    payload.insert(
        "integrations_active".to_string(),
        Value::from(
            conn.query_row(
                "SELECT COUNT(*) FROM integration_configs WHERE ativo=1 AND status='active'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0),
        ),
    );

    for (key, sql) in [
        (
            "funcionarios_ativos",
            "SELECT COUNT(*) FROM funcionarios WHERE ativo = 1",
        ),
        (
            "funcionarios_inativos",
            "SELECT COUNT(*) FROM funcionarios WHERE ativo = 0",
        ),
        (
            "funcionarios_ferias_hoje",
            "SELECT COUNT(DISTINCT funcionario_id)
               FROM ferias_colaboradores
              WHERE ativo = 1
                AND status IN ('ativo', 'programado')
                AND date('now', 'localtime') BETWEEN data_inicial AND data_final",
        ),
        (
            "batidas_hoje",
            "SELECT COUNT(*) FROM batidas
              WHERE COALESCE(ativo, 1) = 1
                AND data_referencia = date('now', 'localtime')",
        ),
        (
            "batidas_pendentes_validacao",
            "SELECT COUNT(*) FROM batidas
              WHERE COALESCE(ativo, 1) = 1
                AND manual_ajuste = 1
                AND validado = 0",
        ),
        (
            "batidas_duplicadas_ocultas",
            "SELECT COUNT(*) FROM batidas
              WHERE COALESCE(ativo, 1) = 0
                AND status = 'duplicidade'",
        ),
        (
            "inconsistencias_hoje",
            "SELECT COUNT(*)
               FROM (
                    SELECT funcionario_id
                      FROM batidas
                     WHERE COALESCE(ativo, 1) = 1
                       AND data_referencia = date('now', 'localtime')
                     GROUP BY funcionario_id
                    HAVING COUNT(*) % 2 <> 0
               )",
        ),
        (
            "afd_importacoes_hoje",
            "SELECT COUNT(*) FROM afd_importacoes
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
        (
            "afd_processadas_hoje",
            "SELECT COALESCE(SUM(total_processadas), 0) FROM afd_importacoes
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
        (
            "afd_descartadas_hoje",
            "SELECT COALESCE(SUM(total_descartadas), 0) FROM afd_importacoes
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
        (
            "conector_coletas_hoje",
            "SELECT COUNT(*) FROM conector_coletas_log
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
        (
            "conector_importadas_hoje",
            "SELECT COALESCE(SUM(total_importadas), 0) FROM conector_coletas_log
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
        (
            "conector_duplicadas_hoje",
            "SELECT COALESCE(SUM(total_duplicadas), 0) FROM conector_coletas_log
              WHERE substr(created_at, 1, 10) = date('now', 'localtime')",
        ),
    ] {
        payload.insert(
            key.to_string(),
            Value::from(
                conn.query_row(sql, [], |row| row.get::<_, i64>(0))
                    .unwrap_or(0),
            ),
        );
    }

    for (key, sql) in [
        (
            "ultima_importacao_afd",
            "SELECT MAX(created_at) FROM afd_importacoes",
        ),
        (
            "ultima_coleta_conector",
            "SELECT MAX(created_at) FROM conector_coletas_log",
        ),
    ] {
        let value = conn
            .query_row(sql, [], |row| row.get::<_, Option<String>>(0))
            .unwrap_or(None);
        payload.insert(key.to_string(), value.map(Value::String).unwrap_or(Value::Null));
    }

    let mut batidas_por_dia = Vec::new();
    if let Ok(mut statement) = conn.prepare(
        "SELECT data_referencia, COUNT(*)
           FROM batidas
          WHERE COALESCE(ativo, 1) = 1
            AND data_referencia >= date('now', 'localtime', '-6 days')
            AND data_referencia <= date('now', 'localtime')
          GROUP BY data_referencia
          ORDER BY data_referencia",
    ) {
        if let Ok(rows) = statement.query_map([], |row| {
            Ok(json!({
                "data": row.get::<_, String>(0)?,
                "total": row.get::<_, i64>(1)?,
            }))
        }) {
            batidas_por_dia.extend(rows.filter_map(Result::ok));
        }
    }
    payload.insert(
        "batidas_por_dia".to_string(),
        Value::Array(batidas_por_dia),
    );

    payload.insert(
        "database_status".to_string(),
        Value::String("ok".to_string()),
    );
    payload.insert(
        "internal_api_status".to_string(),
        Value::String("headless-capable".to_string()),
    );
    payload.insert(
        "carga_padrao_minutos".to_string(),
        Value::from(
            conn.query_row(
                "SELECT COALESCE(valor, '480') FROM configuracoes \
                 WHERE nome = 'carga_padrao_minutos' LIMIT 1",
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
        Value::String(runtime_app_name()),
    );
    payload.insert(
        "identifier".to_string(),
        Value::String(runtime_app_identifier()),
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
    payload.insert(
        "log_file".to_string(),
        Value::String(app_log_file_path(&data_dir).to_string_lossy().to_string()),
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
    let new_db = crate::storage_contract::sqlite_database_path(&target_dir);

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

#[tauri::command]
#[cfg(feature = "desktop")]
pub async fn app_print_html(
    app: AppHandle,
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let html = payload
        .get("html")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if html.is_empty() {
        return Err("Não há conteúdo HTML para impressão.".to_string());
    }

    let data_dir = state.data_dir()?;
    let print_dir = data_dir.join("print-jobs");
    fs::create_dir_all(&print_dir)
        .map_err(|err| format!("Falha ao preparar diretório de impressão: {err}"))?;
    let requested_name = payload
        .get("file_name")
        .or_else(|| payload.get("fileName"))
        .and_then(Value::as_str)
        .unwrap_or("relatorio.html");
    let mut file_name = sanitize_print_file_name(requested_name);
    if !file_name.to_lowercase().ends_with(".html") {
        file_name.push_str(".html");
    }
    let stamp = chrono::Local::now().format("%Y%m%d%H%M%S%3f");
    let target = print_dir.join(format!("{stamp}_{file_name}"));
    fs::write(&target, html_with_external_print_script(html))
        .map_err(|err| format!("Falha ao gravar arquivo temporário de impressão: {err}"))?;

    let print_url = tauri::Url::from_file_path(&target)
        .map_err(|_| "Falha ao montar URL local para impressão.".to_string())?;
    let window_label = format!("print_{stamp}");
    WebviewWindowBuilder::new(&app, &window_label, WebviewUrl::CustomProtocol(print_url))
        .title("Impressão")
        .inner_size(1280.0, 900.0)
        .min_inner_size(900.0, 650.0)
        .center()
        .build()
        .map_err(|err| format!("Falha ao abrir janela de impressão: {err}"))?;

    let mut result = Map::new();
    result.insert("label".to_string(), Value::String(window_label));
    result.insert(
        "path".to_string(),
        Value::String(target.to_string_lossy().to_string()),
    );
    Ok(result)
}

#[tauri::command]
pub fn app_log_write(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let data_dir = state.data_dir()?;
    let conn = open_connection(&db_path)?;
    let level = payload
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("info");
    let category = payload
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("app");
    let message = payload
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("evento sem mensagem");
    let source = payload.get("source").and_then(Value::as_str);
    let route = payload.get("route").and_then(Value::as_str);
    let details = payload.get("details");
    write_app_log(
        &conn,
        &data_dir,
        AppLogInput {
            level,
            category,
            message,
            source,
            route,
            details,
        },
    )?;
    Ok(true)
}

#[tauri::command]
pub fn app_log_list(
    state: State<'_, SharedState>,
    session_token: String,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let _ = require_session_by_token(&conn, &session_token)?;
    let level = filters.get("level").and_then(Value::as_str).unwrap_or("");
    let category = filters
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("");
    let search = filters.get("search").and_then(Value::as_str).unwrap_or("");
    let limit = filters
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(300)
        .clamp(1, 5000);

    let mut sql = String::from(
        "SELECT id, level, category, message, source, route, details_json, created_at FROM app_logs WHERE 1=1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
    if !level.trim().is_empty() {
        sql.push_str(" AND level = ?");
        params_vec.push(rusqlite::types::Value::Text(level.trim().to_string()));
    }
    if !category.trim().is_empty() {
        sql.push_str(" AND category = ?");
        params_vec.push(rusqlite::types::Value::Text(category.trim().to_string()));
    }
    if !search.trim().is_empty() {
        sql.push_str(" AND (message LIKE ? OR COALESCE(details_json,'') LIKE ? OR COALESCE(route,'') LIKE ?)");
        let wild = format!("%{}%", search.trim());
        params_vec.push(rusqlite::types::Value::Text(wild.clone()));
        params_vec.push(rusqlite::types::Value::Text(wild.clone()));
        params_vec.push(rusqlite::types::Value::Text(wild));
    }
    sql.push_str(" ORDER BY id DESC LIMIT ?");
    params_vec.push(rusqlite::types::Value::Integer(limit));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar logs da aplicação: {err}"))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter()),
            row_to_json_map,
        )
        .map_err(|err| format!("Falha ao consultar logs da aplicação: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear logs da aplicação: {err}"))
}

#[tauri::command]
pub fn app_log_clear(state: State<'_, SharedState>, session_token: String) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let data_dir = state.data_dir()?;
    let conn = open_connection(&db_path)?;
    let identity = require_session_by_token(&conn, &session_token)?;
    if !identity.master_user {
        return Err("Apenas usuário master pode limpar os logs da aplicação.".to_string());
    }
    conn.execute("DELETE FROM app_logs", [])
        .map_err(|err| format!("Falha ao limpar logs da aplicação: {err}"))?;
    let log_path = app_log_file_path(&data_dir);
    if log_path.exists() {
        fs::write(&log_path, "").map_err(|err| format!("Falha ao limpar arquivo de log: {err}"))?;
    }
    Ok(true)
}

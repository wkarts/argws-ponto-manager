use crate::{
    app_state::SharedState,
    core::database::{config::DatabaseConfig, health, migrations as db_migrations},
    db::open_connection,
    internal_api,
};

use super::args::{CliArgs, RuntimeMode};

pub fn should_run_without_tauri(args: &CliArgs) -> bool {
    matches!(
        args.mode,
        RuntimeMode::HeadlessApi | RuntimeMode::WebProxy | RuntimeMode::Cli | RuntimeMode::Worker
    )
}

pub fn run(args: CliArgs) -> Result<(), String> {
    let _ = crate::runtime_config::init_env_file(None);
    let state = SharedState::new();
    if let Some(data_dir) = args.data_dir.clone() {
        state.init_with_data_dir(data_dir)?;
    } else {
        state.init()?;
    }
    let _ = crate::runtime_config::init_env_file(Some(&state));

    let db_config = DatabaseConfig::from_env_with_driver(&args.database_driver);
    let sqlite_path = state.db_path()?;
    if db_config.driver.is_external() {
        db_migrations::migrate(&db_config, &sqlite_path)?;
    }

    match args.mode {
        RuntimeMode::HeadlessApi | RuntimeMode::WebProxy => {
            start_requested_background_services(&state, &args)?;
            let config = internal_api::config::InternalApiConfig::default()
                .with_host_port(args.host.clone(), args.port);
            internal_api::server::run_blocking_with_config(&state, config)
        }
        RuntimeMode::Cli => {
            let db_path = state.db_path()?;
            let data_dir = state.data_dir()?;
            println!("Ponto Manager CLI");
            println!("Versão: {}", env!("CARGO_PKG_VERSION"));
            let health = health::report(&db_config, &db_path);
            println!("Banco SQLite local: {}", db_path.display());
            println!("Driver configurado: {}", health.driver);
            println!("Destino: {}", health.target);
            println!("Status banco: {} - {}", health.ok, health.message);
            println!("Dados: {}", data_dir.display());
            Ok(())
        }
        RuntimeMode::Worker => {
            let db_path = state.db_path()?;
            let conn = open_connection(&db_path)?;
            let pending: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sync_queue WHERE status = 'pending'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|err| format!("Falha ao consultar fila de sincronização: {err}"))?;
            let failed: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sync_queue WHERE status = 'failed'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|err| format!("Falha ao consultar falhas de sincronização: {err}"))?;

            println!("Ponto Manager Worker");
            println!("Banco: {}", db_path.display());
            println!("Itens pendentes: {pending}");
            println!("Itens com falha: {failed}");
            println!(
                "A fila foi preservada. O envio exige uma integração habilitada e autenticada."
            );
            Ok(())
        }
        RuntimeMode::Desktop => Ok(()),
    }
}

fn start_requested_background_services(state: &SharedState, args: &CliArgs) -> Result<(), String> {
    let settings = crate::runtime_config::load_settings(state)
        .map(|payload| payload.settings)
        .unwrap_or_default();

    let web_enabled = settings.local_web_enabled
        && !env_bool("ARGWS_PONTO_MANAGER_WEB_DISABLED", false)
        && !env_bool("APP_WEB_DISABLED", false);
    if web_enabled && (args.start_web_proxy || args.start_services || settings.local_web_auto_start)
    {
        crate::internal_api::web_server::start_background(state.clone())?;
    }

    let webhook_enabled = settings.webhook_enabled
        && !env_bool("ARGWS_PONTO_MANAGER_WEBHOOK_DISABLED", false)
        && !env_bool("APP_WEBHOOK_DISABLED", false);
    if webhook_enabled
        && (args.start_webhook
            || args.start_services
            || settings.services_auto_start
            || settings.webhook_auto_start
            || env_bool("ARGWS_PONTO_MANAGER_WEBHOOK_AUTO_START", false)
            || env_bool("ARGWS_PONTO_MANAGER_SERVICES_AUTO_START", false))
    {
        crate::native_webhook::server::start_background(state.clone())?;
    }

    let websocket_enabled = settings.websocket_enabled
        && !env_bool("ARGWS_PONTO_MANAGER_WEBSOCKET_DISABLED", false)
        && !env_bool("APP_WEBSOCKET_DISABLED", false);
    if websocket_enabled
        && (args.start_websocket
            || args.start_services
            || settings.services_auto_start
            || settings.websocket_auto_start
            || env_bool("ARGWS_PONTO_MANAGER_WEBSOCKET_AUTO_START", false)
            || env_bool("ARGWS_PONTO_MANAGER_SERVICES_AUTO_START", false))
    {
        crate::native_websocket::server::start_background(state.clone())?;
    }

    Ok(())
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

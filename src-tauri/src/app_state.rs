use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::migrations;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub db_path: PathBuf,
}

#[derive(Clone)]
pub struct SharedState {
    inner: Arc<RwLock<Option<AppContext>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub fn init(&self) -> Result<(), String> {
        let base_dir = dirs::data_local_dir()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "Não foi possível resolver o diretório de dados.".to_string())?
            .join("pontos_desktop_tauri");

        std::fs::create_dir_all(&base_dir)
            .map_err(|err| format!("Falha ao criar diretório de dados: {err}"))?;

        let db_path = base_dir.join("pontos.db");
        migrations::migrate(&db_path)?;

        let mut guard = self
            .inner
            .write()
            .map_err(|_| "Falha ao obter lock de escrita do estado.".to_string())?;

        *guard = Some(AppContext { db_path });
        Ok(())
    }

    pub fn db_path(&self) -> Result<PathBuf, String> {
        let guard = self
            .inner
            .read()
            .map_err(|_| "Falha ao obter lock de leitura do estado.".to_string())?;

        guard
            .as_ref()
            .map(|ctx| ctx.db_path.clone())
            .ok_or_else(|| "Aplicação ainda não inicializada.".to_string())
    }
}

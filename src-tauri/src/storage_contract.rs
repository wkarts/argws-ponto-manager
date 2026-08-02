use std::path::{Path, PathBuf};

/// Contrato de persistência adotado a partir da versão 1.24.0.
pub const CURRENT_LOCAL_DATA_DIR: &str = "argws-ponto-manager";
pub const CURRENT_SQLITE_DATABASE_FILE_NAME: &str = "ponto-manager.db";

/// Contrato legado das versões anteriores à 1.24.0.
/// Estes caminhos são somente origens de migração e nunca o destino atual.
pub const LEGACY_LOCAL_DATA_DIRS: &[&str] = &["pontos_desktop_tauri", "pontos-desktop-tauri"];
pub const LEGACY_SQLITE_DATABASE_FILE_NAME: &str = "pontos.db";

pub fn sqlite_database_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CURRENT_SQLITE_DATABASE_FILE_NAME)
}

pub fn legacy_database_candidates_in(data_dir: &Path) -> Vec<PathBuf> {
    vec![data_dir.join(LEGACY_SQLITE_DATABASE_FILE_NAME)]
}

pub fn known_legacy_database_candidates(base_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for slug in LEGACY_LOCAL_DATA_DIRS {
        candidates.extend(legacy_database_candidates_in(&base_dir.join(slug)));
    }
    candidates
}

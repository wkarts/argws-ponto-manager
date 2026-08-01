use rusqlite::{params, Connection};
use std::{env, fs, path::Path};
use tempfile::TempDir;

#[path = "../../../src-tauri/src/bootstrap.rs"]
mod bootstrap;
#[path = "../../../src-tauri/src/legacy_data.rs"]
mod legacy_data;

fn create_legacy_database(path: &Path) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.execute_batch(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, login TEXT NOT NULL);
         CREATE TABLE empresas (id INTEGER PRIMARY KEY, nome TEXT NOT NULL);
         CREATE TABLE funcionarios (id INTEGER PRIMARY KEY, nome TEXT NOT NULL);
         CREATE TABLE batidas (id INTEGER PRIMARY KEY, funcionario_id INTEGER NOT NULL);
         CREATE TABLE ferias_colaboradores (id INTEGER PRIMARY KEY, funcionario_id INTEGER NOT NULL);
         INSERT INTO usuarios VALUES (1, 'admin');
         INSERT INTO empresas VALUES (1, 'Empresa teste');
         INSERT INTO funcionarios VALUES (1, 'Colaborador teste');
         INSERT INTO batidas VALUES (1, 1);
         INSERT INTO ferias_colaboradores VALUES (1, 1);",
    )
    .map_err(|error| error.to_string())
}

fn scalar_count(path: &Path, table: &str) -> Result<i64, String> {
    Connection::open(path)
        .and_then(|conn| conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0)))
        .map_err(|error| error.to_string())
}

fn test_new_install() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let target = temp.path().join("ponto-manager.db");
    env::set_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH", temp.path().join("inexistente.db"));
    let recovery = legacy_data::prepare(temp.path(), &target)?;
    if recovery.is_some() || target.exists() {
        return Err("Instalação nova não deveria criar cópia legada.".to_string());
    }
    Ok(())
}

fn test_upgrade_repeat_and_completed_start() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let legacy = temp.path().join("pontos.db");
    let data_dir = temp.path().join("novo");
    let target = data_dir.join("ponto-manager.db");
    create_legacy_database(&legacy)?;
    env::set_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH", &legacy);

    let recovery = legacy_data::prepare(&data_dir, &target)?
        .ok_or_else(|| "Banco legado não foi detectado.".to_string())?;
    if scalar_count(&target, "batidas")? != 1 || scalar_count(&legacy, "batidas")? != 1 {
        return Err("Contagens críticas divergiram durante a cópia.".to_string());
    }
    legacy_data::finalize(&data_dir, &target, Some(&recovery))?;
    if !data_dir.join("legacy-database-migration-v1.json").is_file() {
        return Err("Marcador idempotente não foi criado.".to_string());
    }
    if legacy_data::prepare(&data_dir, &target)?.is_some() {
        return Err("Execução repetida não deveria migrar novamente.".to_string());
    }
    Ok(())
}

fn test_corrupt_legacy() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let legacy = temp.path().join("pontos.db");
    fs::write(&legacy, b"sqlite corrompido").map_err(|error| error.to_string())?;
    env::set_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH", &legacy);
    let target = temp.path().join("novo/ponto-manager.db");
    if legacy_data::prepare(target.parent().unwrap_or(temp.path()), &target).is_ok() {
        return Err("Banco legado corrompido foi aceito.".to_string());
    }
    if fs::read(&legacy).map_err(|error| error.to_string())? != b"sqlite corrompido" {
        return Err("Banco legado corrompido foi modificado.".to_string());
    }
    Ok(())
}

fn test_copy_failure_preserves_source() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let source = temp.path().join("origem.db");
    create_legacy_database(&source)?;
    let destination = temp.path().join("destino-como-diretorio");
    fs::create_dir(&destination).map_err(|error| error.to_string())?;
    let before = scalar_count(&source, "usuarios")?;
    let error = legacy_data::copy_synced(&source, &destination)
        .expect_err("Destino inválido deveria simular falha de espaço/permissão.");
    if !error.contains("espaço") || scalar_count(&source, "usuarios")? != before {
        return Err("Falha de cópia não preservou origem ou não orientou sobre espaço.".to_string());
    }
    Ok(())
}

fn test_failed_migration_rollback() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let legacy = temp.path().join("pontos.db");
    let data_dir = temp.path().join("novo");
    let target = data_dir.join("ponto-manager.db");
    create_legacy_database(&legacy)?;
    env::set_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH", &legacy);
    let recovery = legacy_data::prepare(&data_dir, &target)?;
    Connection::open(&target)
        .and_then(|conn| conn.execute("DELETE FROM usuarios", params![]))
        .map_err(|error| error.to_string())?;
    legacy_data::rollback(&target, recovery.as_ref())?;
    if target.exists() || scalar_count(&legacy, "usuarios")? != 1 {
        return Err("Rollback da migração inicial não preservou o banco legado.".to_string());
    }
    Ok(())
}

fn test_existing_database_rollback() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let target = temp.path().join("ponto-manager.db");
    create_legacy_database(&target)?;
    env::set_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH", temp.path().join("inexistente.db"));
    let recovery = legacy_data::prepare(temp.path(), &target)?;
    Connection::open(&target)
        .and_then(|conn| conn.execute("DELETE FROM usuarios", params![]))
        .map_err(|error| error.to_string())?;
    legacy_data::rollback(&target, recovery.as_ref())?;
    if scalar_count(&target, "usuarios")? != 1 {
        return Err("Rollback não restaurou banco existente a partir do backup.".to_string());
    }
    Ok(())
}

fn test_bootstrap_credential_lifecycle() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let db_path = temp.path().join("ponto-manager.db");
    let credential_file = temp.path().join(".bootstrap-admin.local");
    env::set_var("ARGWS_PONTO_MANAGER_BOOTSTRAP_FILE", &credential_file);
    let first = bootstrap::load_or_create(&db_path)?;
    let second = bootstrap::load_or_create(&db_path)?;
    if first.password.len() != 32 || first.password != second.password || !credential_file.is_file() {
        return Err("Credencial bootstrap não é forte ou idempotente.".to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&credential_file)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777
            != 0o600
        {
            return Err("Credencial bootstrap não possui permissão 0600.".to_string());
        }
    }
    bootstrap::remove_after_password_change(&db_path)?;
    if credential_file.exists() {
        return Err("Credencial bootstrap não foi removida após troca de senha.".to_string());
    }
    Ok(())
}

fn main() -> Result<(), String> {
    test_new_install()?;
    test_upgrade_repeat_and_completed_start()?;
    test_corrupt_legacy()?;
    test_copy_failure_preserves_source()?;
    test_failed_migration_rollback()?;
    test_existing_database_rollback()?;
    test_bootstrap_credential_lifecycle()?;
    env::remove_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH");
    env::remove_var("ARGWS_PONTO_MANAGER_BOOTSTRAP_FILE");
    println!("7 cenários de migração/segurança aprovados.");
    Ok(())
}

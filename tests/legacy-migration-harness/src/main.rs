use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::{env, fs, io::Read, path::Path};
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

fn create_legacy_credential_database(
    path: &Path,
    password_hash: &str,
    temporary_password: i64,
) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.execute_batch(
        "CREATE TABLE usuarios (
            id INTEGER PRIMARY KEY,
            login TEXT NOT NULL,
            senha_hash TEXT NOT NULL,
            senha_provisoria INTEGER NOT NULL DEFAULT 0
         );",
    )
    .map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO usuarios (id, login, senha_hash, senha_provisoria)
         VALUES (1, 'admin', ?1, ?2)",
        params![password_hash, temporary_password],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn create_affected_target_database(
    path: &Path,
    password_hash: &str,
    last_login: Option<&str>,
) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.execute_batch(
        "CREATE TABLE usuarios (
            id INTEGER PRIMARY KEY,
            login TEXT NOT NULL,
            senha_hash TEXT NOT NULL,
            senha_provisoria INTEGER NOT NULL DEFAULT 0,
            ultimo_login_em TEXT,
            updated_at TEXT NOT NULL
         );
         CREATE TABLE app_settings (
            chave TEXT PRIMARY KEY,
            valor TEXT,
            updated_at TEXT NOT NULL
         );
         INSERT INTO app_settings (chave, valor, updated_at)
         VALUES ('security_bootstrap_credential_rotation_v1', 'completed', '2026-08-01T00:00:00Z');",
    )
    .map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO usuarios (
            id, login, senha_hash, senha_provisoria, ultimo_login_em, updated_at
         ) VALUES (1, 'admin', ?1, 1, ?2, '2026-08-01T00:00:00Z')",
        params![password_hash, last_login],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_legacy_migration_marker(
    data_dir: &Path,
    legacy: &Path,
    target: &Path,
) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
    let marker = serde_json::json!({
        "schema_version": 1,
        "application_version": "1.24.2",
        "source_path": legacy.to_string_lossy(),
        "source_checksum_sha256": sha256_file(legacy)?,
        "target_path": target.to_string_lossy(),
        "backup_path": data_dir.join("backups/original.db").to_string_lossy(),
        "completed_at": "2026-08-01T00:00:00Z"
    });
    fs::write(
        data_dir.join("legacy-database-migration-v1.json"),
        serde_json::to_vec_pretty(&marker).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
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

fn test_rotated_legacy_credential_is_restored() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let legacy = temp.path().join("pontos.db");
    let data_dir = temp.path().join("novo");
    let target = data_dir.join("ponto-manager.db");
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    create_legacy_credential_database(&legacy, "hash-legado-preservado", 0)?;
    create_affected_target_database(&target, "hash-bootstrap-rotacionado", None)?;
    write_legacy_migration_marker(&data_dir, &legacy, &target)?;
    fs::write(
        data_dir.join(format!("schema-backup-{}.ok", env!("CARGO_PKG_VERSION"))),
        b"ok\n",
    )
    .map_err(|error| error.to_string())?;

    let recovery = legacy_data::prepare(&data_dir, &target)?
        .ok_or_else(|| "Reparo de credencial deveria forçar backup do destino.".to_string())?;
    if !legacy_data::restore_rotated_legacy_credential(&data_dir, &target)? {
        return Err("Credencial legada afetada não foi restaurada.".to_string());
    }
    legacy_data::finalize(&data_dir, &target, Some(&recovery))?;

    let restored: (String, i64) = Connection::open(&target)
        .and_then(|conn| {
            conn.query_row(
                "SELECT senha_hash, senha_provisoria FROM usuarios WHERE login = 'admin'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        })
        .map_err(|error| error.to_string())?;
    if restored != ("hash-legado-preservado".to_string(), 0) {
        return Err("Hash ou estado provisório da credencial legada foi alterado.".to_string());
    }
    if legacy_data::restore_rotated_legacy_credential(&data_dir, &target)? {
        return Err("Reparo de credencial não foi idempotente.".to_string());
    }
    let source_hash: String = Connection::open(&legacy)
        .and_then(|conn| {
            conn.query_row(
                "SELECT senha_hash FROM usuarios WHERE login = 'admin'",
                [],
                |row| row.get(0),
            )
        })
        .map_err(|error| error.to_string())?;
    if source_hash != "hash-legado-preservado" {
        return Err("Banco legado original foi modificado durante o reparo.".to_string());
    }
    Ok(())
}

fn test_used_target_credential_is_not_overwritten() -> Result<(), String> {
    let temp = TempDir::new().map_err(|error| error.to_string())?;
    let legacy = temp.path().join("pontos.db");
    let data_dir = temp.path().join("novo");
    let target = data_dir.join("ponto-manager.db");
    fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    create_legacy_credential_database(&legacy, "hash-legado-antigo", 0)?;
    create_affected_target_database(
        &target,
        "hash-atual-em-uso",
        Some("2026-08-01T12:00:00Z"),
    )?;
    write_legacy_migration_marker(&data_dir, &legacy, &target)?;

    if legacy_data::restore_rotated_legacy_credential(&data_dir, &target)? {
        return Err("Credencial já utilizada não deveria ser sobrescrita.".to_string());
    }
    let target_hash: String = Connection::open(&target)
        .and_then(|conn| {
            conn.query_row(
                "SELECT senha_hash FROM usuarios WHERE login = 'admin'",
                [],
                |row| row.get(0),
            )
        })
        .map_err(|error| error.to_string())?;
    if target_hash != "hash-atual-em-uso" {
        return Err("Credencial atual foi alterada indevidamente.".to_string());
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
    test_rotated_legacy_credential_is_restored()?;
    test_used_target_credential_is_not_overwritten()?;
    env::remove_var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH");
    env::remove_var("ARGWS_PONTO_MANAGER_BOOTSTRAP_FILE");
    println!("9 cenários de migração/segurança aprovados.");
    Ok(())
}

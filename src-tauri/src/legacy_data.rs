use chrono::Utc;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MIGRATION_MARKER: &str = "legacy-database-migration-v1.json";

#[derive(Debug, Clone)]
pub struct DatabaseRecovery {
    backup_path: PathBuf,
    target_existed: bool,
    copied_from_legacy: bool,
    source_path: PathBuf,
    source_checksum: String,
}

#[derive(Debug, Serialize)]
struct MigrationMarker {
    schema_version: u32,
    application_version: String,
    source_path: String,
    source_checksum_sha256: String,
    target_path: String,
    backup_path: String,
    completed_at: String,
}

pub fn prepare(data_dir: &Path, target_db: &Path) -> Result<Option<DatabaseRecovery>, String> {
    fs::create_dir_all(data_dir)
        .map_err(|err| format!("Falha ao preparar diretório de dados: {err}"))?;

    if target_db.is_file() {
        validate_integrity(target_db)?;
        let marker = data_dir.join(format!("schema-backup-{}.ok", env!("CARGO_PKG_VERSION")));
        if marker.is_file() {
            return Ok(None);
        }
        return backup_database(data_dir, target_db, true, false).map(Some);
    }

    let Some(legacy_db) = resolve_legacy_database()? else {
        return Ok(None);
    };
    validate_integrity(&legacy_db)?;
    let recovery = backup_database(data_dir, &legacy_db, false, true)?;
    let temporary = target_db.with_extension("db.migrating");
    if temporary.exists() {
        fs::remove_file(&temporary)
            .map_err(|err| format!("Falha ao limpar cópia temporária anterior: {err}"))?;
    }
    copy_synced(&legacy_db, &temporary)?;
    validate_integrity(&temporary)?;
    validate_critical_counts(&legacy_db, &temporary)?;
    fs::rename(&temporary, target_db)
        .map_err(|err| format!("Falha ao ativar cópia migrada do banco legado: {err}"))?;
    Ok(Some(recovery))
}

pub fn finalize(
    data_dir: &Path,
    target_db: &Path,
    recovery: Option<&DatabaseRecovery>,
) -> Result<(), String> {
    validate_integrity(target_db)?;
    let version_marker = data_dir.join(format!("schema-backup-{}.ok", env!("CARGO_PKG_VERSION")));
    write_private_file(&version_marker, b"ok\n")?;

    let Some(recovery) = recovery else {
        return Ok(());
    };
    if !recovery.copied_from_legacy {
        return Ok(());
    }
    let marker = MigrationMarker {
        schema_version: 1,
        application_version: env!("CARGO_PKG_VERSION").to_string(),
        source_path: recovery.source_path.to_string_lossy().to_string(),
        source_checksum_sha256: recovery.source_checksum.clone(),
        target_path: target_db.to_string_lossy().to_string(),
        backup_path: recovery.backup_path.to_string_lossy().to_string(),
        completed_at: Utc::now().to_rfc3339(),
    };
    let payload = serde_json::to_vec_pretty(&marker)
        .map_err(|err| format!("Falha ao serializar marcador da migração: {err}"))?;
    write_private_file(&data_dir.join(MIGRATION_MARKER), &payload)
}

pub fn rollback(target_db: &Path, recovery: Option<&DatabaseRecovery>) -> Result<(), String> {
    let Some(recovery) = recovery else {
        return Ok(());
    };
    if recovery.target_existed {
        let temporary = target_db.with_extension("db.rollback");
        copy_synced(&recovery.backup_path, &temporary)?;
        validate_integrity(&temporary)?;
        if target_db.exists() {
            fs::remove_file(target_db)
                .map_err(|err| format!("Falha ao remover banco com migration reprovada: {err}"))?;
        }
        fs::rename(&temporary, target_db)
            .map_err(|err| format!("Falha ao restaurar banco anterior: {err}"))?;
    } else if target_db.exists() {
        fs::remove_file(target_db)
            .map_err(|err| format!("Falha ao remover cópia migrada reprovada: {err}"))?;
    }
    Ok(())
}

fn resolve_legacy_database() -> Result<Option<PathBuf>, String> {
    let mut candidates = Vec::new();
    if let Ok(configured) = std::env::var("ARGWS_PONTO_MANAGER_LEGACY_DB_PATH") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed));
        }
    }
    if let Some(base) = dirs::data_local_dir() {
        let underscore_slug = ["pontos", "desktop", "tauri"].join("_");
        let hyphen_slug = ["pontos", "desktop", "tauri"].join("-");
        candidates.push(base.join(underscore_slug).join("pontos.db"));
        candidates.push(base.join(hyphen_slug).join("pontos.db"));
    }
    Ok(candidates.into_iter().find(|path| path.is_file()))
}

fn backup_database(
    data_dir: &Path,
    source: &Path,
    target_existed: bool,
    copied_from_legacy: bool,
) -> Result<DatabaseRecovery, String> {
    let checksum = sha256_file(source)?;
    let backups = data_dir.join("backups");
    fs::create_dir_all(&backups)
        .map_err(|err| format!("Falha ao criar diretório de backups: {err}"))?;
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let backup_path = backups.join(format!(
        "ponto-manager-{}-{}-{}.db",
        env!("CARGO_PKG_VERSION"),
        stamp,
        &checksum[..16]
    ));
    copy_synced(source, &backup_path)?;
    validate_integrity(&backup_path)?;
    Ok(DatabaseRecovery {
        backup_path,
        target_existed,
        copied_from_legacy,
        source_path: source.to_path_buf(),
        source_checksum: checksum,
    })
}

pub(crate) fn copy_synced(source: &Path, destination: &Path) -> Result<(), String> {
    let mut input = File::open(source)
        .map_err(|err| format!("Falha ao abrir banco de origem {}: {err}", source.display()))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut output = options.open(destination).map_err(|err| {
        format!(
            "Falha ao criar cópia do banco em {} (verifique espaço e permissões): {err}",
            destination.display()
        )
    })?;
    std::io::copy(&mut input, &mut output)
        .map_err(|err| format!("Falha durante cópia segura do banco: {err}"))?;
    output
        .sync_all()
        .map_err(|err| format!("Falha ao sincronizar cópia do banco: {err}"))?;
    Ok(())
}

fn validate_integrity(path: &Path) -> Result<(), String> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags).map_err(|err| {
        format!(
            "Falha ao abrir SQLite para validação {}: {err}",
            path.display()
        )
    })?;
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|err| {
            format!(
                "Falha ao executar integrity_check em {}: {err}",
                path.display()
            )
        })?;
    if result.eq_ignore_ascii_case("ok") {
        Ok(())
    } else {
        Err(format!(
            "Banco SQLite inválido em {}: {result}",
            path.display()
        ))
    }
}

fn validate_critical_counts(source: &Path, target: &Path) -> Result<(), String> {
    let source_conn = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|err| format!("Falha ao abrir banco legado para contagem: {err}"))?;
    let target_conn = Connection::open_with_flags(target, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|err| format!("Falha ao abrir cópia para contagem: {err}"))?;
    for table in [
        "usuarios",
        "empresas",
        "funcionarios",
        "batidas",
        "ferias_colaboradores",
    ] {
        if !table_exists(&source_conn, table)? {
            continue;
        }
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let source_count: i64 = source_conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|err| format!("Falha ao contar {table} na origem: {err}"))?;
        let target_count: i64 = target_conn
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|err| format!("Falha ao contar {table} na cópia: {err}"))?;
        if source_count != target_count {
            return Err(format!(
                "Contagem divergente na cópia de {table}: origem={source_count}, destino={target_count}"
            ));
        }
    }
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .map_err(|err| format!("Falha ao inspecionar tabela {table}: {err}"))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|err| format!("Falha ao abrir arquivo para checksum: {err}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|err| format!("Falha ao calcular checksum: {err}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_private_file(path: &Path, payload: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|err| format!("Falha ao gravar marcador {}: {err}", path.display()))?;
    file.write_all(payload)
        .and_then(|_| file.sync_all())
        .map_err(|err| format!("Falha ao sincronizar marcador {}: {err}", path.display()))
}

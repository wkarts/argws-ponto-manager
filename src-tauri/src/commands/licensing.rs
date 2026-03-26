use chrono::{Duration, Utc};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    security::{decrypt_text, encrypt_text, integrity_hash, machine_key},
};

fn company_seed(conn: &rusqlite::Connection, empresa_id: Option<i64>) -> Result<(i64, String, String), String> {
    let result = if let Some(empresa_id) = empresa_id {
        conn.query_row(
            "SELECT id, COALESCE(documento, ''), COALESCE(nome, '') FROM empresas WHERE id = ?1 LIMIT 1",
            [empresa_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
    } else {
        conn.query_row(
            "SELECT id, COALESCE(documento, ''), COALESCE(nome, '') FROM empresas ORDER BY id ASC LIMIT 1",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )
    };

    result
        .optional()
        .map_err(|err| format!("Falha ao localizar empresa para licenciamento: {err}"))?
        .ok_or_else(|| "Cadastre ao menos uma empresa para utilizar o licenciamento.".to_string())
}

#[tauri::command]
pub fn licensing_status(
    state: State<'_, SharedState>,
    empresa_id: Option<i64>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let (empresa_id_resolved, documento, empresa_nome) = company_seed(&conn, empresa_id)?;

    let row = conn
        .query_row(
            "SELECT id, cnpj, license_kind, status, issued_at, expires_at, fingerprint, payload_encrypted, integrity_hash, created_at, updated_at
             FROM local_licenses WHERE empresa_id = ?1 LIMIT 1",
            [empresa_id_resolved],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar licença local: {err}"))?;

    let mut result = Map::new();
    result.insert("empresa_id".to_string(), Value::from(empresa_id_resolved));
    result.insert("empresa_nome".to_string(), Value::from(empresa_nome));
    result.insert("cnpj".to_string(), Value::from(documento.clone()));
    result.insert("machine_key".to_string(), Value::from(machine_key()));

    if let Some(mut row) = row {
        let encrypted = row.get("payload_encrypted").and_then(Value::as_str).unwrap_or("");
        let decrypted = if encrypted.is_empty() {
            None
        } else {
            decrypt_text(&documento, encrypted).ok()
        };
        row.insert(
            "payload_decrypted".to_string(),
            decrypted.map(Value::from).unwrap_or(Value::Null),
        );
        row.insert(
            "is_trial".to_string(),
            Value::from(row.get("license_kind").and_then(Value::as_str) == Some("trial")),
        );
        result.insert("license".to_string(), Value::Object(row));
    } else {
        result.insert("license".to_string(), Value::Null);
    }

    Ok(result)
}

#[tauri::command]
pub fn licensing_start_trial(
    state: State<'_, SharedState>,
    empresa_id: Option<i64>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now();
    let (empresa_id_resolved, documento, empresa_nome) = company_seed(&conn, empresa_id)?;
    let cnpj = documento.trim().to_string();
    if cnpj.is_empty() {
        return Err("A licença de teste exige CNPJ/CPF cadastrado na empresa ativa.".to_string());
    }

    let existing: Option<Map<String, Value>> = conn
        .query_row(
            "SELECT id, empresa_id, cnpj, license_kind, status, issued_at, expires_at, fingerprint, payload_encrypted, integrity_hash, created_at, updated_at
             FROM local_licenses WHERE empresa_id = ?1 LIMIT 1",
            [empresa_id_resolved],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar licença existente: {err}"))?;
    if let Some(row) = existing {
        return Ok(row);
    }

    let expires_at = (now + Duration::days(45)).to_rfc3339();
    let issued_at = now.to_rfc3339();
    let payload = json!({
        "empresa_id": empresa_id_resolved,
        "empresa_nome": empresa_nome,
        "cnpj": cnpj,
        "kind": "trial",
        "issued_at": issued_at,
        "expires_at": expires_at,
        "machine_key": machine_key(),
        "days": 45,
    });
    let payload_str = payload.to_string();
    let payload_encrypted = encrypt_text(&cnpj, &payload_str);
    let payload_hash = integrity_hash(&cnpj, &payload_str);

    conn.execute(
        "INSERT INTO local_licenses (
            empresa_id, cnpj, license_kind, status, issued_at, expires_at, fingerprint,
            payload_encrypted, integrity_hash, created_at, updated_at
        ) VALUES (?1, ?2, 'trial', 'active', ?3, ?4, ?5, ?6, ?7, ?3, ?3)",
        params![
            empresa_id_resolved,
            cnpj,
            issued_at,
            expires_at,
            machine_key(),
            payload_encrypted,
            payload_hash,
        ],
    )
    .map_err(|err| format!("Falha ao gravar licença de teste: {err}"))?;

    let id = conn.last_insert_rowid();
    let saved = conn
        .query_row(
            "SELECT id, empresa_id, cnpj, license_kind, status, issued_at, expires_at, fingerprint, payload_encrypted, integrity_hash, created_at, updated_at
             FROM local_licenses WHERE id = ?1",
            [id],
            row_to_json_map,
        )
        .map_err(|err| format!("Falha ao reler licença salva: {err}"))?;

    let payload_value = Value::Object(saved.clone());
    write_audit(&conn, "local_licenses", "create_trial", Some(id), &payload_value)?;
    enqueue_sync(&conn, "local_licenses", "create_trial", Some(id), &payload_value)?;
    Ok(saved)
}

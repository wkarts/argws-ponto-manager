use crate::{
    app_state::SharedState,
    commands::afd::afd_import_file,
    db::{open_connection, write_audit},
    models::AfdImportRequest,
    services::conector_client::ConectorClient,
};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::fs;
use tauri::{command, State};

const SETTING_CONECTOR_URL: &str = "ponto_conector_url";
const SETTING_CONECTOR_TOKEN: &str = "ponto_conector_token";
const SETTING_CONECTOR_TIMEOUT: &str = "ponto_conector_timeout";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConectorImportarAfdArgs {
    pub empresa_id: Option<i64>,
    pub equipamento_id: i64,
    pub mode: Option<String>,
    pub completo: Option<bool>,
    pub nsr_inicio: Option<i64>,
    pub nsr_fim: Option<i64>,
    pub data_inicio: Option<String>,
    pub data_fim: Option<String>,
}

fn setting_value(conn: &rusqlite::Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT valor FROM app_settings WHERE chave = ?1 LIMIT 1",
        [key],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|value| value.flatten())
    .map_err(|err| format!("Falha ao consultar configuração do conector {key}: {err}"))
}

fn save_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_settings (chave, valor, created_at, updated_at) VALUES (?1, ?2, ?3, ?3) ON CONFLICT(chave) DO UPDATE SET valor = excluded.valor, updated_at = excluded.updated_at",
        params![key, value, Utc::now().to_rfc3339()],
    )
    .map_err(|err| format!("Falha ao salvar configuração do conector {key}: {err}"))?;
    Ok(())
}

fn build_client(conn: &rusqlite::Connection) -> Result<ConectorClient, String> {
    let base_url = setting_value(conn, SETTING_CONECTOR_URL)?
        .or_else(|| std::env::var("PONTO_CONECTOR_URL").ok())
        .unwrap_or_default();
    let api_token = setting_value(conn, SETTING_CONECTOR_TOKEN)?
        .or_else(|| std::env::var("PONTO_CONECTOR_TOKEN").ok())
        .unwrap_or_default();
    let timeout_secs = setting_value(conn, SETTING_CONECTOR_TIMEOUT)?
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| {
            std::env::var("PONTO_CONECTOR_TIMEOUT")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(30);

    if base_url.trim().is_empty() {
        return Err(
            "Configure a URL base da API do Ponto Manager Conector nas configurações do conector."
                .to_string(),
        );
    }
    if api_token.trim().is_empty() {
        return Err(
            "Configure o token da API do Ponto Manager Conector nas configurações do conector."
                .to_string(),
        );
    }

    ConectorClient::new(
        base_url.trim().trim_end_matches('/').to_string(),
        api_token,
        timeout_secs,
    )
}

fn log_coleta(conn: &rusqlite::Connection, payload: &Value) -> Result<(), String> {
    conn.execute(
        "INSERT INTO conector_coletas_log (equipamento_id, conector_device_id, tipo, status, mensagem, total_recebidas, total_importadas, total_duplicadas, nsr_inicio, nsr_fim, arquivo_path, payload_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            payload.get("equipamento_id").and_then(Value::as_i64).unwrap_or_default(),
            payload.get("conector_device_id").and_then(Value::as_str),
            payload.get("tipo").and_then(Value::as_str).unwrap_or("batidas"),
            payload.get("status").and_then(Value::as_str).unwrap_or("erro"),
            payload.get("mensagem").and_then(Value::as_str),
            payload.get("total_recebidas").and_then(Value::as_i64).unwrap_or_default(),
            payload.get("total_importadas").and_then(Value::as_i64).unwrap_or_default(),
            payload.get("total_duplicadas").and_then(Value::as_i64).unwrap_or_default(),
            payload.get("nsr_inicio").and_then(Value::as_i64),
            payload.get("nsr_fim").and_then(Value::as_i64),
            payload.get("arquivo_path").and_then(Value::as_str),
            payload.get("payload_json").map(Value::to_string).unwrap_or_default(),
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|err| format!("Falha ao gravar log do conector: {err}"))?;
    Ok(())
}

fn parse_datetime(value: &str) -> Option<(String, String)> {
    if value.contains('T') {
        let parts: Vec<&str> = value.split('T').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].chars().take(5).collect()));
        }
    }
    None
}

fn find_funcionario(
    conn: &rusqlite::Connection,
    raw: &Map<String, Value>,
) -> Result<Option<i64>, String> {
    let matricula = raw
        .get("matricula")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let pis = raw.get("pis").and_then(Value::as_str).unwrap_or_default();
    let documento = raw
        .get("documento")
        .and_then(Value::as_str)
        .unwrap_or_default();

    conn.query_row(
        "SELECT id FROM funcionarios
         WHERE (?1 <> '' AND matricula = ?1)
            OR (?2 <> '' AND (pis = ?2 OR ltrim(pis,'0') = ltrim(?2,'0')))
            OR (?3 <> '' AND (documento = ?3 OR ltrim(documento,'0') = ltrim(?3,'0')))
         ORDER BY id ASC LIMIT 1",
        params![matricula, pis, documento],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| format!("Falha ao localizar funcionário no conector: {err}"))
}

#[command]
pub async fn conector_testar(state: State<'_, SharedState>) -> Result<String, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let client = build_client(&conn)?;
    client.testar_conexao().await
}

#[command]
pub async fn conector_coletar_batidas(
    state: State<'_, SharedState>,
    equipamento_id: i64,
    completo: Option<bool>,
    nsr_inicio: Option<i64>,
    nsr_fim: Option<i64>,
    data_inicio: Option<String>,
    data_fim: Option<String>,
) -> Result<Value, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let equipamento = conn
        .query_row(
            "SELECT id, COALESCE(usar_conector,0), COALESCE(conector_device_id,''), COALESCE(conector_ultimo_nsr,0)
             FROM equipamentos WHERE id = ?1 LIMIT 1",
            [equipamento_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)),
        )
        .optional()
        .map_err(|err| format!("Falha ao carregar equipamento: {err}"))?
        .ok_or_else(|| "Equipamento não encontrado.".to_string())?;

    if equipamento.1 == 0 {
        return Err("Equipamento configurado sem uso de conector (usar_conector=0).".to_string());
    }
    if equipamento.2.trim().is_empty() {
        return Err("Equipamento sem conector_device_id configurado.".to_string());
    }

    let nsr_start = if completo.unwrap_or(false) {
        None
    } else if nsr_inicio.is_some() {
        nsr_inicio
    } else {
        Some(equipamento.3 + 1)
    };

    let payload = json!({
        "completo": completo.unwrap_or(false),
        "nsr_inicio": nsr_start,
        "nsr_fim": nsr_fim,
        "data_inicio": data_inicio,
        "data_fim": data_fim,
    });

    let client = build_client(&conn)?;
    let resposta = match client.coletar_batidas(&equipamento.2, &payload).await {
        Ok(value) => value,
        Err(err) => {
            let result = json!({
                "equipamento_id": equipamento_id,
                "conector_device_id": equipamento.2,
                "status": "erro",
                "tipo": "batidas",
                "mensagem": err,
                "nsr_inicio": nsr_start,
                "nsr_fim": nsr_fim,
                "payload_json": payload,
            });
            let _ = log_coleta(&conn, &result);
            return Err(result
                .get("mensagem")
                .and_then(Value::as_str)
                .unwrap_or("Falha na coleta via conector.")
                .to_string());
        }
    };
    let punches = resposta
        .get("punches")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut total_importadas = 0i64;
    let mut total_duplicadas = 0i64;
    let mut max_nsr = equipamento.3;

    for item in punches.iter().filter_map(Value::as_object) {
        let funcionario_id = match find_funcionario(&conn, item)? {
            Some(v) => v,
            None => continue,
        };

        let nsr = item
            .get("nsr")
            .and_then(Value::as_i64)
            .map(|v| v.to_string())
            .or_else(|| {
                item.get("nsr")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            });

        let (data_referencia, hora) = if let (Some(data), Some(hora)) = (
            item.get("data_referencia").and_then(Value::as_str),
            item.get("hora").and_then(Value::as_str),
        ) {
            (data.to_string(), hora.to_string())
        } else if let Some(dt) = item.get("data_hora").and_then(Value::as_str) {
            parse_datetime(dt).unwrap_or_default()
        } else {
            (String::new(), String::new())
        };

        if data_referencia.is_empty() || hora.is_empty() {
            continue;
        }

        let duplicada = conn
            .query_row(
                "SELECT id FROM batidas WHERE funcionario_id = ?1 AND data_referencia = ?2 AND hora = ?3 AND COALESCE(nsr,'') = COALESCE(?4,'') LIMIT 1",
                params![funcionario_id, data_referencia, hora, nsr.clone()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| format!("Falha ao verificar duplicidade de batida: {err}"))?;

        if duplicada.is_some() {
            total_duplicadas += 1;
            continue;
        }

        conn.execute(
            "INSERT INTO batidas (funcionario_id, equipamento_id, data_referencia, hora, nsr, origem, tipo, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'conector', COALESCE(?6, 'entrada'), ?7, ?7)",
            params![
                funcionario_id,
                equipamento_id,
                data_referencia,
                hora,
                nsr,
                item.get("tipo").and_then(Value::as_str),
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|err| format!("Falha ao inserir batida via conector: {err}"))?;
        total_importadas += 1;

        if let Some(n) = item.get("nsr").and_then(Value::as_i64) {
            if n > max_nsr {
                max_nsr = n;
            }
        }
    }

    conn.execute(
        "UPDATE equipamentos SET conector_ultimo_nsr = ?1, conector_ultima_coleta_em = ?2, updated_at = ?2 WHERE id = ?3",
        params![max_nsr, Utc::now().to_rfc3339(), equipamento_id],
    )
    .map_err(|err| format!("Falha ao atualizar estado incremental do equipamento: {err}"))?;

    let result = json!({
        "equipamento_id": equipamento_id,
        "conector_device_id": equipamento.2,
        "status": "sucesso",
        "tipo": "batidas",
        "total_recebidas": punches.len(),
        "total_importadas": total_importadas,
        "total_duplicadas": total_duplicadas,
        "nsr_inicio": nsr_start,
        "nsr_fim": max_nsr,
        "payload_json": resposta,
    });

    log_coleta(&conn, &result)?;
    write_audit(
        &conn,
        "conector",
        "coletar_batidas",
        Some(equipamento_id),
        &result,
    )?;
    Ok(result)
}

#[command]
pub async fn conector_baixar_afd(
    state: State<'_, SharedState>,
    equipamento_id: i64,
) -> Result<Value, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let data_dir = state.data_dir()?;

    let (usar_conector, device_id): (i64, String) = conn
        .query_row(
            "SELECT COALESCE(usar_conector,0), COALESCE(conector_device_id,'') FROM equipamentos WHERE id = ?1 LIMIT 1",
            [equipamento_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|err| format!("Falha ao carregar equipamento para AFD via conector: {err}"))?
        .ok_or_else(|| "Equipamento não encontrado.".to_string())?;

    if usar_conector == 0 || device_id.trim().is_empty() {
        return Err("Equipamento sem configuração de conector ativa para AFD.".to_string());
    }

    let client = build_client(&conn)?;
    let bytes = match client.baixar_afd(&device_id, &json!({})).await {
        Ok(value) => value,
        Err(err) => {
            let result = json!({
                "equipamento_id": equipamento_id,
                "conector_device_id": device_id,
                "tipo": "afd",
                "status": "erro",
                "mensagem": err,
            });
            let _ = log_coleta(&conn, &result);
            return Err(result
                .get("mensagem")
                .and_then(Value::as_str)
                .unwrap_or("Falha ao baixar AFD via conector.")
                .to_string());
        }
    };
    let dir = data_dir
        .join("afd")
        .join(format!("equipamento_{equipamento_id}"));
    fs::create_dir_all(&dir).map_err(|err| format!("Falha ao criar diretório de AFD: {err}"))?;
    let arquivo = dir.join(format!("afd_{}.txt", Utc::now().format("%Y%m%d%H%M%S")));
    fs::write(&arquivo, &bytes).map_err(|err| format!("Falha ao salvar arquivo AFD: {err}"))?;

    let result = json!({
        "equipamento_id": equipamento_id,
        "conector_device_id": device_id,
        "tipo": "afd",
        "status": "sucesso",
        "arquivo_path": arquivo.to_string_lossy(),
        "total_recebidas": 1,
        "payload_json": {"bytes": bytes.len()},
    });
    log_coleta(&conn, &result)?;
    write_audit(
        &conn,
        "conector",
        "baixar_afd",
        Some(equipamento_id),
        &result,
    )?;
    Ok(result)
}

#[command]
pub fn conector_configuracao_carregar(state: State<'_, SharedState>) -> Result<Value, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    Ok(json!({
        "base_url": setting_value(&conn, SETTING_CONECTOR_URL)?
            .or_else(|| std::env::var("PONTO_CONECTOR_URL").ok())
            .unwrap_or_default(),
        "api_token_configurado": setting_value(&conn, SETTING_CONECTOR_TOKEN)?
            .or_else(|| std::env::var("PONTO_CONECTOR_TOKEN").ok())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false),
        "timeout_secs": setting_value(&conn, SETTING_CONECTOR_TIMEOUT)?
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30),
    }))
}

#[command]
pub fn conector_configuracao_salvar(
    state: State<'_, SharedState>,
    base_url: String,
    api_token: Option<String>,
    timeout_secs: Option<u64>,
) -> Result<Value, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let normalized_url = base_url.trim().trim_end_matches('/').to_string();
    if normalized_url.is_empty() {
        return Err("Informe a URL base da API do Ponto Manager Conector.".to_string());
    }
    save_setting(&conn, SETTING_CONECTOR_URL, &normalized_url)?;
    if let Some(token) = api_token {
        if !token.trim().is_empty() {
            save_setting(&conn, SETTING_CONECTOR_TOKEN, token.trim())?;
        }
    }
    let timeout = timeout_secs.unwrap_or(30).max(5);
    save_setting(&conn, SETTING_CONECTOR_TIMEOUT, &timeout.to_string())?;
    Ok(json!({
        "base_url": normalized_url,
        "api_token_configurado": setting_value(&conn, SETTING_CONECTOR_TOKEN)?.map(|value| !value.trim().is_empty()).unwrap_or(false),
        "timeout_secs": timeout,
    }))
}

#[command]
pub async fn conector_importar_afd(
    state: State<'_, SharedState>,
    args: ConectorImportarAfdArgs,
) -> Result<Value, String> {
    let ConectorImportarAfdArgs {
        empresa_id,
        equipamento_id,
        mode,
        completo,
        nsr_inicio,
        nsr_fim,
        data_inicio,
        data_fim,
    } = args;
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let data_dir = state.data_dir()?;

    let (usar_conector, device_id, ultimo_nsr): (i64, String, i64) = conn
        .query_row(
            "SELECT COALESCE(usar_conector,0), COALESCE(conector_device_id,''), COALESCE(conector_ultimo_nsr,0)
             FROM equipamentos WHERE id = ?1 LIMIT 1",
            [equipamento_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|err| format!("Falha ao carregar equipamento para importação AFD via conector: {err}"))?
        .ok_or_else(|| "Equipamento não encontrado.".to_string())?;

    if usar_conector == 0 || device_id.trim().is_empty() {
        return Err(
            "Equipamento sem configuração de conector ativa para importar AFD.".to_string(),
        );
    }

    let nsr_start = if completo.unwrap_or(false) {
        None
    } else if nsr_inicio.is_some() {
        nsr_inicio
    } else if ultimo_nsr > 0 {
        Some(ultimo_nsr + 1)
    } else {
        None
    };

    let request_payload = json!({
        "completo": completo.unwrap_or(false) || nsr_start.is_none(),
        "nsr_inicio": nsr_start,
        "nsr_fim": nsr_fim,
        "data_inicio": data_inicio,
        "data_fim": data_fim,
    });

    let client = build_client(&conn)?;
    let bytes = match client.baixar_afd(&device_id, &request_payload).await {
        Ok(value) => value,
        Err(err) => {
            let result = json!({
                "equipamento_id": equipamento_id,
                "conector_device_id": device_id,
                "tipo": "afd_importacao",
                "status": "erro",
                "mensagem": err,
                "nsr_inicio": nsr_start,
                "nsr_fim": nsr_fim,
                "payload_json": request_payload,
            });
            let _ = log_coleta(&conn, &result);
            return Err(result
                .get("mensagem")
                .and_then(Value::as_str)
                .unwrap_or("Falha ao baixar AFD via conector.")
                .to_string());
        }
    };

    let dir = data_dir
        .join("afd")
        .join(format!("equipamento_{equipamento_id}"));
    fs::create_dir_all(&dir).map_err(|err| format!("Falha ao criar diretório de AFD: {err}"))?;
    let file_name = format!(
        "afd_equipamento_{}_{}.txt",
        equipamento_id,
        Utc::now().format("%Y%m%d%H%M%S")
    );
    let arquivo = dir.join(&file_name);
    fs::write(&arquivo, &bytes)
        .map_err(|err| format!("Falha ao salvar arquivo AFD baixado do conector: {err}"))?;

    let content = String::from_utf8_lossy(&bytes).to_string();
    drop(conn);

    let import_result = afd_import_file(
        state,
        AfdImportRequest {
            empresa_id,
            equipamento_id: Some(equipamento_id),
            file_name: file_name.clone(),
            content,
            mode,
        },
    )?;

    let conn = open_connection(&db_path)?;
    let max_nsr = conn
        .query_row(
            "SELECT MAX(CAST(nsr AS INTEGER)) FROM afd_marcacoes WHERE importacao_id = ?1 AND nsr GLOB '[0-9]*'",
            [import_result.importacao_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar maior NSR importado do AFD: {err}"))?
        .flatten();

    if let Some(nsr) = max_nsr {
        if nsr > ultimo_nsr {
            conn.execute(
                "UPDATE equipamentos SET conector_ultimo_nsr = ?1, conector_ultima_coleta_em = ?2, updated_at = ?2 WHERE id = ?3",
                params![nsr, Utc::now().to_rfc3339(), equipamento_id],
            )
            .map_err(|err| format!("Falha ao atualizar último NSR após importação AFD: {err}"))?;
        }
    }

    let result = json!({
        "equipamento_id": equipamento_id,
        "conector_device_id": device_id,
        "tipo": "afd_importacao",
        "status": "sucesso",
        "arquivo_path": arquivo.to_string_lossy(),
        "total_recebidas": 1,
        "total_importadas": import_result.total_processadas,
        "total_duplicadas": import_result.total_descartadas,
        "nsr_inicio": nsr_start,
        "nsr_fim": max_nsr,
        "importacao": import_result,
        "payload_json": request_payload,
    });
    log_coleta(&conn, &result)?;
    Ok(result)
}

#[command]
pub fn conector_dashboard(state: State<'_, SharedState>) -> Result<Value, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let total_equipamentos: i64 = conn
        .query_row("SELECT COUNT(*) FROM equipamentos", [], |row| row.get(0))
        .unwrap_or(0);
    let total_conector: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM equipamentos WHERE COALESCE(usar_conector,0) = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_importadas: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_importadas),0) FROM conector_coletas_log WHERE tipo = 'batidas' AND status = 'sucesso'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_duplicadas: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_duplicadas),0) FROM conector_coletas_log WHERE tipo = 'batidas'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_afd: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM conector_coletas_log WHERE tipo = 'afd' AND status = 'sucesso'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_erros: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM conector_coletas_log WHERE status <> 'sucesso'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let ultima_coleta: Option<String> = conn
        .query_row(
            "SELECT MAX(created_at) FROM conector_coletas_log",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar última coleta do conector: {err}"))?
        .flatten();

    let mut stmt = conn
        .prepare(
            "SELECT e.id, e.codigo, e.descricao, e.modelo, e.ip, e.porta,
                    COALESCE(e.usar_conector,0), COALESCE(e.conector_device_id,''),
                    e.conector_ultimo_nsr, e.conector_ultima_coleta_em,
                    COALESCE(SUM(l.total_importadas),0), COALESCE(SUM(l.total_duplicadas),0),
                    MAX(l.created_at)
             FROM equipamentos e
             LEFT JOIN conector_coletas_log l ON l.equipamento_id = e.id
             GROUP BY e.id
             ORDER BY e.descricao ASC, e.id ASC",
        )
        .map_err(|err| format!("Falha ao preparar dashboard do conector: {err}"))?;

    let equipamentos = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "codigo": row.get::<_, Option<String>>(1)?,
                "descricao": row.get::<_, String>(2)?,
                "modelo": row.get::<_, Option<String>>(3)?,
                "ip": row.get::<_, Option<String>>(4)?,
                "porta": row.get::<_, Option<i64>>(5)?,
                "usar_conector": row.get::<_, i64>(6)? == 1,
                "conector_device_id": row.get::<_, String>(7)?,
                "conector_ultimo_nsr": row.get::<_, Option<i64>>(8)?,
                "conector_ultima_coleta_em": row.get::<_, Option<String>>(9)?,
                "total_importadas": row.get::<_, i64>(10)?,
                "total_duplicadas": row.get::<_, i64>(11)?,
                "ultima_execucao": row.get::<_, Option<String>>(12)?,
            }))
        })
        .map_err(|err| format!("Falha ao consultar equipamentos do dashboard: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear equipamentos do dashboard: {err}"))?;

    let mut stmt_logs = conn
        .prepare(
            "SELECT l.id, l.equipamento_id, COALESCE(e.descricao, 'Equipamento removido'),
                    l.conector_device_id, l.tipo, l.status, l.mensagem,
                    l.total_recebidas, l.total_importadas, l.total_duplicadas,
                    l.nsr_inicio, l.nsr_fim, l.arquivo_path, l.created_at
             FROM conector_coletas_log l
             LEFT JOIN equipamentos e ON e.id = l.equipamento_id
             ORDER BY l.id DESC
             LIMIT 50",
        )
        .map_err(|err| format!("Falha ao preparar logs do conector: {err}"))?;

    let logs = stmt_logs
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "equipamento_id": row.get::<_, i64>(1)?,
                "equipamento": row.get::<_, String>(2)?,
                "conector_device_id": row.get::<_, Option<String>>(3)?,
                "tipo": row.get::<_, String>(4)?,
                "status": row.get::<_, String>(5)?,
                "mensagem": row.get::<_, Option<String>>(6)?,
                "total_recebidas": row.get::<_, i64>(7)?,
                "total_importadas": row.get::<_, i64>(8)?,
                "total_duplicadas": row.get::<_, i64>(9)?,
                "nsr_inicio": row.get::<_, Option<i64>>(10)?,
                "nsr_fim": row.get::<_, Option<i64>>(11)?,
                "arquivo_path": row.get::<_, Option<String>>(12)?,
                "created_at": row.get::<_, String>(13)?,
            }))
        })
        .map_err(|err| format!("Falha ao consultar logs do conector: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("Falha ao mapear logs do conector: {err}"))?;

    Ok(json!({
        "totais": {
            "equipamentos": total_equipamentos,
            "equipamentos_conector": total_conector,
            "batidas_importadas": total_importadas,
            "batidas_duplicadas": total_duplicadas,
            "afd_baixados": total_afd,
            "erros": total_erros,
            "ultima_coleta": ultima_coleta,
        },
        "equipamentos": equipamentos,
        "logs": logs,
    }))
}

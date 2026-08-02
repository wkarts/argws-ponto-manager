use chrono::Utc;
use rusqlite::{params, params_from_iter, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::PunchFilters,
    punch_integrity::{
        carregar_snapshot, marcar_duplicidade, origem_oficial, reativar_duplicidade,
    },
};

fn json_to_sql_value(value: &Value) -> rusqlite::types::Value {
    match value {
        Value::Null => rusqlite::types::Value::Null,
        Value::Bool(v) => rusqlite::types::Value::Integer(if *v { 1 } else { 0 }),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = v.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        Value::String(v) if v.trim().is_empty() => rusqlite::types::Value::Null,
        Value::String(v) => rusqlite::types::Value::Text(v.to_string()),
        _ => rusqlite::types::Value::Text(value.to_string()),
    }
}

fn parse_id(payload: &Map<String, Value>, field: &str) -> Option<i64> {
    payload.get(field).and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
    })
}

fn parse_bool(payload: &Map<String, Value>, field: &str, default: bool) -> i64 {
    match payload.get(field) {
        Some(Value::Bool(v)) => {
            if *v {
                1
            } else {
                0
            }
        }
        Some(Value::Number(v)) => {
            if v.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Some(Value::String(v)) => {
            if matches!(v.trim(), "1" | "true" | "TRUE" | "sim" | "SIM") {
                1
            } else {
                0
            }
        }
        _ => {
            if default {
                1
            } else {
                0
            }
        }
    }
}

fn carregar_origem(conn: &rusqlite::Connection, id: i64) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT origem FROM batidas WHERE id = ?1 LIMIT 1",
        [id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|err| format!("Falha ao verificar origem da batida: {err}"))
}

fn impedir_alteracao_oficial(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    if carregar_origem(conn, id)?
        .as_deref()
        .is_some_and(origem_oficial)
    {
        return Err(
            "Marcações oficiais AFD/REP/Connector são imutáveis. Se houver repetição, marque a batida como duplicidade para preservar a rastreabilidade."
                .to_string(),
        );
    }
    Ok(())
}

fn registrar_batida_ignorada_por_ajuste(
    conn: &rusqlite::Connection,
    batida_id: i64,
    motivo: &str,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    let origem: Option<String> = conn
        .query_row(
            "SELECT origem FROM batidas WHERE id = ?1 LIMIT 1",
            params![batida_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar origem da batida ajustada: {err}"))?;

    let origem_normalizada = origem.clone().unwrap_or_default().to_lowercase();
    let deve_preservar_ignorada = origem_normalizada.contains("afd")
        || origem_normalizada.contains("conector")
        || origem_normalizada.contains("rep");

    if !deve_preservar_ignorada {
        return Ok(());
    }

    conn.execute(
        "INSERT OR IGNORE INTO batidas_ignoradas_afd (
            batida_id_origem, funcionario_id, equipamento_id, data_referencia, hora, nsr,
            origem, motivo, observacao, created_at, updated_at
         )
         SELECT id, funcionario_id, equipamento_id, data_referencia, hora, nsr,
                origem, ?2,
                'Batida removida/alterada manualmente para não ser recriada em nova importação completa do AFD.',
                ?3, ?3
           FROM batidas
          WHERE id = ?1",
        params![batida_id, motivo, now],
    )
    .map_err(|err| format!("Falha ao registrar batida ignorada por ajuste manual: {err}"))?;

    Ok(())
}

#[tauri::command]
pub fn batidas_list(
    state: State<'_, SharedState>,
    filters: PunchFilters,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = String::from(
        "SELECT b.id,
                b.funcionario_id,
                f.nome AS funcionario_nome,
                b.equipamento_id,
                COALESCE(e.descricao, '') AS equipamento_nome,
                b.justificativa_id,
                COALESCE(j.descricao, '') AS justificativa_nome,
                b.manual_ajuste,
                b.validado,
                b.data_referencia,
                b.hora,
                b.nsr,
                b.origem,
                b.observacao,
                b.tipo,
                COALESCE(b.ativo, 1) AS ativo,
                COALESCE(b.status, 'ativa') AS status,
                b.duplicada_de_id,
                b.inativada_em,
                b.inativada_motivo,
                b.reativada_em,
                CASE
                    WHEN LOWER(COALESCE(b.origem, '')) LIKE '%afd%'
                      OR LOWER(COALESCE(b.origem, '')) LIKE '%conector%'
                      OR LOWER(COALESCE(b.origem, '')) LIKE '%rep%'
                    THEN 1 ELSE 0
                END AS origem_protegida
         FROM batidas b
         INNER JOIN funcionarios f ON f.id = b.funcionario_id
         LEFT JOIN equipamentos e ON e.id = b.equipamento_id
         LEFT JOIN justificativas j ON j.id = b.justificativa_id
         WHERE 1 = 1",
    );

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if !filters.incluir_inativas {
        sql.push_str(" AND COALESCE(b.ativo, 1) = 1");
    }

    if let Some(empresa_id) = filters.empresa_id {
        sql.push_str(" AND f.empresa_id = ?");
        params.push(rusqlite::types::Value::Integer(empresa_id));
    }

    if let Some(funcionario_id) = filters.funcionario_id {
        sql.push_str(" AND b.funcionario_id = ?");
        params.push(rusqlite::types::Value::Integer(funcionario_id));
    }

    if let Some(data_inicial) = filters.data_inicial {
        sql.push_str(" AND b.data_referencia >= ?");
        params.push(rusqlite::types::Value::Text(data_inicial));
    }

    if let Some(data_final) = filters.data_final {
        sql.push_str(" AND b.data_referencia <= ?");
        params.push(rusqlite::types::Value::Text(data_final));
    }

    sql.push_str(" ORDER BY b.data_referencia DESC, b.hora DESC, b.id DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de batidas: {err}"))?;

    let mapped = stmt
        .query_map(params_from_iter(params.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de batidas: {err}"))?;

    let rows: Result<Vec<_>, _> = mapped.collect();
    rows.map_err(|err| format!("Falha ao mapear batidas: {err}"))
}

#[tauri::command]
pub fn batida_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();

    let funcionario_id = parse_id(&payload, "funcionario_id")
        .ok_or_else(|| "funcionario_id é obrigatório.".to_string())?;

    let data_referencia = payload
        .get("data_referencia")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "data_referencia é obrigatória.".to_string())?
        .to_string();

    let hora = payload
        .get("hora")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "hora é obrigatória.".to_string())?
        .to_string();

    let equipamento_id = parse_id(&payload, "equipamento_id");
    let justificativa_id = parse_id(&payload, "justificativa_id");
    let manual_ajuste = parse_bool(&payload, "manual_ajuste", false);
    let validado = parse_bool(&payload, "validado", true);
    let id = parse_id(&payload, "id");

    let origem_value = payload.get("origem").cloned().unwrap_or_else(|| {
        Value::String(if manual_ajuste == 1 {
            "ajuste_manual".to_string()
        } else {
            "manual".to_string()
        })
    });

    let values = vec![
        rusqlite::types::Value::Integer(funcionario_id),
        match equipamento_id {
            Some(v) => rusqlite::types::Value::Integer(v),
            None => rusqlite::types::Value::Null,
        },
        match justificativa_id {
            Some(v) => rusqlite::types::Value::Integer(v),
            None => rusqlite::types::Value::Null,
        },
        rusqlite::types::Value::Integer(manual_ajuste),
        rusqlite::types::Value::Integer(validado),
        rusqlite::types::Value::Text(data_referencia.clone()),
        rusqlite::types::Value::Text(hora.clone()),
        json_to_sql_value(payload.get("nsr").unwrap_or(&Value::Null)),
        json_to_sql_value(&origem_value),
        json_to_sql_value(payload.get("observacao").unwrap_or(&Value::Null)),
        json_to_sql_value(
            payload
                .get("tipo")
                .unwrap_or(&Value::String("entrada".to_string())),
        ),
    ];

    let record_id = if let Some(existing_id) = id {
        impedir_alteracao_oficial(&conn, existing_id)?;

        let assinatura_alterada: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM batidas
                  WHERE id = ?1
                    AND (funcionario_id <> ?2
                         OR data_referencia <> ?3
                         OR hora <> ?4
                         OR COALESCE(nsr, '') <> COALESCE(?5, ''))",
                params![
                    existing_id,
                    funcionario_id,
                    data_referencia,
                    hora,
                    payload.get("nsr").and_then(Value::as_str).unwrap_or("")
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| format!("Falha ao verificar alteração da batida original: {err}"))?
            .unwrap_or(0)
            > 0;

        if assinatura_alterada {
            registrar_batida_ignorada_por_ajuste(&conn, existing_id, "alteracao_manual")?;
        }

        let mut params = values.clone();
        params.push(rusqlite::types::Value::Text(now.clone()));
        params.push(rusqlite::types::Value::Integer(existing_id));

        conn.execute(
            "UPDATE batidas
             SET funcionario_id = ?, equipamento_id = ?, justificativa_id = ?, manual_ajuste = ?, validado = ?, data_referencia = ?, hora = ?, nsr = ?, origem = ?, observacao = ?, tipo = ?, updated_at = ?
             WHERE id = ?",
            params_from_iter(params.iter()),
        )
        .map_err(|err| format!("Falha ao atualizar batida: {err}"))?;

        existing_id
    } else {
        let mut params = values.clone();
        params.push(rusqlite::types::Value::Text(now.clone()));
        params.push(rusqlite::types::Value::Text(now.clone()));

        conn.execute(
            "INSERT INTO batidas
             (funcionario_id, equipamento_id, justificativa_id, manual_ajuste, validado, data_referencia, hora, nsr, origem, observacao, tipo, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params_from_iter(params.iter()),
        )
        .map_err(|err| format!("Falha ao inserir batida: {err}"))?;

        conn.last_insert_rowid()
    };

    let saved = conn
        .query_row(
            "SELECT b.id,
                    b.funcionario_id,
                    f.nome AS funcionario_nome,
                    b.equipamento_id,
                    COALESCE(e.descricao, '') AS equipamento_nome,
                    b.justificativa_id,
                    COALESCE(j.descricao, '') AS justificativa_nome,
                    b.manual_ajuste,
                    b.validado,
                    b.data_referencia,
                    b.hora,
                    b.nsr,
                    b.origem,
                    b.observacao,
                    b.tipo,
                    COALESCE(b.ativo, 1) AS ativo,
                    COALESCE(b.status, 'ativa') AS status,
                    b.duplicada_de_id,
                    b.inativada_em,
                    b.inativada_motivo,
                    b.reativada_em,
                    CASE
                        WHEN LOWER(COALESCE(b.origem, '')) LIKE '%afd%'
                          OR LOWER(COALESCE(b.origem, '')) LIKE '%conector%'
                          OR LOWER(COALESCE(b.origem, '')) LIKE '%rep%'
                        THEN 1 ELSE 0
                    END AS origem_protegida
             FROM batidas b
             INNER JOIN funcionarios f ON f.id = b.funcionario_id
             LEFT JOIN equipamentos e ON e.id = b.equipamento_id
             LEFT JOIN justificativas j ON j.id = b.justificativa_id
             WHERE b.id = ?1",
            [record_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler batida salva: {err}"))?
        .ok_or_else(|| "Batida salva não encontrada.".to_string())?;

    let action_name = if id.is_some() { "update" } else { "create" };
    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "batidas",
        action_name,
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "batidas",
        action_name,
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn batida_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let mut conn = open_connection(&db_path)?;
    let tx = conn
        .transaction()
        .map_err(|err| format!("Falha ao iniciar transação de exclusão de batida: {err}"))?;

    impedir_alteracao_oficial(&tx, id)?;

    registrar_batida_ignorada_por_ajuste(&tx, id, "exclusao_manual")?;

    tx.execute(
        "UPDATE afd_marcacoes SET batida_id = NULL WHERE batida_id = ?1",
        params![id],
    )
    .map_err(|err| format!("Falha ao desvincular marcações AFD da batida: {err}"))?;

    let affected = tx
        .execute("DELETE FROM batidas WHERE id = ?1", params![id])
        .map_err(|err| format!("Falha ao excluir batida: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id });
        write_audit(&tx, "batidas", "delete", Some(id), &payload)?;
        enqueue_sync(&tx, "batidas", "delete", Some(id), &payload)?;
    }

    tx.commit()
        .map_err(|err| format!("Falha ao concluir exclusão de batida: {err}"))?;

    Ok(affected > 0)
}

#[tauri::command]
pub fn batida_marcar_duplicidade(
    state: State<'_, SharedState>,
    id: i64,
    batida_principal_id: i64,
    motivo: Option<String>,
) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let mut conn = open_connection(&db_path)?;
    let tx = conn
        .transaction()
        .map_err(|err| format!("Falha ao iniciar tratamento da duplicidade: {err}"))?;
    let justificativa = motivo
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Duplicidade confirmada pelo usuário.");

    let changed = marcar_duplicidade(&tx, id, batida_principal_id, justificativa)?;
    if changed {
        let payload = json!({
            "id": id,
            "batida_principal_id": batida_principal_id,
            "motivo": justificativa,
            "exclusao_fisica": false,
        });
        write_audit(&tx, "batidas", "mark_duplicate", Some(id), &payload)?;
        enqueue_sync(&tx, "batidas", "mark_duplicate", Some(id), &payload)?;
    }
    tx.commit()
        .map_err(|err| format!("Falha ao concluir tratamento da duplicidade: {err}"))?;
    Ok(changed)
}

#[tauri::command]
pub fn batida_reativar(
    state: State<'_, SharedState>,
    id: i64,
    motivo: Option<String>,
) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let mut conn = open_connection(&db_path)?;
    let tx = conn
        .transaction()
        .map_err(|err| format!("Falha ao iniciar reativação da batida: {err}"))?;

    let before = carregar_snapshot(&tx, id)?
        .ok_or_else(|| "Batida selecionada para reativação não encontrada.".to_string())?;
    let justificativa = motivo
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Reativação confirmada pelo usuário.");
    let changed = reativar_duplicidade(&tx, id, justificativa)?;
    if changed {
        let payload = json!({
            "id": id,
            "funcionario_id": before.funcionario_id,
            "data_referencia": before.data_referencia,
            "motivo": justificativa,
        });
        write_audit(&tx, "batidas", "reactivate", Some(id), &payload)?;
        enqueue_sync(&tx, "batidas", "reactivate", Some(id), &payload)?;
    }
    tx.commit()
        .map_err(|err| format!("Falha ao concluir reativação da batida: {err}"))?;
    Ok(changed)
}

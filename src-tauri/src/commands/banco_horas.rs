use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::{
        ApuracaoRequest, BancoHorasAjusteRequest, BancoHorasProcessRequest,
        BancoHorasProcessResponse,
    },
    timecalc::resolve_schedule_for_employee,
};

use super::reports::apurar_periodo_internal;

#[tauri::command]
pub fn banco_horas_list(
    state: State<'_, SharedState>,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = String::from(
        "SELECT bh.id,
                bh.funcionario_id,
                f.nome AS funcionario_nome,
                bh.jornada_id,
                jt.descricao AS jornada_nome,
                bh.data_referencia,
                bh.minutos,
                bh.categoria,
                bh.classificacao,
                bh.origem,
                bh.referencia_id,
                bh.observacao,
                bh.created_at,
                bh.updated_at
         FROM banco_horas_lancamentos bh
         INNER JOIN funcionarios f ON f.id = bh.funcionario_id
         LEFT JOIN jornadas_trabalho jt ON jt.id = bh.jornada_id
         WHERE 1 = 1",
    );
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(Value::Number(number)) = filters.get("funcionarioId") {
        if let Some(funcionario_id) = number.as_i64() {
            sql.push_str(" AND bh.funcionario_id = ?");
            params_vec.push(rusqlite::types::Value::Integer(funcionario_id));
        }
    }

    if let Some(Value::String(date)) = filters.get("dataInicial") {
        if !date.trim().is_empty() {
            sql.push_str(" AND bh.data_referencia >= ?");
            params_vec.push(rusqlite::types::Value::Text(date.clone()));
        }
    }

    if let Some(Value::String(date)) = filters.get("dataFinal") {
        if !date.trim().is_empty() {
            sql.push_str(" AND bh.data_referencia <= ?");
            params_vec.push(rusqlite::types::Value::Text(date.clone()));
        }
    }

    sql.push_str(" ORDER BY bh.data_referencia DESC, f.nome ASC, bh.id DESC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem do banco de horas: {err}"))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter()),
            row_to_json_map,
        )
        .map_err(|err| format!("Falha ao consultar banco de horas: {err}"))?;
    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear banco de horas: {err}"))
}

#[tauri::command]
pub fn banco_horas_processar_periodo(
    state: State<'_, SharedState>,
    payload: BancoHorasProcessRequest,
) -> Result<BancoHorasProcessResponse, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let overwrite = payload.overwrite.unwrap_or(true);

    let apuracao = apurar_periodo_internal(
        &conn,
        &ApuracaoRequest {
            funcionario_id: payload.funcionario_id,
            data_inicial: Some(payload.data_inicial.clone()),
            data_final: Some(payload.data_final.clone()),
        },
    )?;

    if overwrite {
        conn.execute(
            "DELETE FROM banco_horas_lancamentos
              WHERE categoria = 'apuracao'
                AND data_referencia >= ?1
                AND data_referencia <= ?2
                AND (?3 IS NULL OR funcionario_id = ?3)",
            params![
                payload.data_inicial,
                payload.data_final,
                payload.funcionario_id
            ],
        )
        .map_err(|err| {
            format!("Falha ao limpar lançamentos anteriores do banco de horas: {err}")
        })?;
    }

    let mut dias_processados = 0usize;
    let mut total_creditos = 0i64;
    let mut total_debitos = 0i64;

    for row in &apuracao.rows {
        let schedule = resolve_schedule_for_employee(&conn, row.funcionario_id, &row.data)?;
        let classificacao = if row.saldo_minutos > 0 {
            total_creditos += row.saldo_minutos;
            "credito"
        } else if row.saldo_minutos < 0 {
            total_debitos += row.saldo_minutos.abs();
            "debito"
        } else {
            "neutro"
        };

        conn.execute(
            "INSERT INTO banco_horas_lancamentos (
                funcionario_id, jornada_id, data_referencia, minutos, categoria, classificacao,
                origem, referencia_id, observacao, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 'apuracao', ?5, 'motor_apuracao', NULL, ?6, ?7, ?7)",
            params![
                row.funcionario_id,
                schedule.jornada_id,
                row.data,
                row.saldo_minutos,
                classificacao,
                format!(
                    "Apuração automática. Esperado: {} min, trabalhado: {} min, atraso: {} min, extra: {} min.",
                    row.horario_esperado_minutos, row.trabalhado_minutos, row.atraso_minutos, row.extra_minutos
                ),
                now,
            ],
        )
        .map_err(|err| format!("Falha ao lançar banco de horas: {err}"))?;
        dias_processados += 1;
    }

    let response = BancoHorasProcessResponse {
        dias_processados,
        total_creditos_minutos: total_creditos,
        total_debitos_minutos: total_debitos,
        saldo_liquido_minutos: total_creditos - total_debitos,
    };

    let payload_value = json!({
        "funcionario_id": payload.funcionario_id,
        "data_inicial": payload.data_inicial,
        "data_final": payload.data_final,
        "dias_processados": response.dias_processados,
        "saldo_liquido_minutos": response.saldo_liquido_minutos,
    });
    write_audit(
        &conn,
        "banco_horas_lancamentos",
        "process",
        None,
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "banco_horas_lancamentos",
        "process",
        None,
        &payload_value,
    )?;

    Ok(response)
}

#[tauri::command]
pub fn banco_horas_salvar_ajuste(
    state: State<'_, SharedState>,
    payload: BancoHorasAjusteRequest,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();

    let funcionario_exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM funcionarios WHERE id = ?1 LIMIT 1",
            [payload.funcionario_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar funcionário do ajuste: {err}"))?;
    if funcionario_exists.is_none() {
        return Err(
            "Funcionário informado para o ajuste de banco de horas não existe.".to_string(),
        );
    }

    let classificacao = if payload.minutos > 0 {
        "credito"
    } else if payload.minutos < 0 {
        "debito"
    } else {
        "neutro"
    };

    conn.execute(
        "INSERT INTO banco_horas_lancamentos (
            funcionario_id, jornada_id, data_referencia, minutos, categoria, classificacao,
            origem, referencia_id, observacao, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, 'ajuste', ?5, 'manual', NULL, ?6, ?7, ?7)",
        params![
            payload.funcionario_id,
            payload.jornada_id,
            payload.data_referencia,
            payload.minutos,
            classificacao,
            payload.observacao,
            now,
        ],
    )
    .map_err(|err| format!("Falha ao gravar ajuste de banco de horas: {err}"))?;
    let id = conn.last_insert_rowid();

    let saved = conn
        .query_row(
            "SELECT bh.id,
                    bh.funcionario_id,
                    f.nome AS funcionario_nome,
                    bh.jornada_id,
                    jt.descricao AS jornada_nome,
                    bh.data_referencia,
                    bh.minutos,
                    bh.categoria,
                    bh.classificacao,
                    bh.origem,
                    bh.referencia_id,
                    bh.observacao,
                    bh.created_at,
                    bh.updated_at
             FROM banco_horas_lancamentos bh
             INNER JOIN funcionarios f ON f.id = bh.funcionario_id
             LEFT JOIN jornadas_trabalho jt ON jt.id = bh.jornada_id
             WHERE bh.id = ?1",
            [id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler ajuste de banco de horas: {err}"))?
        .ok_or_else(|| "Ajuste salvo não encontrado.".to_string())?;

    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "banco_horas_lancamentos",
        "create",
        Some(id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "banco_horas_lancamentos",
        "create",
        Some(id),
        &payload_value,
    )?;

    Ok(saved)
}

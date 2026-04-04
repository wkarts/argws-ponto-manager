use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::ComboOption,
};

type JornadaDiaSql = (
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
    i64,
);

fn get_string(payload: &Map<String, Value>, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|value| match value {
            Value::String(text) => Some(text.trim().to_string()),
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(flag) => Some(if *flag {
                "1".to_string()
            } else {
                "0".to_string()
            }),
            _ => None,
        })
        .filter(|value| !value.is_empty())
}

fn get_i64(payload: &Map<String, Value>, key: &str) -> Option<i64> {
    payload.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn get_bool(payload: &Map<String, Value>, key: &str, default: bool) -> i64 {
    match payload.get(key) {
        Some(Value::Bool(flag)) => {
            if *flag {
                1
            } else {
                0
            }
        }
        Some(Value::Number(number)) => {
            if number.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Some(Value::String(text)) => {
            if matches!(text.trim(), "1" | "true" | "TRUE" | "sim" | "SIM") {
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

fn get_id(payload: &Map<String, Value>) -> Option<i64> {
    payload.get("id").and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn day_payload_to_sql(day: &Value) -> Result<JornadaDiaSql, String> {
    let map = day
        .as_object()
        .ok_or_else(|| "Formato inválido dos dias da jornada.".to_string())?;

    let dia_semana = get_i64(map, "dia_semana")
        .ok_or_else(|| "Dia da semana não informado na jornada.".to_string())?;
    if !(1..=7).contains(&dia_semana) {
        return Err("Dia da semana deve estar entre 1 e 7.".to_string());
    }

    Ok((
        dia_semana,
        get_string(map, "entrada_1"),
        get_string(map, "saida_1"),
        get_string(map, "entrada_2"),
        get_string(map, "saida_2"),
        get_i64(map, "carga_prevista_minutos").unwrap_or(0),
        get_i64(map, "intervalo_minutos").unwrap_or(0),
        get_bool(map, "folga", false),
    ))
}


#[tauri::command]
pub fn jornada_preset_list() -> Result<Vec<Map<String, Value>>, String> {
    let presets = vec![
        json!({"codigo":"MERC-6X1-SEG","descricao":"Mercado 6x1 com folga móvel (base segunda)","tipo_jornada":"flexivel","perfil_flexivel":"mercado_6x1_folga_movel","permite_folga_movel":true,"permite_meia_folga":false,"dia_folga_base":1,"periodo_meia_folga":null,"carga_semanal_minutos":2640,"limite_diario_minutos":540,"dias":[
            {"dia_semana":1,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true},
            {"dia_semana":2,"entrada_1":"05:00","saida_1":"09:00","entrada_2":"10:00","saida_2":"15:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":3,"entrada_1":"05:00","saida_1":"09:00","entrada_2":"10:00","saida_2":"15:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":4,"entrada_1":"05:00","saida_1":"09:00","entrada_2":"10:00","saida_2":"15:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":5,"entrada_1":"05:00","saida_1":"09:00","entrada_2":"10:00","saida_2":"15:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":6,"entrada_1":"05:00","saida_1":"09:00","entrada_2":"10:00","saida_2":"15:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":7,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true}
        ]}),
        json!({"codigo":"MERC-MEIA","descricao":"Mercado com meia folga variável","tipo_jornada":"flexivel","perfil_flexivel":"mercado_meia_folga","permite_folga_movel":true,"permite_meia_folga":true,"dia_folga_base":3,"periodo_meia_folga":"tarde","carga_semanal_minutos":2640,"limite_diario_minutos":540,"dias":[
            {"dia_semana":1,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":2,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":3,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"","saida_2":"","carga_prevista_minutos":300,"intervalo_minutos":0,"folga":false},
            {"dia_semana":4,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":5,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":6,"entrada_1":"07:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":540,"intervalo_minutos":60,"folga":false},
            {"dia_semana":7,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true}
        ]}),
        json!({"codigo":"DIARISTA-2D","descricao":"Diarista 2 dias por semana","tipo_jornada":"diarista","perfil_flexivel":"diarista_2_dias","permite_folga_movel":false,"permite_meia_folga":false,"dia_folga_base":null,"periodo_meia_folga":null,"carga_semanal_minutos":960,"limite_diario_minutos":480,"dias":[
            {"dia_semana":1,"entrada_1":"08:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":480,"intervalo_minutos":60,"folga":false},
            {"dia_semana":2,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true},
            {"dia_semana":3,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true},
            {"dia_semana":4,"entrada_1":"08:00","saida_1":"12:00","entrada_2":"13:00","saida_2":"17:00","carga_prevista_minutos":480,"intervalo_minutos":60,"folga":false},
            {"dia_semana":5,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true},
            {"dia_semana":6,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true},
            {"dia_semana":7,"entrada_1":"","saida_1":"","entrada_2":"","saida_2":"","carga_prevista_minutos":0,"intervalo_minutos":0,"folga":true}
        ]}),
    ];

    Ok(presets
        .into_iter()
        .filter_map(|item| item.as_object().cloned())
        .collect())
}

#[tauri::command]
pub fn jornada_combo_list(state: State<'_, SharedState>) -> Result<Vec<ComboOption>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, descricao FROM jornadas_trabalho WHERE ativo = 1 ORDER BY descricao ASC",
        )
        .map_err(|err| format!("Falha ao preparar combo de jornadas: {err}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ComboOption {
                id: row.get(0)?,
                label: row.get(1)?,
            })
        })
        .map_err(|err| format!("Falha ao executar combo de jornadas: {err}"))?;

    let result: Result<Vec<_>, _> = rows.collect();
    result.map_err(|err| format!("Falha ao mapear combo de jornadas: {err}"))
}

#[tauri::command]
pub fn jornada_list(state: State<'_, SharedState>) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT jt.id,
                    jt.empresa_id,
                    e.nome AS empresa_nome,
                    jt.codigo,
                    jt.descricao,
                    jt.tipo_jornada,
                    jt.perfil_flexivel,
                    jt.permite_folga_movel,
                    jt.permite_meia_folga,
                    jt.dia_folga_base,
                    jt.periodo_meia_folga,
                    jt.heuristica_troca_folga,
                    jt.tolerancia_entrada_minutos,
                    jt.tolerancia_saida_minutos,
                    jt.tolerancia_intervalo_minutos,
                    jt.carga_semanal_minutos,
                    jt.limite_diario_minutos,
                    jt.banco_horas_ativo,
                    jt.exige_marcacao_intervalo,
                    jt.compensa_atraso_com_extra,
                    jt.modo_tratamento_afd,
                    jt.observacoes,
                    jt.ativo,
                    jt.created_at,
                    jt.updated_at,
                    (SELECT COUNT(*) FROM jornada_dias jd WHERE jd.jornada_id = jt.id) AS total_dias
             FROM jornadas_trabalho jt
             LEFT JOIN empresas e ON e.id = jt.empresa_id
             ORDER BY jt.descricao ASC",
        )
        .map_err(|err| format!("Falha ao preparar listagem de jornadas: {err}"))?;

    let rows = stmt
        .query_map([], row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de jornadas: {err}"))?;

    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear jornadas: {err}"))
}

#[tauri::command]
pub fn jornada_get(state: State<'_, SharedState>, id: i64) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut item = conn
        .query_row(
            "SELECT jt.id,
                    jt.empresa_id,
                    jt.codigo,
                    jt.descricao,
                    jt.tipo_jornada,
                    jt.perfil_flexivel,
                    jt.permite_folga_movel,
                    jt.permite_meia_folga,
                    jt.dia_folga_base,
                    jt.periodo_meia_folga,
                    jt.heuristica_troca_folga,
                    jt.tolerancia_entrada_minutos,
                    jt.tolerancia_saida_minutos,
                    jt.tolerancia_intervalo_minutos,
                    jt.carga_semanal_minutos,
                    jt.limite_diario_minutos,
                    jt.banco_horas_ativo,
                    jt.exige_marcacao_intervalo,
                    jt.compensa_atraso_com_extra,
                    jt.modo_tratamento_afd,
                    jt.observacoes,
                    jt.ativo,
                    jt.created_at,
                    jt.updated_at
             FROM jornadas_trabalho jt
             WHERE jt.id = ?1",
            [id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar jornada: {err}"))?
        .ok_or_else(|| "Jornada não encontrada.".to_string())?;

    let mut dias_stmt = conn
        .prepare(
            "SELECT id, jornada_id, dia_semana, entrada_1, saida_1, entrada_2, saida_2, carga_prevista_minutos, intervalo_minutos, folga
             FROM jornada_dias
             WHERE jornada_id = ?1
             ORDER BY dia_semana ASC"
        )
        .map_err(|err| format!("Falha ao preparar dias da jornada: {err}"))?;

    let dias = dias_stmt
        .query_map([id], row_to_json_map)
        .map_err(|err| format!("Falha ao consultar dias da jornada: {err}"))?;
    let dias: Result<Vec<_>, _> = dias.collect();
    item.insert(
        "dias".to_string(),
        Value::Array(
            dias.map_err(|err| format!("Falha ao mapear dias da jornada: {err}"))?
                .into_iter()
                .map(Value::Object)
                .collect(),
        ),
    );

    Ok(item)
}

#[tauri::command]
pub fn jornada_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let id = get_id(&payload);

    let descricao = get_string(&payload, "descricao")
        .ok_or_else(|| "Informe a descrição da jornada.".to_string())?;
    let empresa_id = get_i64(&payload, "empresa_id");
    let tipo_jornada = get_string(&payload, "tipo_jornada").unwrap_or_else(|| "fixa".to_string());
    let modo_tratamento_afd =
        get_string(&payload, "modo_tratamento_afd").unwrap_or_else(|| "auto".to_string());
    let dias_raw = payload.get("dias").cloned().unwrap_or(Value::Array(vec![]));
    let dias_array = dias_raw
        .as_array()
        .ok_or_else(|| "Os dias da jornada devem ser enviados em formato de lista.".to_string())?;

    if dias_array.is_empty() {
        return Err("Informe ao menos um dia para a jornada de trabalho.".to_string());
    }

    let duplicate: Option<i64> = conn
        .query_row(
            "SELECT id FROM jornadas_trabalho WHERE descricao = ?1 AND (?2 IS NULL OR id <> ?2) LIMIT 1",
            params![descricao, id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar duplicidade da jornada: {err}"))?;

    if duplicate.is_some() {
        return Err("Já existe uma jornada cadastrada com esta descrição.".to_string());
    }

    let record_id = if let Some(existing_id) = id {
        conn.execute(
            "UPDATE jornadas_trabalho
                SET empresa_id = ?1,
                    codigo = ?2,
                    descricao = ?3,
                    tipo_jornada = ?4,
                    perfil_flexivel = ?5,
                    permite_folga_movel = ?6,
                    permite_meia_folga = ?7,
                    dia_folga_base = ?8,
                    periodo_meia_folga = ?9,
                    heuristica_troca_folga = ?10,
                    tolerancia_entrada_minutos = ?11,
                    tolerancia_saida_minutos = ?12,
                    tolerancia_intervalo_minutos = ?13,
                    carga_semanal_minutos = ?14,
                    limite_diario_minutos = ?15,
                    banco_horas_ativo = ?16,
                    exige_marcacao_intervalo = ?17,
                    compensa_atraso_com_extra = ?18,
                    modo_tratamento_afd = ?19,
                    observacoes = ?20,
                    ativo = ?21,
                    updated_at = ?22
              WHERE id = ?23",
            params![
                empresa_id,
                get_string(&payload, "codigo"),
                descricao,
                tipo_jornada,
                get_string(&payload, "perfil_flexivel"),
                get_bool(&payload, "permite_folga_movel", false),
                get_bool(&payload, "permite_meia_folga", false),
                get_i64(&payload, "dia_folga_base"),
                get_string(&payload, "periodo_meia_folga"),
                get_bool(&payload, "heuristica_troca_folga", true),
                get_i64(&payload, "tolerancia_entrada_minutos").unwrap_or(5),
                get_i64(&payload, "tolerancia_saida_minutos").unwrap_or(5),
                get_i64(&payload, "tolerancia_intervalo_minutos").unwrap_or(5),
                get_i64(&payload, "carga_semanal_minutos").unwrap_or(2640),
                get_i64(&payload, "limite_diario_minutos").unwrap_or(600),
                get_bool(&payload, "banco_horas_ativo", true),
                get_bool(&payload, "exige_marcacao_intervalo", true),
                get_bool(&payload, "compensa_atraso_com_extra", true),
                modo_tratamento_afd,
                get_string(&payload, "observacoes"),
                get_bool(&payload, "ativo", true),
                now,
                existing_id,
            ],
        )
        .map_err(|err| format!("Falha ao atualizar jornada: {err}"))?;
        conn.execute(
            "DELETE FROM jornada_dias WHERE jornada_id = ?1",
            [existing_id],
        )
        .map_err(|err| format!("Falha ao limpar dias anteriores da jornada: {err}"))?;
        existing_id
    } else {
        conn.execute(
            "INSERT INTO jornadas_trabalho (
                empresa_id, codigo, descricao, tipo_jornada, perfil_flexivel, permite_folga_movel,
                permite_meia_folga, dia_folga_base, periodo_meia_folga, heuristica_troca_folga,
                tolerancia_entrada_minutos, tolerancia_saida_minutos, tolerancia_intervalo_minutos, carga_semanal_minutos,
                limite_diario_minutos, banco_horas_ativo, exige_marcacao_intervalo,
                compensa_atraso_com_extra, modo_tratamento_afd, observacoes, ativo, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?22
            )",
            params![
                empresa_id,
                get_string(&payload, "codigo"),
                descricao,
                tipo_jornada,
                get_string(&payload, "perfil_flexivel"),
                get_bool(&payload, "permite_folga_movel", false),
                get_bool(&payload, "permite_meia_folga", false),
                get_i64(&payload, "dia_folga_base"),
                get_string(&payload, "periodo_meia_folga"),
                get_bool(&payload, "heuristica_troca_folga", true),
                get_i64(&payload, "tolerancia_entrada_minutos").unwrap_or(5),
                get_i64(&payload, "tolerancia_saida_minutos").unwrap_or(5),
                get_i64(&payload, "tolerancia_intervalo_minutos").unwrap_or(5),
                get_i64(&payload, "carga_semanal_minutos").unwrap_or(2640),
                get_i64(&payload, "limite_diario_minutos").unwrap_or(600),
                get_bool(&payload, "banco_horas_ativo", true),
                get_bool(&payload, "exige_marcacao_intervalo", true),
                get_bool(&payload, "compensa_atraso_com_extra", true),
                modo_tratamento_afd,
                get_string(&payload, "observacoes"),
                get_bool(&payload, "ativo", true),
                now,
            ],
        )
        .map_err(|err| format!("Falha ao inserir jornada: {err}"))?;
        conn.last_insert_rowid()
    };

    for day in dias_array {
        let (
            dia_semana,
            entrada_1,
            saida_1,
            entrada_2,
            saida_2,
            carga_prevista_minutos,
            intervalo_minutos,
            folga,
        ) = day_payload_to_sql(day)?;
        conn.execute(
            "INSERT INTO jornada_dias (
                jornada_id, dia_semana, entrada_1, saida_1, entrada_2, saida_2,
                carga_prevista_minutos, intervalo_minutos, folga, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                record_id,
                dia_semana,
                entrada_1,
                saida_1,
                entrada_2,
                saida_2,
                carga_prevista_minutos,
                intervalo_minutos,
                folga,
                now
            ],
        )
        .map_err(|err| format!("Falha ao gravar dia da jornada: {err}"))?;
    }

    let saved = jornada_get(state, record_id)?;
    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "jornadas_trabalho",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "jornadas_trabalho",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}


#[tauri::command]
pub fn jornada_clone(state: State<'_, SharedState>, id: i64) -> Result<Map<String, Value>, String> {
    let mut base = jornada_get(state.clone(), id)?;
    base.remove("id");
    let descricao = base.get("descricao").and_then(|v| v.as_str()).unwrap_or("Jornada");
    base.insert("codigo".to_string(), Value::String(format!("{}-COPIA", base.get("codigo").and_then(|v| v.as_str()).unwrap_or("JOR"))));
    base.insert("descricao".to_string(), Value::String(format!("{} (Cópia)", descricao)));
    jornada_save(state, base)
}

#[tauri::command]
pub fn jornada_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let employee_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM funcionarios WHERE jornada_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar uso da jornada: {err}"))?;

    if employee_count > 0 {
        return Err(
            "Não é possível excluir a jornada porque existem funcionários vinculados.".to_string(),
        );
    }

    let bank_hours_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM banco_horas_lancamentos WHERE jornada_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar lançamentos da jornada: {err}"))?;

    if bank_hours_count > 0 {
        return Err(
            "Não é possível excluir a jornada porque existem lançamentos de banco de horas vinculados.".to_string(),
        );
    }

    let affected = conn
        .execute("DELETE FROM jornadas_trabalho WHERE id = ?1", [id])
        .map_err(|err| format!("Falha ao excluir jornada: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id, "entity": "jornadas_trabalho" });
        write_audit(&conn, "jornadas_trabalho", "delete", Some(id), &payload)?;
        enqueue_sync(&conn, "jornadas_trabalho", "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

use chrono::Utc;
use rusqlite::{params_from_iter, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::ComboOption,
    security::hash_password,
};

struct EntityDefinition {
    table: &'static str,
    fields: &'static [&'static str],
    searchable: &'static [&'static str],
    required: &'static [&'static str],
    label_column: &'static str,
}

fn entity_definition(entity: &str) -> Option<EntityDefinition> {
    match entity {
        "empresas" => Some(EntityDefinition {
            table: "empresas",
            fields: &[
                "nome",
                "documento",
                "telefone",
                "email",
                "endereco",
                "bairro",
                "cidade",
                "estado",
                "ativo",
            ],
            searchable: &["nome", "documento", "cidade"],
            required: &["nome"],
            label_column: "nome",
        }),
        "usuarios" => Some(EntityDefinition {
            table: "usuarios",
            fields: &["nome", "login", "senha_hash", "administrador", "ativo"],
            searchable: &["nome", "login"],
            required: &["nome", "login"],
            label_column: "nome",
        }),
        "departamentos" => Some(EntityDefinition {
            table: "departamentos",
            fields: &["descricao", "ativo"],
            searchable: &["descricao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "funcoes" => Some(EntityDefinition {
            table: "funcoes",
            fields: &["descricao", "ativo"],
            searchable: &["descricao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "centro_custos" => Some(EntityDefinition {
            table: "centro_custos",
            fields: &["codigo", "descricao", "ativo"],
            searchable: &["codigo", "descricao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "horarios" => Some(EntityDefinition {
            table: "horarios",
            fields: &[
                "numero",
                "descricao",
                "entrada_1",
                "saida_1",
                "entrada_2",
                "saida_2",
                "carga_horaria_minutos",
                "ativo",
            ],
            searchable: &["descricao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "escalas" => Some(EntityDefinition {
            table: "escalas",
            fields: &[
                "descricao",
                "horario_id",
                "dias_ativos",
                "tolerancia_minutos",
                "ativo",
            ],
            searchable: &["descricao", "dias_ativos"],
            required: &["descricao"],
            label_column: "descricao",
        }),

        "feriados" => Some(EntityDefinition {
            table: "feriados",
            fields: &[
                "data",
                "descricao",
                "contexto_tipo",
                "empresa_id",
                "departamento_id",
                "regra_jornada",
                "regra_compensacao",
                "observacoes",
                "ativo",
            ],
            searchable: &["data", "descricao", "contexto_tipo", "regra_compensacao"],
            required: &["data", "descricao"],
            label_column: "descricao",
        }),
        "jornada_contextos_regras" => Some(EntityDefinition {
            table: "jornada_contextos_regras",
            fields: &[
                "descricao",
                "empresa_id",
                "departamento_id",
                "funcao_id",
                "centro_custo_id",
                "jornada_id",
                "regra_compensacao",
                "banco_horas_ativo",
                "permite_hora_extra",
                "compensa_atraso_com_extra",
                "usa_banco_para_excedente",
                "ativo",
            ],
            searchable: &["descricao", "regra_compensacao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "equipamentos" => Some(EntityDefinition {
            table: "equipamentos",
            fields: &[
                "empresa_id",
                "codigo",
                "descricao",
                "modelo",
                "ip",
                "porta",
                "usar_conector",
                "conector_device_id",
                "conector_base_url",
                "conector_api_token",
                "conector_timeout",
                "conector_ultimo_nsr",
                "ativo",
            ],
            searchable: &["codigo", "descricao", "modelo", "ip"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "eventos" => Some(EntityDefinition {
            table: "eventos",
            fields: &[
                "codigo",
                "descricao",
                "tipo",
                "impacta_banco_horas",
                "ativo",
            ],
            searchable: &["codigo", "descricao", "tipo"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "justificativas" => Some(EntityDefinition {
            table: "justificativas",
            fields: &["descricao", "abono", "ativo"],
            searchable: &["descricao"],
            required: &["descricao"],
            label_column: "descricao",
        }),
        "funcionarios" => Some(EntityDefinition {
            table: "funcionarios",
            fields: &[
                "empresa_id",
                "matricula",
                "nome",
                "documento",
                "pis",
                "email",
                "telefone",
                "departamento_id",
                "funcao_id",
                "centro_custo_id",
                "horario_id",
                "escala_id",
                "data_admissao",
                "ativo",
            ],
            searchable: &["matricula", "nome", "documento"],
            required: &["nome"],
            label_column: "nome",
        }),
        _ => None,
    }
}

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

fn normalize_value(payload: &Map<String, Value>, field: &str) -> Value {
    let value = payload.get(field).cloned().unwrap_or(Value::Null);

    match field {
        "empresa_id"
        | "departamento_id"
        | "funcao_id"
        | "centro_custo_id"
        | "horario_id"
        | "escala_id"
        | "jornada_id"
        | "numero"
        | "porta"
        | "carga_horaria_minutos"
        | "tolerancia_minutos"
        | "conector_timeout"
        | "conector_ultimo_nsr"
        | "ativo"
        | "administrador"
        | "impacta_banco_horas"
        | "abono"
        | "banco_horas_ativo"
        | "permite_hora_extra"
        | "compensa_atraso_com_extra"
        | "usa_banco_para_excedente" => match value {
            Value::String(v) if v.trim().is_empty() => Value::Null,
            Value::String(v) => v.parse::<i64>().map(Value::from).unwrap_or(Value::Null),
            other => other,
        },
        _ => value,
    }
}

#[tauri::command]
pub fn entity_list(
    state: State<'_, SharedState>,
    entity: String,
    search: String,
) -> Result<Vec<Map<String, Value>>, String> {
    let definition =
        entity_definition(&entity).ok_or_else(|| "Entidade não permitida.".to_string())?;
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let mut sql = format!(
        "SELECT id, {} FROM {}",
        definition.fields.join(", "),
        definition.table
    );
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if !search.trim().is_empty() {
        let clauses = definition
            .searchable
            .iter()
            .map(|column| format!("{} LIKE ?", column))
            .collect::<Vec<_>>()
            .join(" OR ");
        sql.push_str(&format!(" WHERE ({clauses})"));
        for _ in definition.searchable {
            params.push(rusqlite::types::Value::Text(format!("%{}%", search.trim())));
        }
    }

    sql.push_str(" ORDER BY id DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de {}: {err}", definition.table))?;

    let mapped = stmt
        .query_map(params_from_iter(params.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de {}: {err}", definition.table))?;

    let rows: Result<Vec<_>, _> = mapped.collect();
    rows.map_err(|err| format!("Falha ao mapear listagem de {}: {err}", definition.table))
}

#[tauri::command]
pub fn combo_list(
    state: State<'_, SharedState>,
    entity: String,
) -> Result<Vec<ComboOption>, String> {
    if entity == "contextos_feriado" {
        return Ok(vec![
            ComboOption {
                id: 1,
                label: "global".to_string(),
            },
            ComboOption {
                id: 2,
                label: "empresa".to_string(),
            },
            ComboOption {
                id: 3,
                label: "departamento".to_string(),
            },
            ComboOption {
                id: 4,
                label: "operacional".to_string(),
            },
        ]);
    }
    if entity == "regras_jornada" {
        return Ok(vec![
            ComboOption {
                id: 1,
                label: "normal".to_string(),
            },
            ComboOption {
                id: 2,
                label: "reduzida".to_string(),
            },
            ComboOption {
                id: 3,
                label: "especial".to_string(),
            },
            ComboOption {
                id: 4,
                label: "escala diferenciada".to_string(),
            },
        ]);
    }
    if entity == "regras_compensacao" {
        return Ok(vec![
            ComboOption {
                id: 1,
                label: "banco_horas".to_string(),
            },
            ComboOption {
                id: 2,
                label: "hora_extra".to_string(),
            },
            ComboOption {
                id: 3,
                label: "consumo_horas".to_string(),
            },
            ComboOption {
                id: 4,
                label: "sem_compensacao".to_string(),
            },
        ]);
    }
    if entity == "jornadas_lookup" {
        let db_path = state.db_path()?;
        let conn = open_connection(&db_path)?;
        let mut stmt = conn
            .prepare("SELECT id, descricao AS label FROM jornadas_trabalho WHERE ativo = 1 ORDER BY descricao ASC")
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
        return result.map_err(|err| format!("Falha ao mapear combo de jornadas: {err}"));
    }

    let definition =
        entity_definition(&entity).ok_or_else(|| "Entidade não permitida.".to_string())?;
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let sql = format!(
        "SELECT id, COALESCE({}, '[sem descrição]') AS label FROM {} ORDER BY label ASC",
        definition.label_column, definition.table
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar combo de {}: {err}", definition.table))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ComboOption {
                id: row.get(0)?,
                label: row.get(1)?,
            })
        })
        .map_err(|err| format!("Falha ao executar combo de {}: {err}", definition.table))?;

    let result: Result<Vec<_>, _> = rows.collect();
    result.map_err(|err| format!("Falha ao mapear combo de {}: {err}", definition.table))
}

#[tauri::command]
pub fn entity_save(
    state: State<'_, SharedState>,
    entity: String,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let definition =
        entity_definition(&entity).ok_or_else(|| "Entidade não permitida.".to_string())?;
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let id = payload.get("id").and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
    });

    for required in definition.required {
        let raw = payload
            .get(*required)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if raw.is_empty() {
            return Err(format!("O campo {} é obrigatório.", required));
        }
    }

    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    for field in definition.fields {
        if entity == "usuarios" && *field == "senha_hash" {
            if let Some(password) = payload.get("senha").and_then(|v| v.as_str()) {
                if !password.trim().is_empty() {
                    columns.push((*field).to_string());
                    values.push(Value::String(hash_password(password)?));
                } else if id.is_none() {
                    return Err("Senha é obrigatória para novo usuário.".to_string());
                }
            } else if id.is_none() {
                return Err("Senha é obrigatória para novo usuário.".to_string());
            }
            continue;
        }

        columns.push((*field).to_string());
        values.push(normalize_value(&payload, field));
    }

    let record_id = if let Some(existing_id) = id {
        let set_clause = columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .chain(std::iter::once("updated_at = ?".to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            definition.table, set_clause
        );
        let mut sql_values: Vec<rusqlite::types::Value> =
            values.iter().map(json_to_sql_value).collect();
        sql_values.push(rusqlite::types::Value::Text(now.clone()));
        sql_values.push(rusqlite::types::Value::Integer(existing_id));

        conn.execute(&sql, params_from_iter(sql_values.iter()))
            .map_err(|err| format!("Falha ao atualizar {}: {err}", definition.table))?;
        existing_id
    } else {
        let mut insert_columns = columns.clone();
        insert_columns.push("created_at".to_string());
        insert_columns.push("updated_at".to_string());

        let placeholders = std::iter::repeat_n("?", insert_columns.len())
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            definition.table,
            insert_columns.join(", "),
            placeholders
        );

        let mut sql_values: Vec<rusqlite::types::Value> =
            values.iter().map(json_to_sql_value).collect();
        sql_values.push(rusqlite::types::Value::Text(now.clone()));
        sql_values.push(rusqlite::types::Value::Text(now.clone()));

        conn.execute(&sql, params_from_iter(sql_values.iter()))
            .map_err(|err| format!("Falha ao inserir em {}: {err}", definition.table))?;
        conn.last_insert_rowid()
    };

    let select_sql = format!(
        "SELECT id, {} FROM {} WHERE id = ?1",
        definition.fields.join(", "),
        definition.table
    );
    let saved = conn
        .query_row(&select_sql, [record_id], row_to_json_map)
        .optional()
        .map_err(|err| {
            format!(
                "Falha ao reler registro salvo em {}: {err}",
                definition.table
            )
        })?
        .ok_or_else(|| "Registro salvo não encontrado.".to_string())?;

    let action_name = if id.is_some() { "update" } else { "create" };
    let payload_value = Value::Object(saved.clone());
    write_audit(&conn, &entity, action_name, Some(record_id), &payload_value)?;
    enqueue_sync(&conn, &entity, action_name, Some(record_id), &payload_value)?;

    Ok(saved)
}

#[tauri::command]
pub fn entity_delete(
    state: State<'_, SharedState>,
    entity: String,
    id: i64,
) -> Result<bool, String> {
    let definition =
        entity_definition(&entity).ok_or_else(|| "Entidade não permitida.".to_string())?;
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let sql = format!("DELETE FROM {} WHERE id = ?1", definition.table);
    let affected = conn
        .execute(&sql, [id])
        .map_err(|err| format!("Falha ao excluir de {}: {err}", definition.table))?;

    if affected > 0 {
        let payload = json!({ "id": id, "entity": entity });
        write_audit(&conn, &entity, "delete", Some(id), &payload)?;
        enqueue_sync(&conn, &entity, "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
};

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

fn get_i64(payload: &Map<String, Value>, key: &str) -> Option<i64> {
    payload.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn normalize_upper_uf(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.to_uppercase())
        .filter(|text| !text.is_empty())
}

fn validate_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.contains('@') && email.contains('.')
}

fn validate_iso_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

fn normalize_cpf(value: &str) -> Option<String> {
    let digits = only_digits(value);
    if digits.len() == 11 {
        return Some(digits);
    }

    if digits.len() > 11 {
        let extra = digits.len() - 11;
        if digits.chars().take(extra).all(|ch| ch == '0') {
            return Some(digits[extra..].to_string());
        }
    }

    None
}

fn is_valid_cpf(value: &str) -> bool {
    let Some(digits) = normalize_cpf(value) else {
        return false;
    };

    let bytes = digits.as_bytes();
    if bytes.iter().all(|b| *b == bytes[0]) {
        return false;
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    let sum1: u32 = nums
        .iter()
        .take(9)
        .enumerate()
        .map(|(idx, n)| n * (10 - idx as u32))
        .sum();
    let rem1 = sum1 % 11;
    let dv1 = if rem1 < 2 { 0 } else { 11 - rem1 };
    if nums[9] != dv1 {
        return false;
    }

    let sum2: u32 = nums
        .iter()
        .take(10)
        .enumerate()
        .map(|(idx, n)| n * (11 - idx as u32))
        .sum();
    let rem2 = sum2 % 11;
    let dv2 = if rem2 < 2 { 0 } else { 11 - rem2 };
    nums[10] == dv2
}

fn is_valid_pis(value: &str) -> bool {
    let digits = only_digits(value);
    if digits.len() != 11 {
        return false;
    }

    let bytes = digits.as_bytes();
    if bytes.iter().all(|b| *b == bytes[0]) {
        return false;
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    let weights = [3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum: u32 = nums
        .iter()
        .take(10)
        .zip(weights.iter())
        .map(|(n, w)| n * w)
        .sum();
    let remainder = 11 - (sum % 11);
    let check_digit = if remainder == 10 || remainder == 11 {
        0
    } else {
        remainder
    };
    nums[10] == check_digit
}

fn employee_select_sql() -> &'static str {
    "SELECT f.id,
            f.empresa_id,
            e.nome AS empresa_nome,
            f.matricula,
            f.nome,
            f.nome_social,
            f.documento,
            f.rg,
            f.pis,
            f.email,
            f.telefone,
            f.celular,
            f.data_nascimento,
            f.data_admissao,
            f.data_demissao,
            f.sexo,
            f.estado_civil,
            f.cep,
            f.endereco,
            f.numero,
            f.complemento,
            f.bairro,
            f.cidade,
            f.estado,
            f.departamento_id,
            d.descricao AS departamento_nome,
            f.funcao_id,
            fnc.descricao AS funcao_nome,
            f.centro_custo_id,
            cc.descricao AS centro_custo_nome,
            f.horario_id,
            h.descricao AS horario_nome,
            f.escala_id,
            es.descricao AS escala_nome,
            f.jornada_id,
            jt.descricao AS jornada_nome,
            f.observacoes,
            f.ativo,
            f.created_at,
            f.updated_at
     FROM funcionarios f
     LEFT JOIN empresas e ON e.id = f.empresa_id
     LEFT JOIN departamentos d ON d.id = f.departamento_id
     LEFT JOIN funcoes fnc ON fnc.id = f.funcao_id
     LEFT JOIN centro_custos cc ON cc.id = f.centro_custo_id
     LEFT JOIN horarios h ON h.id = f.horario_id
     LEFT JOIN escalas es ON es.id = f.escala_id
     LEFT JOIN jornadas_trabalho jt ON jt.id = f.jornada_id"
}

fn validate_fk_exists(
    conn: &rusqlite::Connection,
    table: &str,
    id: Option<i64>,
    description: &str,
) -> Result<(), String> {
    if let Some(record_id) = id {
        let sql = format!("SELECT id FROM {table} WHERE id = ?1 LIMIT 1");
        let exists: Option<i64> = conn
            .query_row(&sql, [record_id], |row| row.get(0))
            .optional()
            .map_err(|err| format!("Falha ao validar {description}: {err}"))?;

        if exists.is_none() {
            return Err(format!("{description} informado não existe."));
        }
    }

    Ok(())
}

#[tauri::command]
pub fn employee_list(
    state: State<'_, SharedState>,
    filters: Map<String, Value>,
) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let search = get_string(&filters, "search").unwrap_or_default();
    let empresa_id = get_i64(&filters, "empresaId");
    let only_active = get_bool(&filters, "onlyActive", false) == 1;

    let mut sql = format!("{} WHERE 1=1", employee_select_sql());
    let mut values: Vec<rusqlite::types::Value> = Vec::new();

    if !search.is_empty() {
        sql.push_str(
            " AND (f.nome LIKE ? OR f.nome_social LIKE ? OR f.matricula LIKE ? OR f.documento LIKE ? OR e.nome LIKE ?)",
        );
        let wildcard = format!("%{}%", search);
        let numeric_search = only_digits(&search);
        let doc_wildcard = format!("%{}%", numeric_search);
        values.push(rusqlite::types::Value::Text(wildcard.clone()));
        values.push(rusqlite::types::Value::Text(wildcard.clone()));
        values.push(rusqlite::types::Value::Text(if numeric_search.is_empty() {
            wildcard.clone()
        } else {
            doc_wildcard.clone()
        }));
        values.push(rusqlite::types::Value::Text(if numeric_search.is_empty() {
            wildcard.clone()
        } else {
            doc_wildcard
        }));
        values.push(rusqlite::types::Value::Text(wildcard));
    }

    if let Some(company_id) = empresa_id {
        sql.push_str(" AND f.empresa_id = ?");
        values.push(rusqlite::types::Value::Integer(company_id));
    }

    if only_active {
        sql.push_str(" AND f.ativo = 1");
    }

    sql.push_str(" ORDER BY f.nome ASC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|err| format!("Falha ao preparar listagem de funcionários: {err}"))?;

    let rows = stmt
        .query_map(rusqlite::params_from_iter(values.iter()), row_to_json_map)
        .map_err(|err| format!("Falha ao executar listagem de funcionários: {err}"))?;

    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear funcionários: {err}"))
}

#[tauri::command]
pub fn employee_get(state: State<'_, SharedState>, id: i64) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let sql = format!("{} WHERE f.id = ?1", employee_select_sql());

    conn.query_row(&sql, [id], row_to_json_map)
        .optional()
        .map_err(|err| format!("Falha ao consultar funcionário: {err}"))?
        .ok_or_else(|| "Funcionário não encontrado.".to_string())
}

#[tauri::command]
pub fn employee_save(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = Utc::now().to_rfc3339();
    let id = get_id(&payload);

    let empresa_id = get_i64(&payload, "empresa_id")
        .ok_or_else(|| "Selecione a empresa do funcionário.".to_string())?;
    let matricula = get_string(&payload, "matricula")
        .ok_or_else(|| "Informe a matrícula do funcionário.".to_string())?;
    let nome =
        get_string(&payload, "nome").ok_or_else(|| "Informe o nome do funcionário.".to_string())?;
    let documento_raw = get_string(&payload, "documento")
        .ok_or_else(|| "Informe o CPF do funcionário.".to_string())?;
    let documento = normalize_cpf(&documento_raw)
        .ok_or_else(|| "CPF do funcionário é inválido.".to_string())?;
    let data_admissao = get_string(&payload, "data_admissao")
        .ok_or_else(|| "Informe a data de admissão.".to_string())?;

    if !is_valid_cpf(&documento) {
        return Err("CPF do funcionário é inválido.".to_string());
    }

    if !validate_iso_date(&data_admissao) {
        return Err("Data de admissão inválida. Utilize o formato YYYY-MM-DD.".to_string());
    }

    let data_nascimento = get_string(&payload, "data_nascimento");
    if let Some(value) = data_nascimento.as_deref() {
        if !validate_iso_date(value) {
            return Err("Data de nascimento inválida. Utilize o formato YYYY-MM-DD.".to_string());
        }
    }

    let data_demissao = get_string(&payload, "data_demissao");
    if let Some(value) = data_demissao.as_deref() {
        if !validate_iso_date(value) {
            return Err("Data de desligamento inválida. Utilize o formato YYYY-MM-DD.".to_string());
        }

        if value < data_admissao.as_str() {
            return Err(
                "Data de desligamento não pode ser menor que a data de admissão.".to_string(),
            );
        }
    }

    let email = get_string(&payload, "email");
    if let Some(value) = email.as_deref() {
        if !validate_email(value) {
            return Err("E-mail do funcionário é inválido.".to_string());
        }
    }

    let pis = get_string(&payload, "pis").map(|value| only_digits(&value));
    if let Some(value) = pis.as_deref() {
        if !is_valid_pis(value) {
            return Err("PIS/PASEP do funcionário é inválido.".to_string());
        }
    }

    let estado = normalize_upper_uf(get_string(&payload, "estado"));
    if let Some(uf) = &estado {
        if uf.len() != 2 {
            return Err("UF do funcionário deve conter 2 caracteres.".to_string());
        }
    }

    let empresa_exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM empresas WHERE id = ?1 LIMIT 1",
            [empresa_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar empresa do funcionário: {err}"))?;

    if empresa_exists.is_none() {
        return Err("A empresa informada para o funcionário não existe.".to_string());
    }

    validate_fk_exists(
        &conn,
        "departamentos",
        get_i64(&payload, "departamento_id"),
        "Departamento",
    )?;
    validate_fk_exists(&conn, "funcoes", get_i64(&payload, "funcao_id"), "Função")?;
    validate_fk_exists(
        &conn,
        "centro_custos",
        get_i64(&payload, "centro_custo_id"),
        "Centro de custo",
    )?;
    validate_fk_exists(
        &conn,
        "horarios",
        get_i64(&payload, "horario_id"),
        "Horário",
    )?;
    validate_fk_exists(&conn, "escalas", get_i64(&payload, "escala_id"), "Escala")?;
    validate_fk_exists(
        &conn,
        "jornadas_trabalho",
        get_i64(&payload, "jornada_id"),
        "Jornada",
    )?;

    let duplicate_matricula: Option<i64> = conn
        .query_row(
            "SELECT id FROM funcionarios WHERE empresa_id = ?1 AND matricula = ?2 AND (?3 IS NULL OR id <> ?3) LIMIT 1",
            params![empresa_id, matricula, id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar matrícula: {err}"))?;

    if duplicate_matricula.is_some() {
        return Err("Já existe funcionário com esta matrícula na empresa selecionada.".to_string());
    }

    let duplicate_cpf: Option<i64> = conn
        .query_row(
            "SELECT id FROM funcionarios WHERE documento = ?1 AND (?2 IS NULL OR id <> ?2) LIMIT 1",
            params![documento, id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao validar CPF do funcionário: {err}"))?;

    if duplicate_cpf.is_some() {
        return Err("Já existe funcionário cadastrado com este CPF.".to_string());
    }

    let nome_social = get_string(&payload, "nome_social");
    let rg = get_string(&payload, "rg");
    let telefone = get_string(&payload, "telefone").map(|value| only_digits(&value));
    let celular = get_string(&payload, "celular").map(|value| only_digits(&value));
    let sexo = get_string(&payload, "sexo");
    let estado_civil = get_string(&payload, "estado_civil");
    let cep = get_string(&payload, "cep").map(|value| only_digits(&value));
    let endereco = get_string(&payload, "endereco");
    let numero = get_string(&payload, "numero");
    let complemento = get_string(&payload, "complemento");
    let bairro = get_string(&payload, "bairro");
    let cidade = get_string(&payload, "cidade");
    let departamento_id = get_i64(&payload, "departamento_id");
    let funcao_id = get_i64(&payload, "funcao_id");
    let centro_custo_id = get_i64(&payload, "centro_custo_id");
    let horario_id = get_i64(&payload, "horario_id");
    let escala_id = get_i64(&payload, "escala_id");
    let jornada_id = get_i64(&payload, "jornada_id");
    let observacoes = get_string(&payload, "observacoes");
    let ativo = get_bool(&payload, "ativo", true);

    let record_id = if let Some(existing_id) = id {
        conn.execute(
            "UPDATE funcionarios
             SET empresa_id = ?1,
                 matricula = ?2,
                 nome = ?3,
                 nome_social = ?4,
                 documento = ?5,
                 rg = ?6,
                 pis = ?7,
                 email = ?8,
                 telefone = ?9,
                 celular = ?10,
                 data_nascimento = ?11,
                 data_admissao = ?12,
                 data_demissao = ?13,
                 sexo = ?14,
                 estado_civil = ?15,
                 cep = ?16,
                 endereco = ?17,
                 numero = ?18,
                 complemento = ?19,
                 bairro = ?20,
                 cidade = ?21,
                 estado = ?22,
                 departamento_id = ?23,
                 funcao_id = ?24,
                 centro_custo_id = ?25,
                 horario_id = ?26,
                 escala_id = ?27,
                 jornada_id = ?28,
                 observacoes = ?29,
                 ativo = ?30,
                 updated_at = ?31
             WHERE id = ?32",
            params![
                empresa_id,
                matricula,
                nome,
                nome_social,
                documento,
                rg,
                pis,
                email,
                telefone,
                celular,
                data_nascimento,
                data_admissao,
                data_demissao,
                sexo,
                estado_civil,
                cep,
                endereco,
                numero,
                complemento,
                bairro,
                cidade,
                estado,
                departamento_id,
                funcao_id,
                centro_custo_id,
                horario_id,
                escala_id,
                jornada_id,
                observacoes,
                ativo,
                now,
                existing_id,
            ],
        )
        .map_err(|err| format!("Falha ao atualizar funcionário: {err}"))?;
        existing_id
    } else {
        conn.execute(
            "INSERT INTO funcionarios (
                empresa_id, matricula, nome, nome_social, documento, rg, pis, email, telefone, celular,
                data_nascimento, data_admissao, data_demissao, sexo, estado_civil, cep, endereco, numero,
                complemento, bairro, cidade, estado, departamento_id, funcao_id, centro_custo_id, horario_id,
                escala_id, jornada_id, observacoes, ativo, created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
                ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?31
             )",
            params![
                empresa_id,
                matricula,
                nome,
                nome_social,
                documento,
                rg,
                pis,
                email,
                telefone,
                celular,
                data_nascimento,
                data_admissao,
                data_demissao,
                sexo,
                estado_civil,
                cep,
                endereco,
                numero,
                complemento,
                bairro,
                cidade,
                estado,
                departamento_id,
                funcao_id,
                centro_custo_id,
                horario_id,
                escala_id,
                jornada_id,
                observacoes,
                ativo,
                now,
            ],
        )
        .map_err(|err| format!("Falha ao inserir funcionário: {err}"))?;
        conn.last_insert_rowid()
    };

    let saved = conn
        .query_row(
            &format!("{} WHERE f.id = ?1", employee_select_sql()),
            [record_id],
            row_to_json_map,
        )
        .optional()
        .map_err(|err| format!("Falha ao reler funcionário salvo: {err}"))?
        .ok_or_else(|| "Funcionário salvo não encontrado.".to_string())?;

    let payload_value = Value::Object(saved.clone());
    write_audit(
        &conn,
        "funcionarios",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;
    enqueue_sync(
        &conn,
        "funcionarios",
        if id.is_some() { "update" } else { "create" },
        Some(record_id),
        &payload_value,
    )?;

    Ok(saved)
}

#[tauri::command]
pub fn employee_delete(state: State<'_, SharedState>, id: i64) -> Result<bool, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;

    let punch_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM batidas WHERE funcionario_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar batidas do funcionário: {err}"))?;

    if punch_count > 0 {
        return Err(
            "Não é possível excluir o funcionário porque existem batidas vinculadas.".to_string(),
        );
    }

    let bank_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM banco_horas_lancamentos WHERE funcionario_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Falha ao verificar banco de horas do funcionário: {err}"))?;

    if bank_count > 0 {
        return Err(
            "Não é possível excluir o funcionário porque existem lançamentos de banco de horas vinculados.".to_string(),
        );
    }

    let affected = conn
        .execute("DELETE FROM funcionarios WHERE id = ?1", [id])
        .map_err(|err| format!("Falha ao excluir funcionário: {err}"))?;

    if affected > 0 {
        let payload = json!({ "id": id, "entity": "funcionarios" });
        write_audit(&conn, "funcionarios", "delete", Some(id), &payload)?;
        enqueue_sync(&conn, "funcionarios", "delete", Some(id), &payload)?;
    }

    Ok(affected > 0)
}

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{params, OptionalExtension};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, row_to_json_map, write_audit},
    models::{AfdImportRequest, AfdImportResponse},
};

#[derive(Debug, Clone, Copy)]
enum AfdLayout {
    Portaria1510,
    Portaria671,
}

impl AfdLayout {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Portaria1510 => "1510",
            Self::Portaria671 => "671",
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedMark {
    nsr: String,
    tipo_registro: String,
    chave_trabalhador: String,
    data_hora_marcacao: String,
    data_hora_gravacao: Option<String>,
    coletor_codigo: Option<String>,
    online: Option<bool>,
    hash_registro: Option<String>,
    linha_bruta: String,
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn slice_ascii(line: &str, start: usize, end: usize) -> String {
    let bytes = line.as_bytes();
    let start_idx = start.saturating_sub(1).min(bytes.len());
    let end_idx = end.min(bytes.len());
    String::from_utf8_lossy(&bytes[start_idx..end_idx]).trim().to_string()
}

fn detect_layout(lines: &[String]) -> AfdLayout {
    for line in lines {
        if line.len() >= 50 {
            let tipo = slice_ascii(line, 10, 10);
            let dateish = slice_ascii(line, 11, 34);
            if (tipo == "3" || tipo == "7") && dateish.contains('T') && dateish.contains('-') {
                return AfdLayout::Portaria671;
            }
        }
        if line.len() >= 34 {
            let tipo = slice_ascii(line, 10, 10);
            let data = slice_ascii(line, 11, 18);
            let hora = slice_ascii(line, 19, 22);
            if tipo == "3" && data.len() == 8 && hora.len() == 4 && data.chars().all(|c| c.is_ascii_digit()) {
                return AfdLayout::Portaria1510;
            }
        }
    }

    AfdLayout::Portaria671
}

fn parse_legacy_datetime(data: &str, hora: &str) -> Option<String> {
    let date = NaiveDate::parse_from_str(data, "%d%m%Y").ok()?;
    let datetime = NaiveDateTime::parse_from_str(
        &format!("{} {}", date.format("%Y-%m-%d"), format!("{}:{}", &hora[0..2], &hora[2..4])),
        "%Y-%m-%d %H:%M",
    )
    .ok()?;
    Some(datetime.format("%Y-%m-%dT%H:%M:00").to_string())
}

fn parse_mark_line_1510(line: &str) -> Option<ParsedMark> {
    if slice_ascii(line, 10, 10) != "3" {
        return None;
    }
    let data = slice_ascii(line, 11, 18);
    let hora = slice_ascii(line, 19, 22);
    Some(ParsedMark {
        nsr: slice_ascii(line, 1, 9),
        tipo_registro: "3".to_string(),
        chave_trabalhador: slice_ascii(line, 23, 34),
        data_hora_marcacao: parse_legacy_datetime(&data, &hora)?,
        data_hora_gravacao: None,
        coletor_codigo: None,
        online: None,
        hash_registro: None,
        linha_bruta: line.to_string(),
    })
}

fn normalize_datetime_671(value: &str) -> Option<String> {
    if value.len() < 16 {
        return None;
    }
    Some(value.replace("-0300", "").replace("-0200", "").replace("+0000", ""))
}

fn parse_mark_line_671(line: &str) -> Option<ParsedMark> {
    let tipo = slice_ascii(line, 10, 10);
    match tipo.as_str() {
        "3" => Some(ParsedMark {
            nsr: slice_ascii(line, 1, 9),
            tipo_registro: tipo,
            chave_trabalhador: slice_ascii(line, 35, 46),
            data_hora_marcacao: normalize_datetime_671(&slice_ascii(line, 11, 34))?,
            data_hora_gravacao: None,
            coletor_codigo: None,
            online: None,
            hash_registro: None,
            linha_bruta: line.to_string(),
        }),
        "7" => Some(ParsedMark {
            nsr: slice_ascii(line, 1, 9),
            tipo_registro: tipo,
            chave_trabalhador: slice_ascii(line, 35, 46),
            data_hora_marcacao: normalize_datetime_671(&slice_ascii(line, 11, 34))?,
            data_hora_gravacao: normalize_datetime_671(&slice_ascii(line, 47, 70)),
            coletor_codigo: Some(slice_ascii(line, 71, 72)),
            online: Some(slice_ascii(line, 73, 73) != "1"),
            hash_registro: Some(slice_ascii(line, 74, 137)),
            linha_bruta: line.to_string(),
        }),
        _ => None,
    }
}

fn parse_marks(layout: AfdLayout, lines: &[String]) -> Vec<ParsedMark> {
    lines
        .iter()
        .filter_map(|line| match layout {
            AfdLayout::Portaria1510 => parse_mark_line_1510(line),
            AfdLayout::Portaria671 => parse_mark_line_671(line),
        })
        .collect()
}

fn employee_id_from_key(conn: &rusqlite::Connection, empresa_id: Option<i64>, layout: AfdLayout, raw_key: &str) -> Result<Option<i64>, String> {
    let digits = only_digits(raw_key);
    if digits.is_empty() {
        return Ok(None);
    }

    let preferred_key = if matches!(layout, AfdLayout::Portaria671) && digits.len() > 11 {
        digits[digits.len() - 11..].to_string()
    } else {
        digits.clone()
    };

    let sql = match layout {
        AfdLayout::Portaria1510 => {
            "SELECT id FROM funcionarios
             WHERE (?1 IS NULL OR empresa_id = ?1)
               AND (pis = ?2 OR ltrim(pis, '0') = ltrim(?2, '0') OR documento = ?3)
             ORDER BY id ASC LIMIT 1"
        }
        AfdLayout::Portaria671 => {
            "SELECT id FROM funcionarios
             WHERE (?1 IS NULL OR empresa_id = ?1)
               AND (documento = ?2 OR ltrim(documento, '0') = ltrim(?2, '0') OR pis = ?3 OR ltrim(pis, '0') = ltrim(?3, '0'))
             ORDER BY id ASC LIMIT 1"
        }
    };

    conn.query_row(sql, params![empresa_id, preferred_key, digits], |row| row.get(0))
        .optional()
        .map_err(|err| format!("Falha ao localizar funcionário para marcação AFD: {err}"))
}

fn datetime_to_date_time(value: &str) -> Option<(String, String)> {
    if value.contains('T') {
        let parts: Vec<&str> = value.split('T').collect();
        if parts.len() == 2 {
            let time = parts[1].get(0..5)?.to_string();
            return Some((parts[0].to_string(), time));
        }
    }
    None
}

#[tauri::command]
pub fn afd_import_list(state: State<'_, SharedState>) -> Result<Vec<Map<String, Value>>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let mut stmt = conn
        .prepare(
            "SELECT ai.id,
                    ai.empresa_id,
                    e.nome AS empresa_nome,
                    ai.equipamento_id,
                    eq.descricao AS equipamento_nome,
                    ai.nome_arquivo,
                    ai.layout_portaria,
                    ai.formato_detectado,
                    ai.periodo_inicial,
                    ai.periodo_final,
                    ai.total_linhas,
                    ai.total_marcacoes,
                    ai.total_processadas,
                    ai.total_descartadas,
                    ai.created_at
             FROM afd_importacoes ai
             LEFT JOIN empresas e ON e.id = ai.empresa_id
             LEFT JOIN equipamentos eq ON eq.id = ai.equipamento_id
             ORDER BY ai.id DESC"
        )
        .map_err(|err| format!("Falha ao preparar listagem de importações AFD: {err}"))?;

    let rows = stmt.query_map([], row_to_json_map).map_err(|err| format!("Falha ao consultar importações AFD: {err}"))?;
    let rows: Result<Vec<_>, _> = rows.collect();
    rows.map_err(|err| format!("Falha ao mapear importações AFD: {err}"))
}

#[tauri::command]
pub fn afd_import_file(state: State<'_, SharedState>, payload: AfdImportRequest) -> Result<AfdImportResponse, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let now = chrono::Utc::now().to_rfc3339();
    let lines: Vec<String> = payload
        .content
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .filter(|line| !line.trim().is_empty())
        .collect();

    if lines.is_empty() {
        return Err("O arquivo AFD está vazio.".to_string());
    }

    let forced = payload.mode.unwrap_or_else(|| "auto".to_string()).to_lowercase();
    let layout = match forced.as_str() {
        "1510" => AfdLayout::Portaria1510,
        "671" => AfdLayout::Portaria671,
        _ => detect_layout(&lines),
    };

    let parsed_marks = parse_marks(layout, &lines);
    if parsed_marks.is_empty() {
        return Err("Nenhuma marcação válida foi encontrada no arquivo AFD informado.".to_string());
    }

    let mut hasher = Sha256::new();
    hasher.update(payload.content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let period_start = parsed_marks.first().map(|item| item.data_hora_marcacao.clone());
    let period_end = parsed_marks.last().map(|item| item.data_hora_marcacao.clone());

    conn.execute(
        "INSERT INTO afd_importacoes (
            empresa_id, equipamento_id, nome_arquivo, layout_portaria, formato_detectado,
            periodo_inicial, periodo_final, total_linhas, total_marcacoes, total_processadas,
            total_descartadas, hash_arquivo, conteudo_bruto, resumo_json, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10, ?11, ?12, ?13, ?13)",
        params![
            payload.empresa_id,
            payload.equipamento_id,
            payload.file_name,
            layout.as_str(),
            if forced == "auto" { format!("auto:{}", layout.as_str()) } else { forced.clone() },
            period_start,
            period_end,
            i64::try_from(lines.len()).unwrap_or(0),
            i64::try_from(parsed_marks.len()).unwrap_or(0),
            hash,
            payload.content,
            json!({"mensagem":"importação inicializada"}).to_string(),
            now,
        ],
    )
    .map_err(|err| format!("Falha ao registrar importação AFD: {err}"))?;
    let importacao_id = conn.last_insert_rowid();

    let mut total_processadas = 0usize;
    let mut total_descartadas = 0usize;
    let mut mensagens = vec![format!("Layout identificado para tratamento: Portaria {}.", layout.as_str())];

    for mark in &parsed_marks {
        let funcionario_id = employee_id_from_key(&conn, payload.empresa_id, layout, &mark.chave_trabalhador)?;
        let (data_referencia, hora) = datetime_to_date_time(&mark.data_hora_marcacao)
            .ok_or_else(|| "Falha ao converter data/hora de marcação do AFD.".to_string())?;

        let (status, mensagem, batida_id) = if let Some(funcionario_id) = funcionario_id {
            let duplicate: Option<i64> = conn
                .query_row(
                    "SELECT id FROM batidas WHERE funcionario_id = ?1 AND data_referencia = ?2 AND hora = ?3 AND COALESCE(nsr, '') = COALESCE(?4, '') LIMIT 1",
                    params![funcionario_id, data_referencia, hora, mark.nsr],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| format!("Falha ao validar duplicidade da marcação AFD: {err}"))?;

            if let Some(existing_id) = duplicate {
                total_descartadas += 1;
                ("duplicada".to_string(), "Marcação já existente no banco local.".to_string(), Some(existing_id))
            } else {
                conn.execute(
                    "INSERT INTO batidas (
                        funcionario_id, equipamento_id, afd_importacao_id, afd_layout_portaria, data_referencia,
                        hora, nsr, origem, observacao, tipo, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'marcacao', ?10, ?10)",
                    params![
                        funcionario_id,
                        payload.equipamento_id,
                        importacao_id,
                        layout.as_str(),
                        data_referencia,
                        hora,
                        mark.nsr,
                        format!("afd_{}", layout.as_str()),
                        format!("Importação AFD {}", payload.file_name),
                        now,
                    ],
                )
                .map_err(|err| format!("Falha ao inserir batida importada do AFD: {err}"))?;
                total_processadas += 1;
                ("importada".to_string(), "Marcação importada com sucesso.".to_string(), Some(conn.last_insert_rowid()))
            }
        } else {
            total_descartadas += 1;
            ("sem_funcionario".to_string(), format!("Nenhum funcionário localizado para a chave {}.", mark.chave_trabalhador), None)
        };

        conn.execute(
            "INSERT INTO afd_marcacoes (
                importacao_id, nsr, tipo_registro, chave_trabalhador, data_hora_marcacao, data_hora_gravacao,
                coletor_codigo, online, hash_registro, linha_bruta, funcionario_id, batida_id, status, mensagem, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                importacao_id,
                mark.nsr,
                mark.tipo_registro,
                mark.chave_trabalhador,
                mark.data_hora_marcacao,
                mark.data_hora_gravacao,
                mark.coletor_codigo,
                mark.online.map(|value| if value { 1 } else { 0 }),
                mark.hash_registro,
                mark.linha_bruta,
                funcionario_id,
                batida_id,
                status,
                mensagem,
                now,
            ],
        )
        .map_err(|err| format!("Falha ao registrar detalhe da importação AFD: {err}"))?;
    }

    conn.execute(
        "UPDATE afd_importacoes
            SET total_processadas = ?1,
                total_descartadas = ?2,
                resumo_json = ?3,
                updated_at = ?4
          WHERE id = ?5",
        params![
            i64::try_from(total_processadas).unwrap_or(0),
            i64::try_from(total_descartadas).unwrap_or(0),
            json!({
                "processadas": total_processadas,
                "descartadas": total_descartadas,
                "layout": layout.as_str()
            })
            .to_string(),
            now,
            importacao_id,
        ],
    )
    .map_err(|err| format!("Falha ao finalizar importação AFD: {err}"))?;

    let payload_value = json!({
        "importacao_id": importacao_id,
        "layout_portaria": layout.as_str(),
        "total_linhas": lines.len(),
        "total_marcacoes": parsed_marks.len(),
        "total_processadas": total_processadas,
        "total_descartadas": total_descartadas,
    });
    write_audit(&conn, "afd_importacoes", "create", Some(importacao_id), &payload_value)?;
    enqueue_sync(&conn, "afd_importacoes", "create", Some(importacao_id), &payload_value)?;

    if matches!(layout, AfdLayout::Portaria1510) {
        mensagens.push("Suporte aplicado ao AFD legado da Portaria 1.510/2009 com chave principal por PIS e compatibilidade para marcas já tratadas no banco local.".to_string());
    } else {
        mensagens.push("Suporte aplicado ao AFD da Portaria 671/2021 para registros tipo 3 (REP-C/REP-A) e tipo 7 (REP-P).".to_string());
    }

    Ok(AfdImportResponse {
        importacao_id,
        layout_portaria: layout.as_str().to_string(),
        total_linhas: lines.len(),
        total_marcacoes: parsed_marks.len(),
        total_processadas,
        total_descartadas,
        mensagens,
    })
}

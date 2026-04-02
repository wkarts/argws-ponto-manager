use chrono::Utc;
use reqwest::blocking::Client;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tauri::State;

use crate::{
    app_state::SharedState,
    db::{enqueue_sync, open_connection, write_audit},
};

const SETTINGS_KEY: &str = "holiday_source_settings";
const EMBEDDED_HOLIDAYS_2026: &str = include_str!("../resources/feriados/feriados_2026_bundle.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HolidaySourceSettings {
    mode: String,
    year: i32,
    remote_json_url: Option<String>,
    api_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HolidayDataset {
    nacional: Vec<HolidayItem>,
    estadual: Vec<HolidayItem>,
    municipal: Vec<HolidayItem>,
    municipios: Vec<MunicipioItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HolidayItem {
    data: String,
    nome: String,
    tipo: String,
    descricao: Option<String>,
    uf: Option<String>,
    codigo_ibge: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MunicipioItem {
    codigo_ibge: i64,
    nome: String,
    codigo_uf: Option<i64>,
}

fn default_settings() -> HolidaySourceSettings {
    HolidaySourceSettings {
        mode: "embedded".to_string(),
        year: 2026,
        remote_json_url: None,
        api_url: None,
    }
}

fn load_settings(conn: &rusqlite::Connection) -> Result<HolidaySourceSettings, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT valor FROM app_settings WHERE chave = ?1 LIMIT 1",
            [SETTINGS_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar configurações de fonte de feriados: {err}"))?;

    match raw {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str::<HolidaySourceSettings>(&value)
                .map_err(|err| format!("Falha ao interpretar configurações de feriados: {err}"))
        }
        _ => Ok(default_settings()),
    }
}

fn save_settings(conn: &rusqlite::Connection, settings: &HolidaySourceSettings) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let raw = serde_json::to_string(settings)
        .map_err(|err| format!("Falha ao serializar configurações de feriados: {err}"))?;

    conn.execute(
        "INSERT INTO app_settings (chave, valor, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(chave) DO UPDATE SET valor = excluded.valor, updated_at = excluded.updated_at",
        params![SETTINGS_KEY, raw, now],
    )
    .map_err(|err| format!("Falha ao salvar configurações de feriados: {err}"))?;

    Ok(())
}

fn sanitize_settings(input: Map<String, Value>) -> HolidaySourceSettings {
    let current = default_settings();
    let mode = input
        .get("mode")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_lowercase())
        .filter(|value| matches!(value.as_str(), "embedded" | "remote_json" | "api"))
        .unwrap_or(current.mode);

    let year = input
        .get("year")
        .and_then(|value| value.as_i64().or_else(|| value.as_str().and_then(|item| item.parse::<i64>().ok())))
        .map(|value| value as i32)
        .filter(|value| *value >= 2000 && *value <= 2100)
        .unwrap_or(current.year);

    let remote_json_url = input
        .get("remote_json_url")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let api_url = input
        .get("api_url")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    HolidaySourceSettings {
        mode,
        year,
        remote_json_url,
        api_url,
    }
}

fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| match ch {
            'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            '\'' | '"' | '-' | '/' | '.' | ',' => ' ',
            _ => ch,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_br_date_to_iso(value: &str) -> Result<String, String> {
    let value = value.trim();
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() == 3 {
        return Ok(format!("{}-{}-{}", parts[2], parts[1], parts[0]));
    }
    if value.len() == 10 && value.chars().nth(4) == Some('-') {
        return Ok(value.to_string());
    }
    Err(format!("Data de feriado inválida no dataset: {value}"))
}

fn find_city_code(dataset: &HolidayDataset, city: &str) -> Option<i64> {
    let normalized = normalize_text(city);
    dataset
        .municipios
        .iter()
        .find(|item| normalize_text(&item.nome) == normalized)
        .map(|item| item.codigo_ibge)
}

fn render_url_template(template: &str, empresa_id: i64, year: i32, uf: &str, cidade: &str) -> String {
    template
        .replace("{empresa_id}", &empresa_id.to_string())
        .replace("{year}", &year.to_string())
        .replace("{uf}", uf)
        .replace("{cidade}", cidade)
        .replace("{city}", cidade)
}

fn load_dataset(settings: &HolidaySourceSettings, empresa_id: i64, uf: &str, cidade: &str) -> Result<HolidayDataset, String> {
    let raw = match settings.mode.as_str() {
        "embedded" => EMBEDDED_HOLIDAYS_2026.to_string(),
        "remote_json" => {
            let template = settings
                .remote_json_url
                .as_deref()
                .ok_or_else(|| "Informe a URL JSON remota nas configurações de feriados.".to_string())?;
            let url = render_url_template(template, empresa_id, settings.year, uf, cidade);
            Client::new()
                .get(url)
                .send()
                .and_then(|response| response.error_for_status())
                .map_err(|err| format!("Falha ao baixar JSON remoto de feriados: {err}"))?
                .text()
                .map_err(|err| format!("Falha ao ler resposta do JSON remoto de feriados: {err}"))?
        }
        "api" => {
            let template = settings
                .api_url
                .as_deref()
                .ok_or_else(|| "Informe a URL da API de feriados nas configurações.".to_string())?;
            let url = render_url_template(template, empresa_id, settings.year, uf, cidade);
            Client::new()
                .get(url)
                .send()
                .and_then(|response| response.error_for_status())
                .map_err(|err| format!("Falha ao consultar API de feriados: {err}"))?
                .text()
                .map_err(|err| format!("Falha ao ler resposta da API de feriados: {err}"))?
        }
        other => {
            return Err(format!("Modo de fonte de feriados inválido: {other}"));
        }
    };

    serde_json::from_str::<HolidayDataset>(&raw)
        .map_err(|err| format!("Falha ao interpretar dataset de feriados: {err}"))
}

fn relation_exists(conn: &rusqlite::Connection, feriado_id: i64, empresa_id: i64) -> Result<bool, String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM feriados_empresas WHERE feriado_id = ?1 AND empresa_id = ?2 LIMIT 1",
            params![feriado_id, empresa_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao verificar vínculo do feriado: {err}"))?;
    Ok(exists.is_some())
}

fn upsert_relation(conn: &rusqlite::Connection, feriado_id: i64, empresa_id: i64, now: &str) -> Result<(), String> {
    if relation_exists(conn, feriado_id, empresa_id)? {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO feriados_empresas (feriado_id, empresa_id, created_at) VALUES (?1, ?2, ?3)",
        params![feriado_id, empresa_id, now],
    )
    .map_err(|err| format!("Falha ao criar vínculo do feriado com a empresa: {err}"))?;

    Ok(())
}

fn upsert_holiday_for_company(
    conn: &rusqlite::Connection,
    company_id: i64,
    item: &HolidayItem,
    source_tag: &str,
    now: &str,
) -> Result<bool, String> {
    let iso_date = parse_br_date_to_iso(&item.data)?;
    let descricao = item.nome.trim();
    let observacoes = item
        .descricao
        .as_deref()
        .map(|value| format!("Fonte {source_tag} {descricao}: {value}"))
        .unwrap_or_else(|| format!("Fonte {source_tag} {descricao}"));

    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT f.id
               FROM feriados f
               INNER JOIN feriados_empresas fe ON fe.feriado_id = f.id
              WHERE fe.empresa_id = ?1 AND f.data = ?2 AND LOWER(TRIM(f.descricao)) = LOWER(TRIM(?3))
              LIMIT 1",
            params![company_id, iso_date, descricao],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar feriado existente: {err}"))?;

    let feriado_id = if let Some(id) = existing_id {
        conn.execute(
            "UPDATE feriados
                SET contexto_tipo = 'empresa',
                    empresa_id = ?1,
                    observacoes = ?2,
                    ativo = 1,
                    updated_at = ?3
              WHERE id = ?4",
            params![company_id, observacoes, now, id],
        )
        .map_err(|err| format!("Falha ao atualizar feriado existente: {err}"))?;
        id
    } else {
        conn.execute(
            "INSERT INTO feriados (
                data, descricao, contexto_tipo, empresa_id, departamento_id,
                regra_jornada, regra_compensacao, observacoes, ativo, created_at, updated_at
             ) VALUES (?1, ?2, 'empresa', ?3, NULL, NULL, NULL, ?4, 1, ?5, ?5)",
            params![iso_date, descricao, company_id, observacoes, now],
        )
        .map_err(|err| format!("Falha ao inserir feriado importado: {err}"))?;
        conn.last_insert_rowid()
    };

    upsert_relation(conn, feriado_id, company_id, now)?;
    Ok(existing_id.is_none())
}

#[tauri::command]
pub fn holiday_source_load_settings(state: State<'_, SharedState>) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let settings = load_settings(&conn)?;
    let value = serde_json::to_value(settings)
        .map_err(|err| format!("Falha ao converter configurações de feriados: {err}"))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

#[tauri::command]
pub fn holiday_source_save_settings(
    state: State<'_, SharedState>,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let settings = sanitize_settings(payload);
    save_settings(&conn, &settings)?;
    let value = serde_json::to_value(&settings)
        .map_err(|err| format!("Falha ao converter configurações salvas de feriados: {err}"))?;
    write_audit(&conn, "holiday_source_settings", "update", None, &value)?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

#[tauri::command]
pub fn holiday_source_import_company_year(
    state: State<'_, SharedState>,
    empresa_id: i64,
    year: Option<i32>,
) -> Result<Map<String, Value>, String> {
    let db_path = state.db_path()?;
    let conn = open_connection(&db_path)?;
    let mut settings = load_settings(&conn)?;
    if let Some(value) = year {
        settings.year = value;
    }

    let company: Option<(String, String)> = conn
        .query_row(
            "SELECT COALESCE(estado, ''), COALESCE(cidade, '') FROM empresas WHERE id = ?1 LIMIT 1",
            [empresa_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|err| format!("Falha ao consultar empresa para importação de feriados: {err}"))?;

    let (uf, cidade) = company.ok_or_else(|| "Empresa não encontrada para importação de feriados.".to_string())?;
    let uf = uf.trim().to_uppercase();
    let cidade = cidade.trim().to_string();
    if uf.is_empty() {
        return Err("A empresa ativa não possui UF cadastrada para importar feriados estaduais.".to_string());
    }
    if cidade.is_empty() {
        return Err("A empresa ativa não possui cidade cadastrada para importar feriados municipais.".to_string());
    }

    let dataset = load_dataset(&settings, empresa_id, &uf, &cidade)?;
    let city_code = find_city_code(&dataset, &cidade);
    let now = Utc::now().to_rfc3339();

    let mut imported_nacional = 0_i64;
    let mut imported_estadual = 0_i64;
    let mut imported_municipal = 0_i64;

    for item in &dataset.nacional {
        if upsert_holiday_for_company(&conn, empresa_id, item, "NACIONAL", &now)? {
            imported_nacional += 1;
        }
    }

    for item in dataset.estadual.iter().filter(|item| item.uf.as_deref().unwrap_or_default().eq_ignore_ascii_case(&uf)) {
        if upsert_holiday_for_company(&conn, empresa_id, item, "ESTADUAL", &now)? {
            imported_estadual += 1;
        }
    }

    if let Some(code) = city_code {
        for item in dataset.municipal.iter().filter(|item| item.codigo_ibge.unwrap_or_default() == code) {
            if upsert_holiday_for_company(&conn, empresa_id, item, "MUNICIPAL", &now)? {
                imported_municipal += 1;
            }
        }
    }

    let payload = json!({
        "empresa_id": empresa_id,
        "uf": uf,
        "cidade": cidade,
        "year": settings.year,
        "source_mode": settings.mode,
        "city_code": city_code,
        "imported_nacional": imported_nacional,
        "imported_estadual": imported_estadual,
        "imported_municipal": imported_municipal,
        "total_importado": imported_nacional + imported_estadual + imported_municipal,
    });

    write_audit(&conn, "feriados", "import_default_holidays", Some(empresa_id), &payload)?;
    enqueue_sync(&conn, "feriados", "import_default_holidays", Some(empresa_id), &payload)?;

    match payload {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

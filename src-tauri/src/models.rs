use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub session_token: Option<String>,
    pub user: Option<AuthUser>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub nome: String,
    pub login: String,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub cargo: Option<String>,
    pub administrador: bool,
    pub master_user: bool,
    pub senha_provisoria: bool,
    pub permission_keys: Vec<String>,
    pub profile_names: Vec<String>,
    pub company_ids: Vec<i64>,
    pub company_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SessionIdentity {
    pub user_id: i64,
    pub master_user: bool,
}

#[derive(Debug, Deserialize)]
pub struct PunchFilters {
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: Option<i64>,
    #[serde(rename = "dataInicial")]
    pub data_inicial: Option<String>,
    #[serde(rename = "dataFinal")]
    pub data_final: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComboOption {
    pub id: i64,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct ApuracaoRequest {
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: Option<i64>,
    #[serde(rename = "dataInicial")]
    pub data_inicial: Option<String>,
    #[serde(rename = "dataFinal")]
    pub data_final: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApuracaoDia {
    pub funcionario_id: i64,
    pub funcionario_nome: String,
    pub data: String,
    pub jornada_nome: String,
    pub tipo_jornada: String,
    pub horario_esperado_minutos: i64,
    pub trabalhado_minutos: i64,
    pub saldo_minutos: i64,
    pub atraso_minutos: i64,
    pub extra_minutos: i64,
    pub saida_antecipada_minutos: i64,
    pub mensagens: Vec<String>,
    pub batidas: Vec<String>,
    pub ocorrencias: Vec<String>,
    pub minutos_abonados: i64,
    pub abonado: bool,
    pub inconsistente: bool,
}

#[derive(Debug, Serialize)]
pub struct ApuracaoResumo {
    pub total_funcionarios: usize,
    pub total_dias: usize,
    pub total_esperado_minutos: i64,
    pub total_trabalhado_minutos: i64,
    pub total_saldo_minutos: i64,
    pub total_atraso_minutos: i64,
    pub total_extra_minutos: i64,
    pub rows: Vec<ApuracaoDia>,
}

#[derive(Debug, Serialize)]
pub struct SyncQueueItem {
    pub id: i64,
    pub entity_name: String,
    pub action_name: String,
    pub record_id: Option<i64>,
    pub status: String,
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AfdImportRequest {
    #[serde(rename = "empresaId")]
    pub empresa_id: Option<i64>,
    #[serde(rename = "equipamentoId")]
    pub equipamento_id: Option<i64>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub content: String,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AfdImportResponse {
    pub importacao_id: i64,
    pub layout_portaria: String,
    pub total_linhas: usize,
    pub total_marcacoes: usize,
    pub total_processadas: usize,
    pub total_descartadas: usize,
    pub mensagens: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BancoHorasProcessRequest {
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: Option<i64>,
    #[serde(rename = "dataInicial")]
    pub data_inicial: String,
    #[serde(rename = "dataFinal")]
    pub data_final: String,
    #[serde(rename = "overwrite")]
    pub overwrite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BancoHorasAjusteRequest {
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: i64,
    #[serde(rename = "jornadaId")]
    pub jornada_id: Option<i64>,
    #[serde(rename = "dataReferencia")]
    pub data_referencia: String,
    pub minutos: i64,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BancoHorasProcessResponse {
    pub dias_processados: usize,
    pub total_creditos_minutos: i64,
    pub total_debitos_minutos: i64,
    pub saldo_liquido_minutos: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GenericEntityPayload {
    pub entity: String,
    pub payload: Map<String, Value>,
}

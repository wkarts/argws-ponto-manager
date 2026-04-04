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
    #[serde(rename = "empresaId")]
    pub empresa_id: Option<i64>,
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
    #[serde(rename = "empresaId")]
    pub empresa_id: Option<i64>,
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: Option<i64>,
    #[serde(rename = "funcionarioIds")]
    pub funcionario_ids: Option<Vec<i64>>,
    #[serde(rename = "employeeStatus")]
    pub employee_status: Option<String>,
    #[serde(rename = "competenciaAno")]
    pub competencia_ano: Option<i32>,
    #[serde(rename = "competenciaMes")]
    pub competencia_mes: Option<u32>,
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
    #[serde(rename = "empresaId")]
    pub empresa_id: Option<i64>,
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

#[derive(Debug, Deserialize)]
pub struct SmartSuggestionRequest {
    #[serde(rename = "empresaId")]
    pub empresa_id: Option<i64>,
    #[serde(rename = "funcionarioId")]
    pub funcionario_id: Option<i64>,
    #[serde(rename = "funcionarioIds")]
    pub funcionario_ids: Option<Vec<i64>>,
    #[serde(rename = "competenciaAno")]
    pub competencia_ano: Option<i32>,
    #[serde(rename = "competenciaMes")]
    pub competencia_mes: Option<u32>,
    #[serde(rename = "dataInicial")]
    pub data_inicial: Option<String>,
    #[serde(rename = "dataFinal")]
    pub data_final: Option<String>,
    #[serde(rename = "bulkMode")]
    pub bulk_mode: Option<String>,
    #[serde(rename = "selectedSuggestionIds")]
    pub selected_suggestion_ids: Option<Vec<String>>,
    #[serde(rename = "selectedBatidaIds")]
    pub selected_batida_ids: Option<Vec<i64>>,
    #[serde(rename = "replacementTipo")]
    pub replacement_tipo: Option<String>,
    #[serde(rename = "replacementJustificativaId")]
    pub replacement_justificativa_id: Option<i64>,
    #[serde(rename = "overwriteExisting")]
    pub overwrite_existing: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SmartSuggestionItem {
    pub suggestion_id: String,
    pub funcionario_id: i64,
    pub funcionario_nome: String,
    pub data: String,
    pub kind: String,
    pub severity: String,
    pub auto_apply: bool,
    pub expected_minutes: i64,
    pub worked_minutes: i64,
    pub saldo_minutes: i64,
    pub suggested_tipo: String,
    pub suggested_abonar_dia: bool,
    pub suggested_minutos_abonados: i64,
    pub observed_punches: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SmartApplyResponse {
    pub total_sugestoes: usize,
    pub total_aplicadas: usize,
    pub total_ignoradas: usize,
    pub total_batidas_excluidas: usize,
    pub mensagens: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DuplicatePunchCandidate {
    pub bucket_id: String,
    pub funcionario_id: i64,
    pub funcionario_nome: String,
    pub data_referencia: String,
    pub hora: String,
    pub batida_ids: Vec<i64>,
    pub total_repeticoes: usize,
    pub origem: Option<String>,
}

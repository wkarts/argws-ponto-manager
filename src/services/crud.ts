import { invokeCommand } from "./tauri";

export interface ComboOption {
  id: number;
  label: string;
}

export interface ApuracaoDia {
  funcionario_id: number;
  funcionario_nome: string;
  data: string;
  jornada_nome: string;
  tipo_jornada: string;
  horario_esperado_minutos: number;
  trabalhado_minutos: number;
  saldo_minutos: number;
  atraso_minutos: number;
  extra_minutos: number;
  saida_antecipada_minutos: number;
  mensagens: string[];
  batidas: string[];
  ocorrencias: string[];
  minutos_abonados: number;
  abonado: boolean;
  inconsistente: boolean;
}

export interface ApuracaoResumo {
  total_funcionarios: number;
  total_dias: number;
  total_esperado_minutos: number;
  total_trabalhado_minutos: number;
  total_saldo_minutos: number;
  total_atraso_minutos: number;
  total_extra_minutos: number;
  rows: ApuracaoDia[];
}

export interface SyncQueueItem {
  id: number;
  entity_name: string;
  action_name: string;
  record_id?: number | null;
  status: string;
  payload_json?: string | null;
  created_at: string;
}

export interface CompanyFilters {
  search?: string;
  onlyActive?: boolean;
}

export interface EmployeeFilters {
  search?: string;
  empresaId?: number | null;
  onlyActive?: boolean;
}

export interface AfdImportResponse {
  importacao_id: number;
  layout_portaria: string;
  total_linhas: number;
  total_marcacoes: number;
  total_processadas: number;
  total_descartadas: number;
  mensagens: string[];
}

export interface BancoHorasProcessResponse {
  dias_processados: number;
  total_creditos_minutos: number;
  total_debitos_minutos: number;
  saldo_liquido_minutos: number;
}

export type GenericRecord = Record<string, unknown>;

export async function listEntity(entity: string, search = ""): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("entity_list", { entity, search });
}

export async function saveEntity(entity: string, payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("entity_save", { entity, payload });
}

export async function deleteEntity(entity: string, id: number): Promise<boolean> {
  return invokeCommand<boolean>("entity_delete", { entity, id });
}

export async function comboList(entity: string): Promise<ComboOption[]> {
  return invokeCommand<ComboOption[]>("combo_list", { entity });
}

export async function listCompanies(filters: CompanyFilters = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("company_list", { filters });
}

export async function getCompany(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("company_get", { id });
}

export async function saveCompany(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("company_save", { payload });
}

export async function deleteCompany(id: number): Promise<boolean> {
  return invokeCommand<boolean>("company_delete", { id });
}

export async function listEmployees(filters: EmployeeFilters = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("employee_list", { filters });
}

export async function getEmployee(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("employee_get", { id });
}

export async function saveEmployee(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("employee_save", { payload });
}

export async function deleteEmployee(id: number): Promise<boolean> {
  return invokeCommand<boolean>("employee_delete", { id });
}

export async function getBootstrap(): Promise<Record<string, unknown>> {
  return invokeCommand<Record<string, unknown>>("app_bootstrap");
}

export async function listBatidas(filters: Record<string, unknown>): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("batidas_list", { filters });
}

export async function saveBatida(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("batida_save", { payload });
}

export async function deleteBatida(id: number): Promise<boolean> {
  return invokeCommand<boolean>("batida_delete", { id });
}

export async function exportBatidasCsv(filters: Record<string, unknown>): Promise<string> {
  return invokeCommand<string>("exportar_batidas_csv", { filters });
}

export async function apurarPeriodo(payload: Record<string, unknown>): Promise<ApuracaoResumo> {
  return invokeCommand<ApuracaoResumo>("apurar_periodo", { payload });
}

export async function listSyncQueue(): Promise<SyncQueueItem[]> {
  return invokeCommand<SyncQueueItem[]>("sync_queue_list");
}

export async function markSyncQueueSynced(id: number): Promise<boolean> {
  return invokeCommand<boolean>("sync_queue_mark_synced", { id });
}

export async function listJornadas(): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("jornada_list");
}

export async function getJornada(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("jornada_get", { id });
}

export async function saveJornada(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("jornada_save", { payload });
}

export async function deleteJornada(id: number): Promise<boolean> {
  return invokeCommand<boolean>("jornada_delete", { id });
}

export async function comboJornadas(): Promise<ComboOption[]> {
  return invokeCommand<ComboOption[]>("jornada_combo_list");
}

export async function listAfdImports(): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("afd_import_list");
}

export async function importAfdFile(payload: {
  empresaId?: number | null;
  equipamentoId?: number | null;
  fileName: string;
  content: string;
  mode?: string;
}): Promise<AfdImportResponse> {
  return invokeCommand<AfdImportResponse>("afd_import_file", { payload });
}

export async function listBancoHoras(filters: Record<string, unknown>): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("banco_horas_list", { filters });
}

export async function processBancoHoras(payload: Record<string, unknown>): Promise<BancoHorasProcessResponse> {
  return invokeCommand<BancoHorasProcessResponse>("banco_horas_processar_periodo", { payload });
}

export async function saveBancoHorasAjuste(payload: Record<string, unknown>): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("banco_horas_salvar_ajuste", { payload });
}

export async function listOcorrencias(filters: Record<string, unknown>): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("ocorrencia_list", { filters });
}

export async function saveOcorrencia(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("ocorrencia_save", { payload });
}

export async function deleteOcorrencia(id: number): Promise<boolean> {
  return invokeCommand<boolean>("ocorrencia_delete", { id });
}

export async function exportOcorrenciaAnexo(id: number): Promise<string> {
  return invokeCommand<string>("ocorrencia_exportar_anexo", { id });
}

export async function listFechamentos(filters: Record<string, unknown>): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("fechamento_list", { filters });
}

export async function gerarFechamentoRelatorio(payload: Record<string, unknown>): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("fechamento_gerar_relatorio", { payload });
}

export async function listProfiles(sessionToken: string, filters: Record<string, unknown> = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("profile_list", { session_token: sessionToken, filters });
}

export async function getProfile(sessionToken: string, id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("profile_get", { session_token: sessionToken, id });
}

export async function saveProfile(sessionToken: string, payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("profile_save", { session_token: sessionToken, payload });
}

export async function deleteProfile(sessionToken: string, id: number): Promise<boolean> {
  return invokeCommand<boolean>("profile_delete", { session_token: sessionToken, id });
}

export async function listUsers(sessionToken: string, filters: Record<string, unknown> = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("user_list", { session_token: sessionToken, filters });
}

export async function getUser(sessionToken: string, id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("user_get", { session_token: sessionToken, id });
}

export async function saveUser(sessionToken: string, payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("user_save", { session_token: sessionToken, payload });
}

export async function deleteUser(sessionToken: string, id: number): Promise<boolean> {
  return invokeCommand<boolean>("user_delete", { session_token: sessionToken, id });
}

export async function listPermissionCatalog(sessionToken: string): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("permission_catalog", { session_token: sessionToken });
}


export async function getAppMeta(): Promise<Record<string, unknown>> {
  return invokeCommand<Record<string, unknown>>("app_meta");
}

export async function getSystemInfo(): Promise<Record<string, unknown>> {
  return invokeCommand<Record<string, unknown>>("system_info");
}

export async function setSystemDataDir(dataDir: string): Promise<Record<string, unknown>> {
  return invokeCommand<Record<string, unknown>>("system_set_data_dir", { data_dir: dataDir });
}

export async function getLicensingStatus(empresaId?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_status", { empresa_id: empresaId ?? null });
}

export async function startTrialLicense(empresaId?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_start_trial", { empresa_id: empresaId ?? null });
}

export async function exportRepEmpresaTxt(brand: string, empresaId: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("rep_export_empresa_txt", { brand, empresa_id: empresaId });
}

export async function exportRepFuncionariosTxt(brand: string, empresaId: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("rep_export_funcionarios_txt", { brand, empresa_id: empresaId });
}

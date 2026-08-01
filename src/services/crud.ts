import { invokeCommand } from "./tauri";

export type GenericRecord = Record<string, unknown>;

export interface ComboOption {
  id: number;
  label: string;
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

export interface GeneratedReportPayload {
  descricao: string;
  tipoRelatorio: string;
  origemRotina: string;
  formato: string;
  fileName: string;
  mimeType?: string | null;
  competencia?: string | null;
  funcionarioId?: number | null;
  funcionarioNome?: string | null;
  usuarioLogin?: string | null;
  detalhado?: boolean;
  status?: string | null;
  filePath?: string | null;
  contentBase64?: string | null;
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

export interface HolidaySourceSettings extends GenericRecord {
  mode?: string;
  year?: number;
  remote_json_url?: string | null;
  api_url?: string | null;
}

export interface FeriadoRecord extends GenericRecord {
  id?: number;
  data?: string;
  descricao?: string;
  contexto_tipo?: string;
  empresa_id?: number | null;
  departamento_id?: number | null;
  regra_jornada?: string | null;
  regra_compensacao?: string | null;
  observacoes?: string | null;
  ativo?: boolean | number;
  empresa_ids?: number[];
  departamento_ids?: number[];
}


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

export async function cloneEmployee(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("employee_clone", { id });
}

export async function employeeTemplateCsv(): Promise<string> {
  return invokeCommand<string>("employee_template_csv");
}

export async function importEmployeesCsv(payload: Record<string, unknown>): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("employee_import_csv", { payload });
}

export async function deleteEmployee(id: number): Promise<boolean> {
  return invokeCommand<boolean>("employee_delete", { id });
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

export async function listJornadas(): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("jornada_list");
}

export async function getJornada(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("jornada_get", { id });
}

export async function saveJornada(payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("jornada_save", { payload });
}

export async function cloneJornada(id: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("jornada_clone", { id });
}

export async function jornadaPresetList(): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("jornada_preset_list");
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

export async function registerGeneratedReport(payload: GeneratedReportPayload): Promise<number> {
  return invokeCommand<number>("report_generated_register", { payload });
}

export async function listGeneratedReports(filters: Record<string, unknown> = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("report_generated_list", { filters });
}

export async function downloadGeneratedReport(id: number): Promise<{ file_name: string; mime_type: string; content_base64: string }> {
  return invokeCommand<{ file_name: string; mime_type: string; content_base64: string }>("report_generated_download", { id });
}

export async function exportRepEmpresaTxt(brand: string, empresaId: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("rep_export_empresa_txt", { brand, empresa_id: empresaId });
}

export async function exportRepFuncionariosTxt(brand: string, empresaId: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("rep_export_funcionarios_txt", { brand, empresa_id: empresaId });
}

export async function listFeriados(search = ""): Promise<FeriadoRecord[]> {
  return invokeCommand<FeriadoRecord[]>("feriado_list", { search });
}

export async function getFeriado(id: number): Promise<FeriadoRecord> {
  return invokeCommand<FeriadoRecord>("feriado_get", { id });
}

export async function saveFeriado(payload: FeriadoRecord): Promise<FeriadoRecord> {
  return invokeCommand<FeriadoRecord>("feriado_save", { payload });
}

export async function deleteFeriado(id: number): Promise<boolean> {
  return invokeCommand<boolean>("feriado_delete", { id });
}

export async function loadHolidaySourceSettings(): Promise<HolidaySourceSettings> {
  return invokeCommand<HolidaySourceSettings>("holiday_source_load_settings");
}

export async function saveHolidaySourceSettings(payload: HolidaySourceSettings): Promise<HolidaySourceSettings> {
  return invokeCommand<HolidaySourceSettings>("holiday_source_save_settings", { payload });
}

export async function importCompanyDefaultHolidays(empresaId: number, year?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("holiday_source_import_company_year", { empresa_id: empresaId, year: year ?? null });
}

export async function listSyncQueue(): Promise<SyncQueueItem[]> {
  return invokeCommand<SyncQueueItem[]>("sync_queue_list");
}

export async function markSyncQueueSynced(id: number): Promise<boolean> {
  return invokeCommand<boolean>("sync_queue_mark_synced", { id });
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

export async function getUserPolicy(sessionToken: string): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("user_policy_get", { session_token: sessionToken });
}

export async function saveUserPolicy(sessionToken: string, payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("user_policy_save", { session_token: sessionToken, payload });
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

export async function listAppLogs(sessionToken: string, filters: Record<string, unknown> = {}): Promise<GenericRecord[]> {
  return invokeCommand<GenericRecord[]>("app_log_list", { session_token: sessionToken, filters });
}

export async function clearAppLogs(sessionToken: string): Promise<boolean> {
  return invokeCommand<boolean>("app_log_clear", { session_token: sessionToken });
}

export async function getLicensingStatus(sessionToken: string, empresaId?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_status", { session_token: sessionToken, empresa_id: empresaId ?? null });
}

export async function loadLicensingSettings(sessionToken: string, adminUnlockToken?: string | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_load_settings", {
    session_token: sessionToken,
    admin_unlock_token: adminUnlockToken ?? null,
  });
}

export async function saveLicensingSettings(sessionToken: string, adminUnlockToken: string, payload: GenericRecord): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_save_settings", { session_token: sessionToken, admin_unlock_token: adminUnlockToken, payload });
}

export async function getLicensingDeviceInfo(sessionToken: string): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_device_info", { session_token: sessionToken });
}

export async function checkLicensingRuntime(sessionToken: string, empresaId?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_check_runtime", { session_token: sessionToken, empresa_id: empresaId ?? null });
}

export async function startTrialLicense(sessionToken: string, empresaId?: number | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("licensing_start_trial", { session_token: sessionToken, empresa_id: empresaId ?? null });
}

export async function getSupportGuardStatus(sessionToken: string): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("support_guard_status", { session_token: sessionToken });
}

export async function provisionSupportGuard(sessionToken: string, forceRotate = false): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("support_guard_provision", { session_token: sessionToken, force_rotate: forceRotate });
}

export async function enableSupportGuardTotp(sessionToken: string, currentPassword: string, supportSecret: string, totpCode: string): Promise<boolean> {
  return invokeCommand<boolean>("support_guard_enable_totp", {
    session_token: sessionToken,
    current_password: currentPassword,
    support_secret: supportSecret,
    totp_code: totpCode,
  });
}

export async function unlockSupportGuard(sessionToken: string, currentPassword: string, supportSecret: string, totpCode?: string | null, scope = "global"): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("support_guard_unlock", {
    session_token: sessionToken,
    current_password: currentPassword,
    support_secret: supportSecret,
    totp_code: totpCode ?? null,
    scope,
  });
}

export async function lookupCompanyCnpj(documento: string, uf?: string | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("company_lookup_cnpj", { documento, uf: uf ?? null });
}

export async function lookupCompanyIe(documento: string, uf?: string | null): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("company_lookup_ie", { documento, uf: uf ?? null });
}

export async function getBootstrap(): Promise<Record<string, unknown>> {
  return invokeCommand<Record<string, unknown>>("app_bootstrap");
}

export async function getInternalApiStatus(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_status");
}

export async function startInternalApi(host?: string, port?: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_start", { host, port });
}

export async function stopInternalApi(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_stop");
}

export async function restartInternalApi(host?: string, port?: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_restart", { host, port });
}

export async function testInternalApiPort(host: string, port: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_test_port", { host, port });
}

export async function testInternalApi(baseUrl: string, timeoutMs?: number): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("internal_api_test", { base_url: baseUrl, timeout_ms: timeoutMs });
}

export async function getAppServiceStatus(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("app_service_status");
}

export async function startAppService(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("app_service_start");
}

export async function stopAppService(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("app_service_stop");
}

export async function restartAppService(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("app_service_restart");
}

export interface RuntimeSettings {
  internal_api_host: string;
  internal_api_port: number;
  internal_api_base_url: string;
  internal_api_docs_url: string;
  internal_api_auto_start: boolean;
  internal_api_restart_on_config_change: boolean;
  internal_api_require_token: boolean;
  app_service_name: string;
  internal_api_token_header: string;
  internal_api_cors_enabled: boolean;
  internal_api_allow_public_network: boolean;
  internal_api_token: string;
  internal_api_docs_public_local: boolean;
  internal_api_open_scalar_after_start: boolean;
  internal_api_timeout_ms: number;
  internal_api_log_mode: string;
  internal_api_docs_enabled: boolean;
  internal_api_docs_path: string;
  local_web_enabled: boolean;
  local_web_auto_start: boolean;
  local_web_host: string;
  local_web_port: number;
  auxiliary_host: string;
  auxiliary_port: number;
  bridge_core_host: string;
  bridge_core_port: number;
  webhook_enabled: boolean;
  webhook_auto_start: boolean;
  webhook_host: string;
  webhook_port: number;
  webhook_base_path: string;
  webhook_token_required: boolean;
  webhook_token_header: string;
  webhook_token: string;
  webhook_allow_lan: boolean;
  webhook_allow_external: boolean;
  websocket_enabled: boolean;
  websocket_auto_start: boolean;
  websocket_host: string;
  websocket_port: number;
  websocket_path: string;
  websocket_token_required: boolean;
  websocket_token_query: string;
  websocket_token_header: string;
  websocket_token: string;
  websocket_allow_lan: boolean;
  websocket_allow_external: boolean;
  websocket_heartbeat_seconds: number;
  tray_enabled: boolean;
  minimize_to_tray: boolean;
  close_to_tray: boolean;
  start_with_windows: boolean;
  services_auto_start: boolean;
}

export interface RuntimeSettingsPayload extends GenericRecord {
  env_path: string;
  settings: RuntimeSettings;
  ports: Array<Record<string, unknown>>;
  warnings: string[];
}

export async function loadRuntimeSettings(): Promise<RuntimeSettingsPayload> {
  return invokeCommand<RuntimeSettingsPayload>("runtime_settings_load");
}

export async function saveRuntimeSettings(settings: RuntimeSettings): Promise<RuntimeSettingsPayload> {
  return invokeCommand<RuntimeSettingsPayload>("runtime_settings_save", { settings });
}

export async function setStartupWithWindows(enabled: boolean): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("startup_with_windows_set", { enabled });
}

export async function getWebProxyStatus(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("web_proxy_status");
}

export async function startWebProxy(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("web_proxy_start");
}

export async function stopWebProxy(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("web_proxy_stop");
}

export async function restartWebProxy(): Promise<GenericRecord> {
  return invokeCommand<GenericRecord>("web_proxy_restart");
}

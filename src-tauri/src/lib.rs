mod app_state;
mod db;
mod migrations;
mod models;
mod security;
mod timecalc;

mod commands {
    pub mod access;
    pub mod afd;
    pub mod app;
    pub mod auth;
    pub mod banco_horas;
    pub mod companies;
    pub mod employees;
    pub mod entities;
    pub mod feriados;
    pub mod holiday_sources;
    pub mod jornadas;
    pub mod licensing;
    pub mod punches;
    pub mod rep;
    pub mod reports;
    pub mod support;
    pub mod sync;
    pub mod treatments;
}

use app_state::SharedState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(SharedState::new())
        .setup(|app| {
            let state = app.state::<SharedState>();
            state.init().map_err(|err| -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::other(err))
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_bootstrap,
            commands::app::app_meta,
            commands::app::system_info,
            commands::app::system_set_data_dir,
            commands::app::app_log_write,
            commands::app::app_log_list,
            commands::app::app_log_clear,
            commands::auth::auth_login,
            commands::auth::auth_restore,
            commands::auth::auth_logout,
            commands::auth::auth_change_password,
            commands::access::permission_catalog,
            commands::access::profile_list,
            commands::access::profile_get,
            commands::access::profile_save,
            commands::access::profile_delete,
            commands::access::user_list,
            commands::access::user_get,
            commands::access::user_save,
            commands::access::user_delete,
            commands::companies::company_list,
            commands::companies::company_get,
            commands::companies::company_save,
            commands::companies::company_delete,
            commands::companies::company_lookup_cnpj,
            commands::companies::company_lookup_ie,
            commands::licensing::licensing_status,
            commands::licensing::licensing_load_settings,
            commands::licensing::licensing_save_settings,
            commands::licensing::licensing_device_info,
            commands::licensing::licensing_check_runtime,
            commands::licensing::licensing_start_trial,
            commands::support::support_guard_status,
            commands::support::support_guard_provision,
            commands::support::support_guard_enable_totp,
            commands::support::support_guard_unlock,
            commands::employees::employee_list,
            commands::employees::employee_get,
            commands::employees::employee_save,
            commands::employees::employee_clone,
            commands::employees::employee_template_csv,
            commands::employees::employee_import_csv,
            commands::employees::employee_delete,
            commands::feriados::feriado_list,
            commands::feriados::feriado_get,
            commands::feriados::feriado_save,
            commands::feriados::feriado_delete,
            commands::holiday_sources::holiday_source_load_settings,
            commands::holiday_sources::holiday_source_save_settings,
            commands::holiday_sources::holiday_source_import_company_year,
            commands::entities::entity_list,
            commands::entities::entity_save,
            commands::entities::entity_delete,
            commands::entities::combo_list,
            commands::jornadas::jornada_combo_list,
            commands::jornadas::jornada_preset_list,
            commands::jornadas::jornada_list,
            commands::jornadas::jornada_get,
            commands::jornadas::jornada_save,
            commands::jornadas::jornada_clone,
            commands::jornadas::jornada_delete,
            commands::punches::batidas_list,
            commands::punches::batida_save,
            commands::punches::batida_delete,
            commands::reports::apurar_periodo,
            commands::reports::exportar_batidas_csv,
            commands::reports::report_generated_register,
            commands::reports::report_generated_list,
            commands::reports::report_generated_download,
            commands::rep::rep_export_empresa_txt,
            commands::rep::rep_export_funcionarios_txt,
            commands::afd::afd_import_list,
            commands::afd::afd_import_file,
            commands::banco_horas::banco_horas_list,
            commands::banco_horas::banco_horas_processar_periodo,
            commands::banco_horas::banco_horas_salvar_ajuste,
            commands::treatments::ocorrencia_list,
            commands::treatments::ocorrencia_save,
            commands::treatments::ocorrencia_delete,
            commands::treatments::ocorrencia_exportar_anexo,
            commands::treatments::fechamento_list,
            commands::treatments::fechamento_gerar_relatorio,
            commands::sync::sync_queue_list,
            commands::sync::sync_queue_mark_synced,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar a aplicação Tauri");
}

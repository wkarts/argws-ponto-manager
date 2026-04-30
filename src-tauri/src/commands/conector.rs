use crate::services::conector_client::ConectorClient;
use std::env;
use tauri::command;

fn build_client() -> Result<ConectorClient, String> {
    let base_url = env::var("PONTO_CONECTOR_URL")
        .map_err(|_| "Variável PONTO_CONECTOR_URL não configurada.".to_string())?;
    let api_token = env::var("PONTO_CONECTOR_TOKEN")
        .map_err(|_| "Variável PONTO_CONECTOR_TOKEN não configurada.".to_string())?;

    ConectorClient::new(base_url, api_token)
}

#[command]
pub async fn conector_testar() -> Result<String, String> {
    let client = build_client()?;
    println!("[Conector] Teste de conexão iniciado");
    client.testar_conexao().await
}

#[command]
pub async fn conector_batidas_por_data(
    device_id: String,
    data_inicio: String,
    data_fim: String,
) -> Result<serde_json::Value, String> {
    let client = build_client()?;
    println!(
        "[Conector] Coleta por data iniciada device_id={}",
        device_id
    );
    client
        .obter_batidas_por_data(&device_id, &data_inicio, &data_fim)
        .await
}

#[command]
pub async fn conector_batidas_por_nsr(
    device_id: String,
    nsr_inicio: i64,
    nsr_fim: Option<i64>,
) -> Result<serde_json::Value, String> {
    let client = build_client()?;
    println!("[Conector] Coleta por NSR iniciada device_id={}", device_id);
    client
        .obter_batidas_por_nsr(&device_id, nsr_inicio, nsr_fim)
        .await
}

#[command]
pub async fn conector_baixar_afd(device_id: String) -> Result<Vec<u8>, String> {
    let client = build_client()?;
    println!("[Conector] Download AFD iniciado device_id={}", device_id);
    client.obter_afd(&device_id).await
}

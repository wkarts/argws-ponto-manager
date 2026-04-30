use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct ConectorClient {
    pub base_url: String,
    pub api_token: String,
    client: Client,
}

impl ConectorClient {
    pub fn new(base_url: String, api_token: String, timeout_secs: u64) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs.max(5)))
            .build()
            .map_err(|err| format!("Falha ao criar cliente HTTP do conector: {err}"))?;

        Ok(Self {
            base_url,
            api_token,
            client,
        })
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("Authorization", format!("Bearer {}", self.api_token))
    }

    pub async fn testar_conexao(&self) -> Result<String, String> {
        let url = format!("{}/health", self.base_url);
        let res = self.auth(self.client.get(&url)).send().await;

        match res {
            Ok(r) => Ok(r.status().to_string()),
            Err(e) => Err(format!("Erro conexão: {e}")),
        }
    }

    pub async fn coletar_batidas(&self, device_id: &str, payload: &Value) -> Result<Value, String> {
        let url = format!(
            "{}/api/devices/{}/punches/collect",
            self.base_url, device_id
        );
        let res = self.auth(self.client.post(&url).json(payload)).send().await;
        match res {
            Ok(r) => r.json().await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn baixar_afd(&self, device_id: &str, payload: &Value) -> Result<Vec<u8>, String> {
        let url = format!("{}/api/devices/{}/afd/download", self.base_url, device_id);
        let res = self.auth(self.client.post(&url).json(payload)).send().await;

        match res {
            Ok(r) => r
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

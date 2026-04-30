use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct ConectorClient {
    pub base_url: String,
    pub api_token: String,
    client: Client,
}

impl ConectorClient {
    pub fn new(base_url: String, api_token: String) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
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
        let url = format!("{}/api/health", self.base_url);

        let res = self.auth(self.client.get(&url)).send().await;

        match res {
            Ok(r) => Ok(r.status().to_string()),
            Err(e) => Err(format!("Erro conexão: {e}")),
        }
    }

    pub async fn obter_batidas_por_data(
        &self,
        device_id: &str,
        data_inicio: &str,
        data_fim: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/punches/date", self.base_url);

        let payload = serde_json::json!({
            "device_id": device_id,
            "data_inicio": data_inicio,
            "data_fim": data_fim
        });

        let res = self
            .auth(self.client.post(&url).json(&payload))
            .send()
            .await;

        match res {
            Ok(r) => r.json().await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn obter_batidas_por_nsr(
        &self,
        device_id: &str,
        nsr_inicio: i64,
        nsr_fim: Option<i64>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/punches/nsr", self.base_url);

        let payload = serde_json::json!({
            "device_id": device_id,
            "nsr_inicio": nsr_inicio,
            "nsr_fim": nsr_fim
        });

        let res = self
            .auth(self.client.post(&url).json(&payload))
            .send()
            .await;

        match res {
            Ok(r) => r.json().await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn obter_afd(&self, device_id: &str) -> Result<Vec<u8>, String> {
        let url = format!("{}/api/afd/{}", self.base_url, device_id);

        let res = self.auth(self.client.get(&url)).send().await;

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

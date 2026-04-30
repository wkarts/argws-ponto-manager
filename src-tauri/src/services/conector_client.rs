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
            base_url: base_url.trim().trim_end_matches('/').to_string(),
            api_token,
            client,
        })
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_token.trim().is_empty() {
            req
        } else {
            req.header("Authorization", format!("Bearer {}", self.api_token))
                .header("X-API-Key", self.api_token.clone())
        }
    }

    async fn decode_json_result(&self, response: reqwest::Response) -> Result<Value, String> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| format!("Falha ao ler resposta do conector: {err}"))?;

        if !status.is_success() {
            return Err(format!(
                "Conector retornou HTTP {}. Resposta: {}",
                status,
                body.chars().take(500).collect::<String>()
            ));
        }

        let value: Value = serde_json::from_str(&body).map_err(|err| {
            format!(
                "Resposta inválida da API do conector. Esperado JSON. Erro: {err}. Resposta: {}",
                body.chars().take(500).collect::<String>()
            )
        })?;

        if let Some(err) = value.get("Err").and_then(Value::as_str) {
            return Err(err.to_string());
        }
        if let Some(ok) = value.get("Ok") {
            return Ok(ok.clone());
        }
        if let Some(err) = value.get("error").and_then(Value::as_str) {
            return Err(err.to_string());
        }
        if let Some(message) = value.get("message").and_then(Value::as_str) {
            if value.get("success").and_then(Value::as_bool) == Some(false) {
                return Err(message.to_string());
            }
        }

        Ok(value)
    }

    pub async fn testar_conexao(&self) -> Result<String, String> {
        let url = format!("{}/health", self.base_url);
        self.auth(self.client.get(&url))
            .send()
            .await
            .map(|response| response.status().to_string())
            .map_err(|err| {
                format!(
                    "Erro ao conectar na API do Ponto Manager Conector em {}. Verifique URL, porta, firewall e se o serviço está iniciado. Detalhe: {err}",
                    self.base_url
                )
            })
    }

    pub async fn coletar_batidas(&self, device_id: &str, payload: &Value) -> Result<Value, String> {
        let url = format!(
            "{}/api/devices/{}/punches/collect",
            self.base_url, device_id
        );
        let response = self
            .auth(self.client.post(&url).json(payload))
            .send()
            .await
            .map_err(|err| {
                format!(
                    "Erro ao enviar requisição para coleta de batidas no conector. URL: {}. Detalhe: {err}",
                    url
                )
            })?;
        self.decode_json_result(response).await
    }

    pub async fn baixar_afd(&self, device_id: &str, payload: &Value) -> Result<Value, String> {
        let url = format!("{}/api/devices/{}/afd/download", self.base_url, device_id);
        let response = self
            .auth(self.client.post(&url).json(payload))
            .send()
            .await
            .map_err(|err| {
                format!(
                    "Erro ao enviar requisição para download de AFD no conector. URL: {}. Detalhe: {err}",
                    url
                )
            })?;
        self.decode_json_result(response).await
    }
}

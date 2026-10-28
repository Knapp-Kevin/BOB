//! Explicitly authorized text generation over fixed provider endpoints.
//! No caller-supplied URL, no automatic model fallback, and no credential logging.

use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use anyhow::{bail, Context, Result};
use reqwest::{redirect::Policy, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const LM_STUDIO_ENDPOINT: &str = "http://127.0.0.1:1234/v1/chat/completions";
const TIMEOUT: Duration = Duration::from_secs(60);
const MAX_PROMPT_BYTES: usize = 4_000;
const MAX_RESPONSE_BYTES: usize = 64_000;

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    messages: [Message<'a>; 1],
    stream: bool,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: OutputMessage,
}

#[derive(Deserialize)]
struct OutputMessage {
    content: Option<String>,
}

pub struct InferenceTransport {
    cloud: Client,
    local: Client,
}

impl InferenceTransport {
    pub fn new() -> Result<Self> {
        let cloud = Client::builder()
            .redirect(Policy::none())
            .timeout(TIMEOUT)
            .build()
            .context("initialize cloud inference transport")?;
        let local = Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .timeout(TIMEOUT)
            .build()
            .context("initialize local inference transport")?;
        Ok(Self { cloud, local })
    }

    /// The allowlist and billing gate runs before constructing or sending a request.
    pub async fn generate(
        &self,
        authorization: &InferenceAuthorization,
        provider: InferenceProvider,
        model: &str,
        prompt: &str,
        openrouter_api_key: Option<&str>,
    ) -> Result<String> {
        authorization
            .authorize(provider, model)
            .map_err(|reason| anyhow::anyhow!("inference authorization denied: {reason:?}"))?;
        if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
            bail!("invalid inference prompt length");
        }

        let (client, endpoint) = match provider {
            InferenceProvider::OpenRouter => (&self.cloud, OPENROUTER_ENDPOINT),
            InferenceProvider::LmStudio => (&self.local, LM_STUDIO_ENDPOINT),
            InferenceProvider::Ollama => bail!("Ollama uses the existing local adapter"),
        };
        let mut request = client.post(endpoint).json(&Request {
            model,
            messages: [Message {
                role: "user",
                content: prompt,
            }],
            stream: false,
            max_tokens: 512,
        });
        if provider == InferenceProvider::OpenRouter {
            let key = openrouter_api_key.filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("OpenRouter credential missing"))?;
            request = request.bearer_auth(key);
        }
        let response = request.send().await.context("inference provider unavailable")?;
        if !response.status().is_success() {
            bail!("inference provider rejected request: HTTP {}", response.status().as_u16());
        }
        // Bound allocation while reading, not only after downloading a potentially
        // unbounded provider response into memory.
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.context("read inference response")? {
            if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
                bail!("inference response exceeds size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        let parsed: Response = serde_json::from_slice(&bytes)
            .context("invalid inference response")?;
        let output = parsed.choices.into_iter().next()
            .and_then(|choice| choice.message.content)
            .unwrap_or_default();
        if output.trim().is_empty() {
            bail!("inference response is empty");
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference_policy::InferenceAuthorization;

    #[tokio::test]
    async fn denies_unapproved_models_before_network() {
        let transport = InferenceTransport::new().unwrap();
        let result = transport.generate(
            &InferenceAuthorization::default(),
            InferenceProvider::OpenRouter,
            "unapproved/model",
            "hello",
            None,
        ).await;
        assert!(result.unwrap_err().to_string().contains("authorization denied"));
    }

    #[tokio::test]
    async fn blocks_cloud_without_billing_authorization_before_network() {
        let transport = InferenceTransport::new().unwrap();
        let mut authorization = InferenceAuthorization::default();
        authorization.openrouter_models.insert("approved/model".into());
        let result = transport.generate(
            &authorization,
            InferenceProvider::OpenRouter,
            "approved/model",
            "hello",
            None,
        ).await;
        assert!(result.unwrap_err().to_string().contains("authorization denied"));
    }
}

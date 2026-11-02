//! Explicitly authorized text generation over fixed provider endpoints.
//! No caller-supplied URL, no automatic model fallback, and no credential logging.
//!
//! OpenRouter routes an approved model to one of several upstream providers.
//! Selecting a model does not fix that upstream provider or its data policy, so
//! every request restricts routing to providers that do not collect prompt data
//! and never supplies alternate models.
//!
//! The OpenAI and Anthropic APIs are called directly at their fixed vendor
//! endpoints with the same bounds: one approved model, one user message, no
//! streaming, no tools, and a hard output-token cap.

use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use anyhow::{bail, Context, Result};
use reqwest::{redirect::Policy, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OPENROUTER_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const LM_STUDIO_ENDPOINT: &str = "http://127.0.0.1:1234/v1/chat/completions";
const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_PROMPT_BYTES: usize = 4_000;
const MAX_RESPONSE_BYTES: usize = 64_000;
const MAX_OUTPUT_TOKENS: u32 = 512;
// Must match the canonical chat-entry persistence limit.
const MAX_GENERATION_BYTES: usize = 8_000;

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    messages: [Message<'a>; 1],
    stream: bool,
    // OpenAI's current Chat Completions API takes `max_completion_tokens`;
    // OpenRouter and LM Studio take `max_tokens`. Exactly one is sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<ProviderRouting>,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: [Message<'a>; 1],
    stream: bool,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    content: Vec<AnthropicBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Serialize)]
struct ProviderRouting {
    data_collection: &'static str,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    message: OutputMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OutputMessage {
    content: Option<String>,
}

pub struct InferenceTransport {
    cloud: Client,
    local: Client,
    openrouter_endpoint: String,
    lm_studio_endpoint: String,
    openai_endpoint: String,
    anthropic_endpoint: String,
}

impl InferenceTransport {
    pub fn new() -> Result<Self> {
        Self::with_endpoints(OPENROUTER_ENDPOINT.into(), LM_STUDIO_ENDPOINT.into())
    }

    fn with_endpoints(openrouter_endpoint: String, lm_studio_endpoint: String) -> Result<Self> {
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
        Ok(Self {
            cloud,
            local,
            openrouter_endpoint,
            lm_studio_endpoint,
            openai_endpoint: OPENAI_ENDPOINT.into(),
            anthropic_endpoint: ANTHROPIC_ENDPOINT.into(),
        })
    }

    /// Test-only transport whose cloud traffic (OpenRouter, OpenAI, Anthropic)
    /// goes to a local mock server.
    #[cfg(test)]
    pub(crate) fn for_test(cloud_endpoint: String) -> Result<Self> {
        let mut transport =
            Self::with_endpoints(cloud_endpoint.clone(), LM_STUDIO_ENDPOINT.into())?;
        transport.openai_endpoint = cloud_endpoint.clone();
        transport.anthropic_endpoint = cloud_endpoint;
        transport.cloud = Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(transport)
    }

    /// The allowlist and billing gate runs before constructing or sending a request.
    pub async fn generate(
        &self,
        authorization: &InferenceAuthorization,
        provider: InferenceProvider,
        model: &str,
        prompt: &str,
        api_key: Option<&str>,
    ) -> Result<String> {
        authorization
            .authorize(provider, model)
            .map_err(|reason| anyhow::anyhow!("inference authorization denied: {reason:?}"))?;
        if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
            bail!("invalid inference prompt length");
        }
        let messages = [Message {
            role: "user",
            content: prompt,
        }];
        let chat = |routing, openai: bool| Request {
            model,
            messages: [Message {
                role: "user",
                content: prompt,
            }],
            stream: false,
            max_tokens: (!openai).then_some(MAX_OUTPUT_TOKENS),
            max_completion_tokens: openai.then_some(MAX_OUTPUT_TOKENS),
            store: openai.then_some(false),
            provider: routing,
        };
        let required_key = || {
            api_key
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("inference provider credential missing"))
        };

        let request = match provider {
            InferenceProvider::OpenRouter => self
                .cloud
                .post(&self.openrouter_endpoint)
                .bearer_auth(required_key()?)
                .json(&chat(
                    Some(ProviderRouting {
                        data_collection: "deny",
                    }),
                    false,
                )),
            InferenceProvider::OpenAi => self
                .cloud
                .post(&self.openai_endpoint)
                .bearer_auth(required_key()?)
                .json(&chat(None, true)),
            InferenceProvider::Anthropic => self
                .cloud
                .post(&self.anthropic_endpoint)
                .header("x-api-key", required_key()?)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .json(&AnthropicRequest {
                    model,
                    max_tokens: MAX_OUTPUT_TOKENS,
                    messages,
                    stream: false,
                }),
            InferenceProvider::LmStudio => self
                .local
                .post(&self.lm_studio_endpoint)
                .json(&chat(None, false)),
            InferenceProvider::Ollama => bail!("Ollama uses the existing local adapter"),
            InferenceProvider::ClaudeCli | InferenceProvider::CodexCli => {
                bail!("CLI runtimes use the external-process adapter")
            }
        };
        // Error strings never include the request, headers, or credential.
        let mut response = request
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("inference provider unavailable"))?;
        if !response.status().is_success() {
            bail!(
                "inference provider rejected request: HTTP {}",
                response.status().as_u16()
            );
        }
        // Bound allocation while reading, not only after downloading a potentially
        // unbounded provider response into memory.
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| anyhow::anyhow!("could not read inference response"))?
        {
            if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
                bail!("inference response exceeds size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        if provider == InferenceProvider::Anthropic {
            parse_anthropic_response(&bytes)
        } else {
            parse_response(&bytes)
        }
    }
}

fn parse_anthropic_response(bytes: &[u8]) -> Result<String> {
    let parsed: AnthropicResponse =
        serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("invalid inference response"))?;
    if parsed.kind != "message" {
        bail!("inference provider reported an error");
    }
    if parsed.stop_reason.as_deref() == Some("refusal") {
        bail!("inference provider declined the request");
    }
    let output: String = parsed
        .content
        .into_iter()
        .filter(|block| block.kind == "text")
        .filter_map(|block| block.text)
        .collect();
    validate_generated_text(&output)?;
    Ok(output)
}

fn parse_response(bytes: &[u8]) -> Result<String> {
    let parsed: Response =
        serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("invalid inference response"))?;
    if parsed.error.is_some() {
        bail!("inference provider reported an error");
    }
    let choice = parsed
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("inference response is empty"))?;
    if choice.finish_reason.as_deref() == Some("error") {
        bail!("inference provider reported an error");
    }
    let output = choice.message.content.unwrap_or_default();
    validate_generated_text(&output)?;
    Ok(output)
}

fn validate_generated_text(output: &str) -> Result<()> {
    if output.trim().is_empty() {
        bail!("inference response is empty");
    }
    if output.len() > MAX_GENERATION_BYTES {
        bail!("inference response exceeds chat persistence limit");
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod mock {
    //! Minimal one-shot HTTP server so transport tests never reach a real provider.
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    pub struct MockServer {
        pub endpoint: String,
        pub requests: Arc<Mutex<Vec<String>>>,
    }

    /// Serves `status` and `body` to every connection and records each raw request.
    pub fn serve(status: u16, body: String) -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!(
            "http://{}/api/v1/chat/completions",
            listener.local_addr().unwrap()
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let raw = read_request(&mut stream);
                recorded.lock().unwrap().push(raw);
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        MockServer { endpoint, requests }
    }

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut data = Vec::new();
        let mut buffer = [0u8; 4096];
        while let Ok(read) = stream.read(&mut buffer) {
            if read == 0 {
                break;
            }
            data.extend_from_slice(&buffer[..read]);
            let text = String::from_utf8_lossy(&data);
            if let Some(header_end) = text.find("\r\n\r\n") {
                let length = text[..header_end]
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())?
                    })
                    .unwrap_or(0);
                if data.len() >= header_end + 4 + length {
                    break;
                }
            }
        }
        String::from_utf8_lossy(&data).into_owned()
    }

    pub fn anthropic_message(content: &str) -> String {
        serde_json::json!({
            "id": "msg_test",
            "type": "message",
            "role": "assistant",
            "model": "claude-test-5",
            "content": [{ "type": "text", "text": content }],
            "stop_reason": "end_turn"
        })
        .to_string()
    }

    pub fn completion(content: &str) -> String {
        serde_json::json!({
            "model": "vendor/approved-model",
            "choices": [{ "message": { "role": "assistant", "content": content }, "finish_reason": "stop" }]
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::mock::{completion, serve};
    use super::*;
    use crate::inference_policy::InferenceAuthorization;

    const MODEL: &str = "vendor/approved-model";
    const KEY: &str = "sk-or-v1-test-secret-for-tests";

    fn authorized() -> InferenceAuthorization {
        let mut authorization = InferenceAuthorization {
            cloud_enabled: true,
            metered_enabled: true,
            ..Default::default()
        };
        authorization.openrouter_models.insert(MODEL.into());
        authorization
    }

    fn run(
        transport: &InferenceTransport,
        authorization: &InferenceAuthorization,
        model: &str,
        prompt: &str,
        key: Option<&str>,
    ) -> Result<String> {
        tauri::async_runtime::block_on(transport.generate(
            authorization,
            InferenceProvider::OpenRouter,
            model,
            prompt,
            key,
        ))
    }

    #[test]
    fn rejects_generated_text_that_cannot_be_saved() {
        assert!(validate_generated_text("Useful reply").is_ok());
        assert!(validate_generated_text("   ").is_err());
        assert!(validate_generated_text(&"🧠".repeat(2_001)).is_err());
        assert!(validate_generated_text(&"x".repeat(8_000)).is_ok());
    }

    #[test]
    fn denied_requests_never_reach_the_network() {
        let server = serve(200, completion("should not be used"));
        let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
        let mut cloud_only = authorized();
        cloud_only.metered_enabled = false;
        let mut metered_only = authorized();
        metered_only.cloud_enabled = false;
        let cases: [(&InferenceAuthorization, &str, Option<&str>); 5] = [
            (&InferenceAuthorization::default(), MODEL, Some(KEY)),
            (&cloud_only, MODEL, Some(KEY)),
            (&metered_only, MODEL, Some(KEY)),
            (&authorized(), "vendor/unapproved-model", Some(KEY)),
            (&authorized(), MODEL, None),
        ];
        for (authorization, model, key) in cases {
            assert!(run(&transport, authorization, model, "hello", key).is_err());
        }
        assert!(run(&transport, &authorized(), MODEL, "hello", Some("   ")).is_err());
        assert!(run(&transport, &authorized(), MODEL, "   ", Some(KEY)).is_err());
        let oversized = "x".repeat(MAX_PROMPT_BYTES + 1);
        let error = run(&transport, &authorized(), MODEL, &oversized, Some(KEY)).unwrap_err();
        assert!(error.to_string().contains("prompt length"));
        std::thread::sleep(Duration::from_millis(100));
        assert!(server.requests.lock().unwrap().is_empty());
    }

    #[test]
    fn authorized_request_uses_fixed_model_privacy_routing_and_bearer_key() {
        let server = serve(200, completion("Start with the smallest step."));
        let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
        let reply = run(&transport, &authorized(), MODEL, "Help me start", Some(KEY)).unwrap();
        assert_eq!(reply, "Start with the smallest step.");
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let raw = &requests[0];
        assert!(raw.starts_with("POST /api/v1/chat/completions "));
        assert!(raw
            .to_ascii_lowercase()
            .contains(&format!("authorization: bearer {KEY}")));
        let body: serde_json::Value =
            serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["model"], MODEL);
        assert_eq!(body["stream"], false);
        assert_eq!(body["max_tokens"], MAX_OUTPUT_TOKENS);
        assert!(body.get("max_completion_tokens").is_none());
        assert_eq!(body["provider"]["data_collection"], "deny");
        assert!(body.get("models").is_none(), "no alternate model fallback");
        assert!(body.get("route").is_none());
        assert_eq!(body["messages"][0]["content"], "Help me start");
    }

    #[test]
    fn invalid_provider_responses_are_rejected_without_leaking_key() {
        let oversized = format!(
            "{{\"choices\":[{{\"message\":{{\"content\":\"{}\"}}}}]}}",
            "x".repeat(MAX_RESPONSE_BYTES)
        );
        let too_long_for_chat = completion(&"y".repeat(MAX_GENERATION_BYTES + 1));
        let cases = [
            (200, "not json".to_owned()),
            (200, "{}".to_owned()),
            (200, "{\"choices\":[]}".to_owned()),
            (200, "{\"choices\":[{\"message\":{\"content\":null}}]}".to_owned()),
            (200, completion("   ")),
            (
                200,
                "{\"choices\":[{\"message\":{\"content\":\"partial\"},\"finish_reason\":\"error\"}]}"
                    .to_owned(),
            ),
            (
                200,
                "{\"choices\":[{\"message\":{\"content\":\"x\"}}],\"error\":{\"code\":502}}"
                    .to_owned(),
            ),
            (200, oversized),
            (200, too_long_for_chat),
            (401, "{\"error\":{\"message\":\"bad key\"}}".to_owned()),
            (402, "{\"error\":{\"message\":\"insufficient credits\"}}".to_owned()),
            (500, "{}".to_owned()),
        ];
        for (status, body) in cases {
            let server = serve(status, body);
            let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
            let error = run(&transport, &authorized(), MODEL, "hello", Some(KEY)).unwrap_err();
            let message = format!("{error:#} {error:?}");
            assert!(!message.contains(KEY), "credential leaked: {message}");
            assert!(!message.contains("sk-or"), "credential leaked: {message}");
        }
    }

    #[test]
    fn unreachable_provider_error_does_not_leak_key() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
        drop(listener);
        let transport = InferenceTransport::for_test(endpoint).unwrap();
        let error = run(&transport, &authorized(), MODEL, "hello", Some(KEY)).unwrap_err();
        let message = format!("{error:#} {error:?}");
        assert!(message.contains("unavailable"));
        assert!(!message.contains(KEY));
    }

    #[test]
    fn production_endpoint_is_fixed_openrouter_https() {
        let transport = InferenceTransport::new().unwrap();
        assert_eq!(transport.openrouter_endpoint, OPENROUTER_ENDPOINT);
        assert!(OPENROUTER_ENDPOINT.starts_with("https://openrouter.ai/"));
        assert_eq!(transport.openai_endpoint, OPENAI_ENDPOINT);
        assert!(OPENAI_ENDPOINT.starts_with("https://api.openai.com/"));
        assert_eq!(transport.anthropic_endpoint, ANTHROPIC_ENDPOINT);
        assert!(ANTHROPIC_ENDPOINT.starts_with("https://api.anthropic.com/"));
    }

    fn vendor_authorized() -> InferenceAuthorization {
        let mut authorization = InferenceAuthorization {
            cloud_enabled: true,
            metered_enabled: true,
            ..Default::default()
        };
        authorization.openai_models.insert("gpt-test-mini".into());
        authorization
            .anthropic_models
            .insert("claude-test-5".into());
        authorization
    }

    fn run_vendor(
        transport: &InferenceTransport,
        authorization: &InferenceAuthorization,
        provider: InferenceProvider,
        prompt: &str,
        key: Option<&str>,
    ) -> Result<String> {
        let model = match provider {
            InferenceProvider::OpenAi => "gpt-test-mini",
            _ => "claude-test-5",
        };
        tauri::async_runtime::block_on(transport.generate(
            authorization,
            provider,
            model,
            prompt,
            key,
        ))
    }

    #[test]
    fn openai_request_is_bounded_unstored_and_bearer_authenticated() {
        let server = serve(200, completion("One small step."));
        let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
        let reply = run_vendor(
            &transport,
            &vendor_authorized(),
            InferenceProvider::OpenAi,
            "Help me start",
            Some(KEY),
        )
        .unwrap();
        assert_eq!(reply, "One small step.");
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let raw = &requests[0];
        assert!(raw
            .to_ascii_lowercase()
            .contains(&format!("authorization: bearer {KEY}")));
        let body: serde_json::Value =
            serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["model"], "gpt-test-mini");
        assert_eq!(body["max_completion_tokens"], MAX_OUTPUT_TOKENS);
        assert!(body.get("max_tokens").is_none());
        assert_eq!(body["store"], false);
        assert_eq!(body["stream"], false);
        assert!(body.get("tools").is_none());
        assert!(body.get("provider").is_none());
    }

    #[test]
    fn anthropic_request_uses_messages_api_headers_and_parses_text_blocks() {
        let server = serve(200, mock::anthropic_message("Pick one thing."));
        let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
        let reply = run_vendor(
            &transport,
            &vendor_authorized(),
            InferenceProvider::Anthropic,
            "Help me start",
            Some(KEY),
        )
        .unwrap();
        assert_eq!(reply, "Pick one thing.");
        let requests = server.requests.lock().unwrap();
        let raw = requests[0].to_ascii_lowercase();
        assert!(raw.contains(&format!("x-api-key: {KEY}")));
        assert!(raw.contains("anthropic-version: 2023-06-01"));
        assert!(!raw.contains("authorization: bearer"));
        let body: serde_json::Value =
            serde_json::from_str(requests[0].split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["model"], "claude-test-5");
        assert_eq!(body["max_tokens"], MAX_OUTPUT_TOKENS);
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(body.get("tools").is_none());
        assert!(body.get("system").is_none());
    }

    #[test]
    fn vendor_requests_are_denied_before_network_without_full_authorization() {
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
        let mut no_metered = vendor_authorized();
        no_metered.metered_enabled = false;
        let mut no_cloud = vendor_authorized();
        no_cloud.cloud_enabled = false;
        for provider in [InferenceProvider::OpenAi, InferenceProvider::Anthropic] {
            for authorization in [&InferenceAuthorization::default(), &no_metered, &no_cloud] {
                assert!(run_vendor(&transport, authorization, provider, "hi", Some(KEY)).is_err());
            }
            assert!(run_vendor(&transport, &vendor_authorized(), provider, "hi", None).is_err());
        }
        std::thread::sleep(Duration::from_millis(100));
        assert!(server.requests.lock().unwrap().is_empty());
    }

    #[test]
    fn vendor_failures_and_malformed_output_are_rejected_without_leaking_key() {
        let cases = [
            (InferenceProvider::OpenAi, 401, "{\"error\":{\"message\":\"bad key\"}}".to_owned()),
            (InferenceProvider::OpenAi, 429, "{}".to_owned()),
            (InferenceProvider::OpenAi, 200, "not json".to_owned()),
            (InferenceProvider::Anthropic, 401, "{\"type\":\"error\"}".to_owned()),
            (InferenceProvider::Anthropic, 529, "{}".to_owned()),
            (InferenceProvider::Anthropic, 200, "{\"type\":\"error\",\"error\":{}}".to_owned()),
            (InferenceProvider::Anthropic, 200, "{\"type\":\"message\",\"content\":[]}".to_owned()),
            (
                InferenceProvider::Anthropic,
                200,
                "{\"type\":\"message\",\"content\":[{\"type\":\"text\",\"text\":\"no\"}],\"stop_reason\":\"refusal\"}".to_owned(),
            ),
            (InferenceProvider::Anthropic, 200, mock::anthropic_message(&"z".repeat(MAX_GENERATION_BYTES + 1))),
        ];
        for (provider, status, body) in cases {
            let server = serve(status, body);
            let transport = InferenceTransport::for_test(server.endpoint.clone()).unwrap();
            let error = run_vendor(&transport, &vendor_authorized(), provider, "hi", Some(KEY))
                .unwrap_err();
            let message = format!("{error:#} {error:?}");
            assert!(!message.contains(KEY), "credential leaked: {message}");
        }
    }
}

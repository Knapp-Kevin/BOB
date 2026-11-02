//! Direct-vendor API adapters (OpenAI, Anthropic), following the OpenRouter
//! pattern.
//!
//! Each adapter is off by default and classified `metered`. Generation
//! requires, all at once: one exact approved model, persisted cloud and
//! metered permissions, a credential in the OS secret store, and a Rust-held
//! session grant that starts empty on every launch, is bound to the model it
//! was granted for, and is revoked by any model, permission, or credential
//! change. At most one cloud Chat backend (including OpenRouter) can hold a
//! session grant at a time, so the backend never changes silently.
//!
//! Metered use also requires a B.O.B.-side monthly spending limit and the
//! model's token prices. Each request reserves its worst-case cost against the
//! limit before it is sent and is then charged its actual cost (see `spend`).
//!
//! B.O.B. is API-only: vendor agent CLIs (Claude Code, Codex) are out of scope
//! (see `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`).

use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use crate::inference_transport::{
    InferenceTransport, ProviderRejected, MAX_OUTPUT_TOKENS, MAX_PROMPT_BYTES,
};
use crate::openrouter::OpenRouterCredentials;
use crate::secrets::{OsSecretStore, SecretName, SecretStore};
use crate::spend::{self, TokenPrices};
use crate::state::{SpendLimits, Store};
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdapterId {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
}

impl AdapterId {
    pub const ALL: [AdapterId; 2] = [AdapterId::OpenAi, AdapterId::Anthropic];

    /// Stable key used in SQLite and by the frontend.
    pub fn key(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
        }
    }

    fn provider(self) -> InferenceProvider {
        match self {
            Self::OpenAi => InferenceProvider::OpenAi,
            Self::Anthropic => InferenceProvider::Anthropic,
        }
    }

    fn secret(self) -> SecretName {
        match self {
            Self::OpenAi => SecretName::OpenAiApiKey,
            Self::Anthropic => SecretName::AnthropicApiKey,
        }
    }
}

fn authorization_for(
    adapter: AdapterId,
    model: &str,
    cloud: bool,
    metered: bool,
) -> InferenceAuthorization {
    let mut authorization = InferenceAuthorization {
        cloud_enabled: cloud,
        metered_enabled: metered,
        ..Default::default()
    };
    let allowed = match adapter {
        AdapterId::OpenAi => &mut authorization.openai_models,
        AdapterId::Anthropic => &mut authorization.anthropic_models,
    };
    allowed.insert(model.to_owned());
    authorization
}

fn persisted_authorization(
    store: &Store,
    adapter: AdapterId,
) -> Result<(InferenceAuthorization, String)> {
    let (model, cloud, metered) = store.adapter_settings(adapter.key())?;
    let model =
        model.ok_or_else(|| anyhow!("{} model has not been authorized", adapter.label()))?;
    Ok((authorization_for(adapter, &model, cloud, metered), model))
}

/// The configured monthly limit and prices, or a refusal if any is missing.
fn required_limits(store: &Store, adapter: AdapterId) -> Result<(i64, TokenPrices)> {
    let limits = store.adapter_limits(adapter.key())?;
    match (
        limits.monthly_cap_micros,
        limits.input_price_micros,
        limits.output_price_micros,
    ) {
        (Some(cap), Some(input), Some(output)) => Ok((cap, TokenPrices { input, output })),
        _ => bail!("{} monthly spending limit not configured", adapter.label()),
    }
}

pub struct Adapters {
    secrets: Arc<dyn SecretStore>,
    sessions: Mutex<HashMap<AdapterId, String>>,
}

impl Adapters {
    pub fn new() -> Self {
        Self::with_secrets(Arc::new(OsSecretStore::default()))
    }

    fn with_secrets(secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            secrets,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn sessions(&self) -> std::sync::MutexGuard<'_, HashMap<AdapterId, String>> {
        // A poisoned lock still holds a valid map; revocation must always work.
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn active_session(&self) -> Option<AdapterId> {
        self.sessions().keys().next().copied()
    }

    fn session_model(&self, adapter: AdapterId) -> Option<String> {
        self.sessions().get(&adapter).cloned()
    }

    pub fn revoke_session(&self, adapter: AdapterId) {
        self.sessions().remove(&adapter);
    }

    pub fn credential_configured(&self, adapter: AdapterId) -> Result<bool> {
        Ok(self.secrets.read(adapter.secret())?.is_some())
    }

    pub fn configure_credential(&self, adapter: AdapterId, mut api_key: String) -> Result<bool> {
        let normalized = Zeroizing::new(api_key.trim().to_owned());
        api_key.zeroize();
        if normalized.len() < 20
            || normalized.len() > 4096
            || normalized
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control())
        {
            bail!("invalid {} credential format", adapter.label());
        }
        // A replaced credential may belong to a different account or budget.
        self.revoke_session(adapter);
        self.secrets.write(adapter.secret(), &normalized)?;
        self.credential_configured(adapter)
    }

    pub fn remove_credential(&self, adapter: AdapterId) -> Result<bool> {
        self.revoke_session(adapter);
        self.secrets.delete(adapter.secret())?;
        self.credential_configured(adapter)
    }

    /// Enabling requires the full persisted authorization, a stored
    /// credential, and no other cloud Chat backend holding a session.
    pub fn set_session(
        &self,
        store: &Store,
        adapter: AdapterId,
        enabled: bool,
        other_backend_active: bool,
    ) -> Result<bool> {
        if !enabled {
            self.revoke_session(adapter);
            return Ok(false);
        }
        if other_backend_active
            || self
                .active_session()
                .is_some_and(|active| active != adapter)
        {
            bail!("another cloud Chat backend is enabled; turn it off first");
        }
        let (authorization, model) = persisted_authorization(store, adapter)?;
        authorization
            .authorize(adapter.provider(), &model)
            .map_err(|reason| anyhow!("{} authorization denied: {reason:?}", adapter.label()))?;
        if !self.credential_configured(adapter)? {
            bail!("{} credential not configured", adapter.label());
        }
        let (cap, _) = required_limits(store, adapter)?;
        if store.spent_micros(adapter.key(), &spend::current_month())? >= cap {
            bail!("monthly spending limit reached");
        }
        self.sessions().insert(adapter, model);
        Ok(true)
    }

    pub async fn generate(
        &self,
        store: &Store,
        transport: &InferenceTransport,
        adapter: AdapterId,
        prompt: &str,
    ) -> Result<String> {
        let granted = self
            .session_model(adapter)
            .ok_or_else(|| anyhow!("{} Chat is not enabled for this session", adapter.label()))?;
        let (authorization, model) = persisted_authorization(store, adapter)?;
        if granted != model {
            self.revoke_session(adapter);
            bail!(
                "{} model changed since Chat was enabled; re-enable to continue",
                adapter.label()
            );
        }
        authorization
            .authorize(adapter.provider(), &model)
            .map_err(|reason| anyhow!("{} authorization denied: {reason:?}", adapter.label()))?;
        let key = self
            .secrets
            .read(adapter.secret())?
            .ok_or_else(|| anyhow!("{} credential not configured", adapter.label()))?;
        if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
            bail!("invalid inference prompt length");
        }
        let (cap, prices) = required_limits(store, adapter)?;
        let month = spend::current_month();
        let reserved = prices.worst_case(prompt.len(), MAX_OUTPUT_TOKENS);
        store.reserve_spend(adapter.key(), &month, reserved, cap)?;
        let result = transport
            .generate_with_usage(
                &authorization,
                adapter.provider(),
                &model,
                prompt,
                Some(key.as_str()),
            )
            .await;
        // Charge the actual cost; a rejected request is not billed; any other
        // outcome (timeout, unreadable reply, missing usage) stays at worst case.
        let correction = match &result {
            Ok(generation) => match (generation.input_tokens, generation.output_tokens) {
                (Some(input), Some(output)) => prices.cost(input, output) - reserved,
                _ => 0,
            },
            Err(error) if error.downcast_ref::<ProviderRejected>().is_some() => -reserved,
            Err(_) => 0,
        };
        store.adjust_spend(adapter.key(), &month, correction)?;
        result.map(|generation| generation.text)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterSettings {
    adapter: AdapterId,
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
    credential_configured: bool,
    chat_session_enabled: bool,
    monthly_cap_micros: Option<i64>,
    input_price_micros: Option<i64>,
    output_price_micros: Option<i64>,
    spent_this_month_micros: i64,
    suggested_monthly_cap_micros: i64,
}

fn settings_view(
    store: &Store,
    adapters: &Adapters,
    adapter: AdapterId,
) -> Result<AdapterSettings> {
    let (model_id, cloud_enabled, metered_enabled) = store.adapter_settings(adapter.key())?;
    let limits = store.adapter_limits(adapter.key())?;
    Ok(AdapterSettings {
        adapter,
        model_id,
        cloud_enabled,
        metered_enabled,
        credential_configured: adapters.credential_configured(adapter)?,
        chat_session_enabled: adapters.session_model(adapter).is_some(),
        monthly_cap_micros: limits.monthly_cap_micros,
        input_price_micros: limits.input_price_micros,
        output_price_micros: limits.output_price_micros,
        spent_this_month_micros: store.spent_micros(adapter.key(), &spend::current_month())?,
        suggested_monthly_cap_micros: spend::DEFAULT_MONTHLY_CAP_MICROS,
    })
}

fn all_settings(store: &Store, adapters: &Adapters) -> Result<Vec<AdapterSettings>> {
    AdapterId::ALL
        .into_iter()
        .map(|adapter| settings_view(store, adapters, adapter))
        .collect()
}

fn to_message(error: anyhow::Error) -> String {
    error.to_string()
}

#[tauri::command]
pub fn adapter_settings(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    all_settings(&store, &adapters).map_err(to_message)
}

/// Model authorization and its required spending configuration, as entered
/// in Settings. Dollar amounts; prices are per million tokens.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterAuthorizationInput {
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
    monthly_cap_usd: Option<f64>,
    input_price_usd: Option<f64>,
    output_price_usd: Option<f64>,
}

#[tauri::command]
pub fn configure_adapter_model(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
    authorization: AdapterAuthorizationInput,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    // Every model or billing policy change requires a fresh session opt-in.
    adapters.revoke_session(adapter);
    let limits = SpendLimits {
        monthly_cap_micros: authorization
            .monthly_cap_usd
            .and_then(spend::dollars_to_micros),
        input_price_micros: authorization
            .input_price_usd
            .and_then(spend::dollars_to_micros),
        output_price_micros: authorization
            .output_price_usd
            .and_then(spend::dollars_to_micros),
    };
    // A monthly limit and both prices are required configuration for metered use.
    if authorization.metered_enabled
        && (limits.monthly_cap_micros.is_none()
            || limits.input_price_micros.is_none()
            || limits.output_price_micros.is_none())
    {
        return Err(format!(
            "{} monthly spending limit not configured",
            adapter.label()
        ));
    }
    store
        .set_adapter_limits(adapter.key(), limits)
        .map_err(to_message)?;
    store
        .set_adapter_settings(
            adapter.key(),
            authorization.model_id.as_deref(),
            authorization.cloud_enabled,
            authorization.metered_enabled,
        )
        .map_err(to_message)?;
    all_settings(&store, &adapters).map_err(to_message)
}

#[tauri::command]
pub fn set_adapter_chat_session(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    openrouter: State<'_, OpenRouterCredentials>,
    adapter: AdapterId,
    enabled: bool,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    let openrouter_active = openrouter.chat_session_active().map_err(to_message)?;
    adapters
        .set_session(&store, adapter, enabled, openrouter_active)
        .map_err(to_message)?;
    all_settings(&store, &adapters).map_err(to_message)
}

#[tauri::command]
pub fn configure_adapter_credential(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
    api_key: String,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    adapters
        .configure_credential(adapter, api_key)
        .map_err(to_message)?;
    all_settings(&store, &adapters).map_err(to_message)
}

#[tauri::command]
pub fn remove_adapter_credential(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    adapters.remove_credential(adapter).map_err(to_message)?;
    all_settings(&store, &adapters).map_err(to_message)
}

#[tauri::command]
pub async fn generate_with_adapter(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
    prompt: String,
) -> std::result::Result<String, String> {
    let transport = InferenceTransport::new().map_err(to_message)?;
    adapters
        .generate(&store, &transport, adapter, &prompt)
        .await
        .map_err(to_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference_transport::mock::{anthropic_message, completion, serve};

    const KEY: &str = "sk-test-secret-for-adapter-tests";

    #[derive(Default)]
    struct FakeSecretStore {
        values: Mutex<HashMap<&'static str, String>>,
    }

    fn slot(name: SecretName) -> &'static str {
        match name {
            SecretName::OpenAiApiKey => "openai",
            SecretName::AnthropicApiKey => "anthropic",
            SecretName::OpenRouterApiKey => "openrouter",
            SecretName::GeminiApiKey => "gemini",
        }
    }

    impl SecretStore for FakeSecretStore {
        fn read(&self, name: SecretName) -> Result<Option<Zeroizing<String>>> {
            Ok(self
                .values
                .lock()
                .unwrap()
                .get(slot(name))
                .cloned()
                .map(Zeroizing::new))
        }
        fn write(&self, name: SecretName, value: &str) -> Result<()> {
            self.values
                .lock()
                .unwrap()
                .insert(slot(name), value.to_owned());
            Ok(())
        }
        fn delete(&self, name: SecretName) -> Result<()> {
            self.values.lock().unwrap().remove(slot(name));
            Ok(())
        }
    }

    /// $20 a month; $1 per million input tokens, $2 per million output tokens.
    const LIMITS: SpendLimits = SpendLimits {
        monthly_cap_micros: Some(spend::DEFAULT_MONTHLY_CAP_MICROS),
        input_price_micros: Some(1_000_000),
        output_price_micros: Some(2_000_000),
    };

    fn fixture() -> Result<(tempfile::TempDir, Store, Adapters)> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        for adapter in AdapterId::ALL {
            store.set_adapter_limits(adapter.key(), LIMITS)?;
        }
        Ok((
            directory,
            store,
            Adapters::with_secrets(Arc::new(FakeSecretStore::default())),
        ))
    }

    fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
        tauri::async_runtime::block_on(future)
    }

    fn model_for(adapter: AdapterId) -> &'static str {
        match adapter {
            AdapterId::OpenAi => "gpt-test-mini",
            AdapterId::Anthropic => "claude-test-5",
        }
    }

    #[test]
    fn every_adapter_is_off_by_default() -> Result<()> {
        let (_dir, store, adapters) = fixture()?;
        for view in all_settings(&store, &adapters)? {
            assert_eq!(view.model_id, None);
            assert!(!view.cloud_enabled && !view.metered_enabled);
            assert!(!view.credential_configured && !view.chat_session_enabled);
        }
        for adapter in AdapterId::ALL {
            assert!(adapters.set_session(&store, adapter, true, false).is_err());
        }
        assert_eq!(adapters.active_session(), None);
        Ok(())
    }

    #[test]
    fn api_session_requires_full_authorization() -> Result<()> {
        for adapter in AdapterId::ALL {
            let (_dir, store, adapters) = fixture()?;
            let model = model_for(adapter);
            adapters.configure_credential(adapter, KEY.into())?;
            assert!(
                adapters.set_session(&store, adapter, true, false).is_err(),
                "no model"
            );
            store.set_adapter_settings(adapter.key(), Some(model), false, true)?;
            assert!(
                adapters.set_session(&store, adapter, true, false).is_err(),
                "no cloud"
            );
            store.set_adapter_settings(adapter.key(), Some(model), true, false)?;
            assert!(
                adapters.set_session(&store, adapter, true, false).is_err(),
                "no metered"
            );
            store.set_adapter_settings(adapter.key(), Some(model), true, true)?;
            adapters.remove_credential(adapter)?;
            assert!(
                adapters.set_session(&store, adapter, true, false).is_err(),
                "no key"
            );
            adapters.configure_credential(adapter, KEY.into())?;
            assert!(
                adapters.set_session(&store, adapter, true, true).is_err(),
                "OpenRouter already active"
            );
            assert!(adapters.set_session(&store, adapter, true, false)?);
            assert!(!adapters.set_session(&store, adapter, false, false)?);
        }
        Ok(())
    }

    #[test]
    fn only_one_backend_session_at_a_time() -> Result<()> {
        let (_dir, store, adapters) = fixture()?;
        for adapter in AdapterId::ALL {
            adapters.configure_credential(adapter, KEY.into())?;
            store.set_adapter_settings(adapter.key(), Some(model_for(adapter)), true, true)?;
        }
        adapters.set_session(&store, AdapterId::OpenAi, true, false)?;
        assert!(adapters
            .set_session(&store, AdapterId::Anthropic, true, false)
            .is_err());
        assert_eq!(adapters.active_session(), Some(AdapterId::OpenAi));
        Ok(())
    }

    #[test]
    fn permitted_api_requests_reach_the_backend() -> Result<()> {
        for (adapter, body) in [
            (AdapterId::OpenAi, completion("OpenAI reply")),
            (AdapterId::Anthropic, anthropic_message("Anthropic reply")),
        ] {
            let (_dir, store, adapters) = fixture()?;
            let server = serve(200, body);
            let transport = InferenceTransport::for_test(server.endpoint.clone())?;
            adapters.configure_credential(adapter, KEY.into())?;
            store.set_adapter_settings(adapter.key(), Some(model_for(adapter)), true, true)?;
            adapters.set_session(&store, adapter, true, false)?;
            let reply = block_on(adapters.generate(&store, &transport, adapter, "hi"))?;
            assert!(reply.ends_with("reply"));
            assert_eq!(server.requests.lock().unwrap().len(), 1);
        }
        Ok(())
    }

    #[test]
    fn no_api_request_without_session_or_after_revocation() -> Result<()> {
        for adapter in AdapterId::ALL {
            let (_dir, store, adapters) = fixture()?;
            let server = serve(200, completion("unused"));
            let transport = InferenceTransport::for_test(server.endpoint.clone())?;
            let model = model_for(adapter);
            adapters.configure_credential(adapter, KEY.into())?;
            store.set_adapter_settings(adapter.key(), Some(model), true, true)?;
            let send = |adapters: &Adapters| {
                block_on(adapters.generate(&store, &transport, adapter, "hi")).is_err()
            };

            assert!(send(&adapters), "persisted authorization alone never sends");

            adapters.set_session(&store, adapter, true, false)?;
            adapters.configure_credential(adapter, "sk-test-replacement-key-0000".into())?;
            assert!(send(&adapters), "credential replacement revokes");

            adapters.set_session(&store, adapter, true, false)?;
            adapters.remove_credential(adapter)?;
            assert!(send(&adapters), "credential removal revokes");

            adapters.configure_credential(adapter, KEY.into())?;
            adapters.set_session(&store, adapter, true, false)?;
            store.set_adapter_settings(adapter.key(), Some(model), true, false)?;
            assert!(send(&adapters), "metered permission withdrawn behind grant");

            store.set_adapter_settings(adapter.key(), Some(model), true, true)?;
            adapters.set_session(&store, adapter, true, false)?;
            store.set_adapter_settings(adapter.key(), Some("other-model"), true, true)?;
            let error = block_on(adapters.generate(&store, &transport, adapter, "hi")).unwrap_err();
            assert!(error.to_string().contains("model changed"));
            assert_eq!(adapters.session_model(adapter), None);

            std::thread::sleep(std::time::Duration::from_millis(100));
            assert!(server.requests.lock().unwrap().is_empty());
        }
        Ok(())
    }

    #[test]
    fn restart_does_not_restore_a_session() -> Result<()> {
        let (dir, store, adapters) = fixture()?;
        adapters.configure_credential(AdapterId::OpenAi, KEY.into())?;
        store.set_adapter_settings("openai", Some("gpt-test-mini"), true, true)?;
        adapters.set_session(&store, AdapterId::OpenAi, true, false)?;
        drop(store);
        let store = Store::open(dir.path())?;
        let relaunched = Adapters::with_secrets(Arc::clone(&adapters.secrets));
        assert_eq!(relaunched.active_session(), None);
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(
            block_on(relaunched.generate(&store, &transport, AdapterId::OpenAi, "hi")).is_err()
        );
        assert!(server.requests.lock().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn settings_and_errors_never_contain_credentials() -> Result<()> {
        let (dir, store, adapters) = fixture()?;
        adapters.configure_credential(AdapterId::Anthropic, KEY.into())?;
        store.set_adapter_settings("anthropic", Some("claude-test-5"), true, true)?;
        adapters.set_session(&store, AdapterId::Anthropic, true, false)?;
        let json = serde_json::to_string(&all_settings(&store, &adapters)?)?;
        assert!(!json.contains(KEY));
        let server = serve(401, "{\"type\":\"error\"}".into());
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        let error = block_on(adapters.generate(&store, &transport, AdapterId::Anthropic, "hi"))
            .unwrap_err();
        assert!(error.to_string().contains("HTTP 401"));
        assert!(!format!("{error:#} {error:?}").contains(KEY));
        drop(store);
        for entry in std::fs::read_dir(dir.path())? {
            let path = entry?.path();
            if path.is_file() {
                let bytes = std::fs::read(&path)?;
                assert!(!bytes
                    .windows(KEY.len())
                    .any(|window| window == KEY.as_bytes()));
            }
        }
        Ok(())
    }

    fn ready(store: &Store, adapters: &Adapters, adapter: AdapterId) -> Result<()> {
        adapters.configure_credential(adapter, KEY.into())?;
        store.set_adapter_settings(adapter.key(), Some(model_for(adapter)), true, true)?;
        adapters.set_session(store, adapter, true, false)?;
        Ok(())
    }

    #[test]
    fn metered_use_requires_a_monthly_limit_and_prices() -> Result<()> {
        let (_dir, store, adapters) = fixture()?;
        adapters.configure_credential(AdapterId::OpenAi, KEY.into())?;
        store.set_adapter_settings("openai", Some("gpt-test-mini"), true, true)?;
        for missing in [
            SpendLimits {
                monthly_cap_micros: None,
                ..LIMITS
            },
            SpendLimits {
                input_price_micros: None,
                ..LIMITS
            },
            SpendLimits {
                output_price_micros: None,
                ..LIMITS
            },
        ] {
            store.set_adapter_limits("openai", missing)?;
            let error = adapters
                .set_session(&store, AdapterId::OpenAi, true, false)
                .unwrap_err();
            assert!(error.to_string().contains("spending limit not configured"));
        }
        // A grant made earlier still cannot send once the limit is removed.
        store.set_adapter_limits("openai", LIMITS)?;
        adapters.set_session(&store, AdapterId::OpenAi, true, false)?;
        store.set_adapter_limits("openai", SpendLimits::default())?;
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(block_on(adapters.generate(&store, &transport, AdapterId::OpenAi, "hi")).is_err());
        assert!(server.requests.lock().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn each_reply_is_charged_its_actual_cost() -> Result<()> {
        for (adapter, body) in [
            (AdapterId::OpenAi, completion("reply")),
            (AdapterId::Anthropic, anthropic_message("reply")),
        ] {
            let (_dir, store, adapters) = fixture()?;
            ready(&store, &adapters, adapter)?;
            let server = serve(200, body);
            let transport = InferenceTransport::for_test(server.endpoint.clone())?;
            block_on(adapters.generate(&store, &transport, adapter, "hi"))?;
            // Mocks report 3 input and 5 output tokens: ceil((3*1 + 5*2) / 1e6 * 1e6) micros.
            assert_eq!(
                store.spent_micros(adapter.key(), &spend::current_month())?,
                TokenPrices {
                    input: 1_000_000,
                    output: 2_000_000
                }
                .cost(3, 5)
            );
        }
        Ok(())
    }

    #[test]
    fn requests_that_could_exceed_the_limit_are_never_sent() -> Result<()> {
        let (_dir, store, adapters) = fixture()?;
        ready(&store, &adapters, AdapterId::OpenAi)?;
        let month = spend::current_month();
        let worst = TokenPrices {
            input: 1_000_000,
            output: 2_000_000,
        }
        .worst_case(2, MAX_OUTPUT_TOKENS);
        // Leave less headroom than one worst-case request.
        store.adjust_spend(
            "openai",
            &month,
            spend::DEFAULT_MONTHLY_CAP_MICROS - worst + 1,
        )?;
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        let error =
            block_on(adapters.generate(&store, &transport, AdapterId::OpenAi, "hi")).unwrap_err();
        assert!(error.to_string().contains("monthly spending limit reached"));
        assert!(server.requests.lock().unwrap().is_empty());
        // Once the limit is reached, Chat cannot even be re-enabled.
        store.adjust_spend("openai", &month, worst)?;
        adapters.set_session(&store, AdapterId::OpenAi, false, false)?;
        assert!(adapters
            .set_session(&store, AdapterId::OpenAi, true, false)
            .is_err());
        Ok(())
    }

    #[test]
    fn rejected_requests_release_the_reservation_and_unknown_outcomes_keep_it() -> Result<()> {
        let (_dir, store, adapters) = fixture()?;
        ready(&store, &adapters, AdapterId::Anthropic)?;
        let month = spend::current_month();
        let server = serve(429, "{}".into());
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(
            block_on(adapters.generate(&store, &transport, AdapterId::Anthropic, "hi")).is_err()
        );
        assert_eq!(store.spent_micros("anthropic", &month)?, 0);

        let server = serve(200, "not json".into());
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(
            block_on(adapters.generate(&store, &transport, AdapterId::Anthropic, "hi")).is_err()
        );
        let worst = TokenPrices {
            input: 1_000_000,
            output: 2_000_000,
        }
        .worst_case(2, MAX_OUTPUT_TOKENS);
        assert_eq!(store.spent_micros("anthropic", &month)?, worst);
        Ok(())
    }

    #[test]
    fn adapter_ids_serialize_to_stable_keys() {
        for adapter in AdapterId::ALL {
            assert_eq!(
                serde_json::to_string(&adapter).unwrap(),
                format!("\"{}\"", adapter.key())
            );
        }
    }
}

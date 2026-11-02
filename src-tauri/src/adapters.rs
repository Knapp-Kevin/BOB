//! Direct-vendor API adapters (OpenAI, Anthropic) and vendor CLI adapters
//! (Claude Code, Codex), following the OpenRouter pattern.
//!
//! Each adapter is off by default. Generation requires, all at once: one
//! exact approved model, the persisted permissions its billing class needs, a
//! credential in the OS secret store (API adapters) or a signed-in CLI, and a
//! Rust-held session grant that starts empty on every launch, is bound to the
//! model it was granted for, and is revoked by any model, permission, or
//! credential change. At most one cloud Chat backend (including OpenRouter) can
//! hold a session grant at a time, so the backend never changes silently.
//!
//! CLI adapters are classified `unknown` billing (see `inference_policy`), so
//! the policy blocks them before any process starts until the owner records a
//! billing decision.

use crate::cli_runtime::{self, CancelToken, CliKind, CliStatus};
use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use crate::inference_transport::{InferenceTransport, MAX_PROMPT_BYTES};
use crate::openrouter::OpenRouterCredentials;
use crate::secrets::{OsSecretStore, SecretName, SecretStore};
use crate::state::Store;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdapterId {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "claude_cli")]
    ClaudeCli,
    #[serde(rename = "codex_cli")]
    CodexCli,
}

impl AdapterId {
    pub const ALL: [AdapterId; 4] = [
        AdapterId::OpenAi,
        AdapterId::Anthropic,
        AdapterId::ClaudeCli,
        AdapterId::CodexCli,
    ];

    /// Stable key used in SQLite and by the frontend.
    pub fn key(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::ClaudeCli => "claude_cli",
            Self::CodexCli => "codex_cli",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::ClaudeCli => "Claude CLI",
            Self::CodexCli => "Codex CLI",
        }
    }

    fn provider(self) -> InferenceProvider {
        match self {
            Self::OpenAi => InferenceProvider::OpenAi,
            Self::Anthropic => InferenceProvider::Anthropic,
            Self::ClaudeCli => InferenceProvider::ClaudeCli,
            Self::CodexCli => InferenceProvider::CodexCli,
        }
    }

    fn secret(self) -> Option<SecretName> {
        match self {
            Self::OpenAi => Some(SecretName::OpenAiApiKey),
            Self::Anthropic => Some(SecretName::AnthropicApiKey),
            Self::ClaudeCli | Self::CodexCli => None,
        }
    }

    fn cli(self) -> Option<CliKind> {
        match self {
            Self::ClaudeCli => Some(CliKind::Claude),
            Self::CodexCli => Some(CliKind::Codex),
            Self::OpenAi | Self::Anthropic => None,
        }
    }

    fn billing(self) -> &'static str {
        match self.provider() {
            InferenceProvider::ClaudeCli | InferenceProvider::CodexCli => "unknown",
            _ => "metered",
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
        AdapterId::ClaudeCli => &mut authorization.claude_cli_models,
        AdapterId::CodexCli => &mut authorization.codex_cli_models,
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

pub struct Adapters {
    secrets: Arc<dyn SecretStore>,
    sessions: Mutex<HashMap<AdapterId, String>>,
    in_flight: Mutex<Option<(AdapterId, CancelToken)>>,
    cli_override: Option<PathBuf>,
}

impl Adapters {
    pub fn new() -> Self {
        Self::with_secrets(Arc::new(OsSecretStore::default()))
    }

    fn with_secrets(secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            secrets,
            sessions: Mutex::new(HashMap::new()),
            in_flight: Mutex::new(None),
            cli_override: None,
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

    /// Revokes the grant and stops any in-flight generation for this adapter.
    pub fn revoke_session(&self, adapter: AdapterId) {
        self.sessions().remove(&adapter);
        let in_flight = self
            .in_flight
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((running, token)) = in_flight.as_ref() {
            if *running == adapter {
                token.cancel();
            }
        }
    }

    pub fn cancel_generation(&self) -> bool {
        let in_flight = self
            .in_flight
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match in_flight.as_ref() {
            Some((_, token)) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    pub fn credential_configured(&self, adapter: AdapterId) -> Result<bool> {
        match adapter.secret() {
            Some(name) => Ok(self.secrets.read(name)?.is_some()),
            None => Ok(false),
        }
    }

    pub fn configure_credential(&self, adapter: AdapterId, mut api_key: String) -> Result<bool> {
        let normalized = Zeroizing::new(api_key.trim().to_owned());
        api_key.zeroize();
        let Some(name) = adapter.secret() else {
            bail!("{} signs in through its own CLI", adapter.label());
        };
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
        self.secrets.write(name, &normalized)?;
        self.credential_configured(adapter)
    }

    pub fn remove_credential(&self, adapter: AdapterId) -> Result<bool> {
        let Some(name) = adapter.secret() else {
            bail!("{} signs in through its own CLI", adapter.label());
        };
        self.revoke_session(adapter);
        self.secrets.delete(name)?;
        self.credential_configured(adapter)
    }

    /// Enabling requires the full persisted authorization (which blocks
    /// `unknown` billing), a stored credential for API adapters, and no other
    /// cloud Chat backend holding a session.
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
        if adapter.secret().is_some() && !self.credential_configured(adapter)? {
            bail!("{} credential not configured", adapter.label());
        }
        self.sessions().insert(adapter, model);
        Ok(true)
    }

    fn cli_program(&self, kind: CliKind) -> Option<PathBuf> {
        self.cli_override
            .clone()
            .or_else(|| cli_runtime::locate(kind))
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
        if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
            bail!("invalid inference prompt length");
        }

        if let Some(name) = adapter.secret() {
            let key = self
                .secrets
                .read(name)?
                .ok_or_else(|| anyhow!("{} credential not configured", adapter.label()))?;
            return transport
                .generate(
                    &authorization,
                    adapter.provider(),
                    &model,
                    prompt,
                    Some(key.as_str()),
                )
                .await;
        }

        let kind = adapter
            .cli()
            .ok_or_else(|| anyhow!("unsupported adapter"))?;
        let program = self
            .cli_program(kind)
            .ok_or(cli_runtime::CliFailure::NotInstalled)?;
        let token = CancelToken::default();
        {
            let mut in_flight = self
                .in_flight
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if in_flight.is_some() {
                bail!("a CLI generation is already running");
            }
            *in_flight = Some((adapter, token.clone()));
        }
        let prompt = prompt.to_owned();
        let result = tauri::async_runtime::spawn_blocking(move || {
            cli_runtime::generate(
                kind,
                &program,
                &model,
                &prompt,
                cli_runtime::GENERATION_TIMEOUT,
                &token,
            )
        })
        .await;
        *self
            .in_flight
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        Ok(result.map_err(|_| anyhow!("CLI runtime failed"))??)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterSettings {
    adapter: AdapterId,
    billing: &'static str,
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
    credential_configured: bool,
    chat_session_enabled: bool,
}

fn settings_view(
    store: &Store,
    adapters: &Adapters,
    adapter: AdapterId,
) -> Result<AdapterSettings> {
    let (model_id, cloud_enabled, metered_enabled) = store.adapter_settings(adapter.key())?;
    Ok(AdapterSettings {
        adapter,
        billing: adapter.billing(),
        model_id,
        cloud_enabled,
        metered_enabled,
        credential_configured: adapters.credential_configured(adapter)?,
        chat_session_enabled: adapters.session_model(adapter).is_some(),
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

#[tauri::command]
pub fn configure_adapter_model(
    store: State<'_, Store>,
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
) -> std::result::Result<Vec<AdapterSettings>, String> {
    // Every model or billing policy change requires a fresh session opt-in.
    adapters.revoke_session(adapter);
    store
        .set_adapter_settings(
            adapter.key(),
            model_id.as_deref(),
            cloud_enabled,
            metered_enabled,
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

#[tauri::command]
pub fn cancel_adapter_generation(adapters: State<'_, Adapters>) -> bool {
    adapters.cancel_generation()
}

/// Installation, version, and sign-in detection for a CLI adapter. Runs only
/// the CLI's version and auth-status commands; never sends a prompt.
#[tauri::command]
pub async fn adapter_runtime_status(
    adapters: State<'_, Adapters>,
    adapter: AdapterId,
) -> std::result::Result<CliStatus, String> {
    let kind = adapter
        .cli()
        .ok_or_else(|| format!("{} is not a CLI adapter", adapter.label()))?;
    let program = adapters.cli_program(kind);
    tauri::async_runtime::spawn_blocking(move || cli_runtime::detect(kind, program.as_deref()))
        .await
        .map_err(|_| "CLI detection failed".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_runtime::fake;
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

    fn fixture() -> Result<(tempfile::TempDir, Store, Adapters)> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
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
            AdapterId::ClaudeCli => "sonnet",
            AdapterId::CodexCli => "gpt-test",
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
        for adapter in [AdapterId::OpenAi, AdapterId::Anthropic] {
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
        for adapter in [AdapterId::OpenAi, AdapterId::Anthropic] {
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
        for adapter in [AdapterId::OpenAi, AdapterId::Anthropic] {
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
    fn cli_adapters_never_start_a_process_while_billing_is_unknown() -> Result<()> {
        for adapter in [AdapterId::ClaudeCli, AdapterId::CodexCli] {
            let (_dir, store, mut adapters) = fixture()?;
            let cli = fake::install("{}", 0, 0);
            adapters.cli_override = Some(cli.program.clone());
            store.set_adapter_settings(adapter.key(), Some(model_for(adapter)), true, true)?;
            let error = adapters
                .set_session(&store, adapter, true, false)
                .unwrap_err();
            assert!(error.to_string().contains("BillingUnknown"), "{error}");
            // Even a grant injected behind the gate is re-checked per request.
            adapters
                .sessions()
                .insert(adapter, model_for(adapter).to_owned());
            let transport = InferenceTransport::new()?;
            let error = block_on(adapters.generate(&store, &transport, adapter, "hi")).unwrap_err();
            assert!(error.to_string().contains("BillingUnknown"), "{error}");
            assert!(!cli.invoked(), "CLI must not run");
        }
        Ok(())
    }

    #[test]
    fn cli_adapters_reject_api_keys() {
        let adapters = Adapters::with_secrets(Arc::new(FakeSecretStore::default()));
        assert!(adapters
            .configure_credential(AdapterId::ClaudeCli, KEY.into())
            .is_err());
        assert!(!adapters.credential_configured(AdapterId::CodexCli).unwrap());
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

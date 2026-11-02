//! OpenRouter credentials remain in the operating-system secret store.
//! No API exposes the credential value to the frontend or writes it to SQLite.
//!
//! Persisted authorization (approved model, cloud and metered permission) is
//! necessary but not sufficient: Chat generation also requires a session grant
//! held only in memory. It starts empty on every launch, is bound to the exact
//! model that was authorized when granted, and is revoked by any change to the
//! model authorization or credential.
//!
//! B.O.B.-side spending safeguards sit on top of the vendor console:
//! free-only mode is the default (only `:free` models, every price capped at
//! zero, and any reply reporting a charge revokes the session). Paid use needs
//! a B.O.B. monthly spending limit, charged with OpenRouter's reported cost of
//! each reply, plus a key check before enabling and before every request that
//! refuses keys without a credit limit or with no credit left.

use crate::inference_policy::{InferenceAuthorization, InferenceProvider, ModelBilling};
use crate::inference_transport::{FreeOnlyCharged, InferenceTransport, OpenRouterKeyStatus};
use crate::secrets::{OsSecretStore, SecretName, SecretStore};
use crate::spend;
use crate::state::Store;
use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::State;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterCredentialStatus {
    configured: bool,
}

pub struct OpenRouterCredentials {
    secrets: Arc<dyn SecretStore>,
    session: Mutex<Option<String>>,
}

impl OpenRouterCredentials {
    pub fn new() -> Self {
        Self::with_secrets(Arc::new(OsSecretStore::default()))
    }

    fn with_secrets(secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            secrets,
            session: Mutex::new(None),
        }
    }

    pub fn status(&self) -> Result<OpenRouterCredentialStatus> {
        Ok(OpenRouterCredentialStatus {
            configured: self.secrets.read(SecretName::OpenRouterApiKey)?.is_some(),
        })
    }

    pub fn configure(&self, mut api_key: String) -> Result<OpenRouterCredentialStatus> {
        let normalized = Zeroizing::new(api_key.trim().to_owned());
        api_key.zeroize();
        if normalized.len() < 20
            || normalized.len() > 4096
            || normalized
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control())
        {
            bail!("invalid OpenRouter credential format");
        }
        // A replaced credential may belong to a different account or budget.
        self.revoke_session();
        self.secrets
            .write(SecretName::OpenRouterApiKey, &normalized)?;
        self.status()
    }

    pub fn remove(&self) -> Result<OpenRouterCredentialStatus> {
        self.revoke_session();
        self.secrets.delete(SecretName::OpenRouterApiKey)?;
        self.status()
    }

    fn revoke_session(&self) {
        // A poisoned lock still holds a valid Option; always clear it.
        match self.session.lock() {
            Ok(mut grant) => *grant = None,
            Err(poisoned) => *poisoned.into_inner() = None,
        }
    }

    pub(crate) fn chat_session_active(&self) -> Result<bool> {
        Ok(self.session_model()?.is_some())
    }

    fn session_model(&self) -> Result<Option<String>> {
        self.session
            .lock()
            .map(|grant| grant.clone())
            .map_err(|_| anyhow!("OpenRouter session lock is poisoned"))
    }

    /// Metered use is refused unless the key itself carries a credit limit with
    /// credit left, so a leaked or forgotten key cannot spend without bound.
    async fn require_capped_key(&self, transport: &InferenceTransport, key: &str) -> Result<()> {
        let status = transport.openrouter_key_status(key).await?;
        if !status.allows_metered() {
            bail!("OpenRouter key has no credit limit or no credit left (uncapped key refused)");
        }
        Ok(())
    }

    pub async fn key_status(&self, transport: &InferenceTransport) -> Result<OpenRouterKeyStatus> {
        let key = self
            .secrets
            .read(SecretName::OpenRouterApiKey)?
            .ok_or_else(|| anyhow!("OpenRouter credential not configured"))?;
        transport.openrouter_key_status(key.as_str()).await
    }

    /// Enabling requires the full persisted authorization plus a stored
    /// credential, and for metered use a capped key.
    pub async fn set_session(
        &self,
        store: &Store,
        transport: &InferenceTransport,
        enabled: bool,
    ) -> Result<bool> {
        // Every enable attempt replaces any prior session authority. A denied
        // re-authorization must never leave an older grant visibly active.
        self.revoke_session();
        if !enabled {
            return Ok(false);
        }
        let (authorization, model) = persisted_authorization(store)?;
        let billing = authorization
            .authorize(InferenceProvider::OpenRouter, &model)
            .map_err(|reason| anyhow!("OpenRouter authorization denied: {reason:?}"))?;
        let key = self
            .secrets
            .read(SecretName::OpenRouterApiKey)?
            .ok_or_else(|| anyhow!("OpenRouter credential not configured"))?;
        if billing == ModelBilling::Metered {
            require_spending_headroom(store)?;
            self.require_capped_key(transport, key.as_str()).await?;
        }
        *self
            .session
            .lock()
            .map_err(|_| anyhow!("OpenRouter session lock is poisoned"))? = Some(model);
        Ok(true)
    }

    pub async fn generate(
        &self,
        store: &Store,
        transport: &InferenceTransport,
        prompt: &str,
    ) -> Result<String> {
        let granted = self
            .session_model()?
            .ok_or_else(|| anyhow!("OpenRouter Chat is not enabled for this session"))?;
        let (authorization, model) = persisted_authorization(store)?;
        if granted != model {
            self.revoke_session();
            bail!("OpenRouter model changed since Chat was enabled; re-enable to continue");
        }
        let billing = authorization
            .authorize(InferenceProvider::OpenRouter, &model)
            .map_err(|reason| anyhow!("OpenRouter authorization denied: {reason:?}"))?;
        let key = self
            .secrets
            .read(SecretName::OpenRouterApiKey)?
            .ok_or_else(|| anyhow!("OpenRouter credential not configured"))?;
        let metered = billing == ModelBilling::Metered;
        if metered {
            // The limit may be used up, or the key cap removed on openrouter.ai,
            // since Chat was enabled.
            let checked = match require_spending_headroom(store) {
                Ok(()) => self.require_capped_key(transport, key.as_str()).await,
                Err(error) => Err(error),
            };
            if let Err(error) = checked {
                self.revoke_session();
                return Err(error);
            }
        }
        let result = transport
            .generate_with_usage(
                &authorization,
                InferenceProvider::OpenRouter,
                &model,
                prompt,
                Some(key.as_str()),
            )
            .await;
        if result
            .as_ref()
            .is_err_and(|error| error.downcast_ref::<FreeOnlyCharged>().is_some())
        {
            self.revoke_session();
        }
        let generation = result?;
        if metered {
            // Paid replies are charged OpenRouter's reported cost. Without one,
            // spend cannot be tracked, so the reply is dropped and Chat stops.
            let Some(cost) = generation.cost.and_then(spend::reported_cost_to_micros) else {
                self.revoke_session();
                bail!("OpenRouter did not report the cost of a paid reply; Chat turned off");
            };
            store.adjust_spend("openrouter", &spend::current_month(), cost)?;
        }
        Ok(generation.text)
    }
}

/// Paid use needs a configured monthly limit with spend still below it.
fn require_spending_headroom(store: &Store) -> Result<()> {
    let cap = store
        .openrouter_monthly_cap()?
        .ok_or_else(|| anyhow!("OpenRouter monthly spending limit not configured"))?;
    if store.spent_micros("openrouter", &spend::current_month())? >= cap {
        bail!("monthly spending limit reached");
    }
    Ok(())
}

fn persisted_authorization(store: &Store) -> Result<(InferenceAuthorization, String)> {
    let (model, cloud_enabled, metered_enabled) = store.openrouter_settings()?;
    let model = model.ok_or_else(|| anyhow!("OpenRouter model has not been authorized"))?;
    let mut authorization = InferenceAuthorization {
        cloud_enabled,
        metered_enabled,
        openrouter_free_only: store.openrouter_free_only()?,
        ..Default::default()
    };
    authorization.openrouter_models.insert(model.clone());
    Ok((authorization, model))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterSettings {
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
    free_only: bool,
    chat_session_enabled: bool,
    monthly_cap_micros: Option<i64>,
    spent_this_month_micros: i64,
    suggested_monthly_cap_micros: i64,
}

fn settings_view(
    store: &Store,
    credentials: &OpenRouterCredentials,
) -> std::result::Result<OpenRouterSettings, String> {
    let (model_id, cloud_enabled, metered_enabled) = store
        .openrouter_settings()
        .map_err(|error| error.to_string())?;
    let chat_session_enabled = credentials
        .session_model()
        .map_err(|error| error.to_string())?
        .is_some();
    let free_only = store
        .openrouter_free_only()
        .map_err(|error| error.to_string())?;
    let monthly_cap_micros = store
        .openrouter_monthly_cap()
        .map_err(|error| error.to_string())?;
    let spent_this_month_micros = store
        .spent_micros("openrouter", &spend::current_month())
        .map_err(|error| error.to_string())?;
    Ok(OpenRouterSettings {
        model_id,
        cloud_enabled,
        metered_enabled,
        free_only,
        chat_session_enabled,
        monthly_cap_micros,
        spent_this_month_micros,
        suggested_monthly_cap_micros: spend::DEFAULT_MONTHLY_CAP_MICROS,
    })
}

#[tauri::command]
pub fn openrouter_settings(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
) -> std::result::Result<OpenRouterSettings, String> {
    settings_view(&store, &credentials)
}

#[tauri::command]
pub fn configure_openrouter_model(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
    free_only: bool,
    monthly_cap_usd: Option<f64>,
) -> std::result::Result<OpenRouterSettings, String> {
    // Every model or billing policy change requires a fresh session opt-in.
    credentials.revoke_session();
    let cap = monthly_cap_usd.and_then(spend::dollars_to_micros);
    // Paid use requires a monthly spending limit as part of its configuration.
    if metered_enabled && !free_only && cap.is_none() {
        return Err("OpenRouter monthly spending limit not configured".into());
    }
    if cap.is_some() {
        store
            .set_openrouter_monthly_cap(cap)
            .map_err(|error| error.to_string())?;
    }
    // Switch to free-only first and leave it last, so a partial failure can
    // only ever leave the stricter mode in place.
    if free_only {
        store
            .set_openrouter_free_only(true)
            .map_err(|error| error.to_string())?;
    }
    store
        .set_openrouter_settings(model_id.as_deref(), cloud_enabled, metered_enabled)
        .map_err(|error| error.to_string())?;
    if !free_only {
        store
            .set_openrouter_free_only(false)
            .map_err(|error| error.to_string())?;
    }
    settings_view(&store, &credentials)
}

#[tauri::command]
pub async fn set_openrouter_chat_session(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
    adapters: State<'_, crate::adapters::Adapters>,
    enabled: bool,
) -> std::result::Result<OpenRouterSettings, String> {
    // Only one cloud Chat backend may hold a session grant at a time.
    if enabled && adapters.active_session().is_some() {
        return Err("another cloud Chat backend is enabled; turn it off first".into());
    }
    let transport = InferenceTransport::new().map_err(|error| error.to_string())?;
    credentials
        .set_session(&store, &transport, enabled)
        .await
        .map_err(|error| error.to_string())?;
    settings_view(&store, &credentials)
}

/// Credit limit, remaining credit, and free requests left today for the saved
/// key. Sends no prompt and never returns the key or account identity.
#[tauri::command]
pub async fn openrouter_key_status(
    credentials: State<'_, OpenRouterCredentials>,
) -> std::result::Result<OpenRouterKeyStatus, String> {
    let transport = InferenceTransport::new().map_err(|error| error.to_string())?;
    credentials
        .key_status(&transport)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn generate_openrouter(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
    prompt: String,
) -> std::result::Result<String, String> {
    let transport = InferenceTransport::new().map_err(|error| error.to_string())?;
    credentials
        .generate(&store, &transport, &prompt)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn openrouter_credential_status(
    credentials: State<'_, OpenRouterCredentials>,
) -> std::result::Result<OpenRouterCredentialStatus, String> {
    credentials.status().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn configure_openrouter_credential(
    credentials: State<'_, OpenRouterCredentials>,
    api_key: String,
) -> std::result::Result<OpenRouterCredentialStatus, String> {
    credentials
        .configure(api_key)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_openrouter_credential(
    credentials: State<'_, OpenRouterCredentials>,
) -> std::result::Result<OpenRouterCredentialStatus, String> {
    credentials.remove().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference_transport::mock::{
        completion, completion_costing, key_status, serve_routes,
    };
    use crate::state::ChatEntry;
    use anyhow::Result;

    const KEY: &str = "sk-or-v1-test-secret-for-tests";
    const MODEL: &str = "vendor/approved-model";

    #[derive(Default)]
    struct FakeSecretStore {
        value: Mutex<Option<String>>,
        fail_writes: bool,
    }

    impl SecretStore for FakeSecretStore {
        fn read(&self, _name: SecretName) -> Result<Option<Zeroizing<String>>> {
            Ok(self.value.lock().unwrap().clone().map(Zeroizing::new))
        }
        fn write(&self, _name: SecretName, value: &str) -> Result<()> {
            if self.fail_writes {
                bail!("simulated OS credential store failure");
            }
            *self.value.lock().unwrap() = Some(value.to_owned());
            Ok(())
        }
        fn delete(&self, _name: SecretName) -> Result<()> {
            *self.value.lock().unwrap() = None;
            Ok(())
        }
    }

    /// A profile set up for paid OpenRouter use with a $20 monthly limit.
    fn fixture() -> Result<(tempfile::TempDir, Store, OpenRouterCredentials)> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        store.set_openrouter_free_only(false)?;
        store.set_openrouter_monthly_cap(Some(spend::DEFAULT_MONTHLY_CAP_MICROS))?;
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore::default()));
        Ok((directory, store, credentials))
    }

    fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
        tauri::async_runtime::block_on(future)
    }

    /// Mock OpenRouter: a capped key with credit left, and `chat` for completions.
    fn serve(status: u16, chat: String) -> crate::inference_transport::mock::MockServer {
        serve_routes(vec![
            ("/key", 200, key_status(Some(5.0), Some(4.0))),
            ("/chat/completions", status, chat),
        ])
    }

    fn chat_requests(server: &crate::inference_transport::mock::MockServer) -> usize {
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|raw| raw.starts_with("POST "))
            .count()
    }

    fn enable(credentials: &OpenRouterCredentials, store: &Store) -> Result<bool> {
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        block_on(credentials.set_session(store, &transport, true))
    }

    fn disable(credentials: &OpenRouterCredentials, store: &Store) -> Result<bool> {
        block_on(credentials.set_session(store, &InferenceTransport::new()?, false))
    }

    #[test]
    fn credential_lifecycle_never_returns_secret() -> Result<()> {
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore::default()));
        assert!(!credentials.status()?.configured);
        assert!(credentials.configure("short".to_owned()).is_err());
        assert!(!credentials.status()?.configured);
        let status = credentials.configure(KEY.to_owned())?;
        assert!(status.configured);
        let json = serde_json::to_string(&status)?;
        assert!(!json.contains("sk-or"));
        assert!(credentials.configure("bad key with spaces".into()).is_err());
        assert!(credentials.status()?.configured);
        assert!(!credentials.remove()?.configured);
        Ok(())
    }

    #[test]
    fn failed_credential_write_reports_error_without_secret() {
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore {
            fail_writes: true,
            ..Default::default()
        }));
        let error = credentials.configure(KEY.to_owned()).unwrap_err();
        assert!(!format!("{error:#}").contains(KEY));
        assert!(!credentials.status().unwrap().configured);
    }

    #[test]
    fn disabled_by_default_and_session_requires_full_authorization() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        assert_eq!(store.openrouter_settings()?, (None, false, false));
        assert!(enable(&credentials, &store).is_err());

        credentials.configure(KEY.into())?;
        assert!(enable(&credentials, &store).is_err(), "no model");
        store.set_openrouter_settings(Some(MODEL), false, true)?;
        assert!(enable(&credentials, &store).is_err(), "no cloud");
        store.set_openrouter_settings(Some(MODEL), true, false)?;
        assert!(enable(&credentials, &store).is_err(), "no metered");
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        credentials.remove()?;
        assert!(enable(&credentials, &store).is_err(), "no key");
        credentials.configure(KEY.into())?;
        assert!(enable(&credentials, &store)?);
        assert!(!disable(&credentials, &store)?);
        assert_eq!(credentials.session_model()?, None);
        Ok(())
    }

    #[test]
    fn permitted_request_reaches_backend_and_reply_is_durably_saved() -> Result<()> {
        let (dir, store, credentials) = fixture()?;
        let server = serve(200, completion("Pick the smallest next step."));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;

        let reply = block_on(credentials.generate(&store, &transport, "Where do I start?"))?;
        assert_eq!(reply, "Pick the smallest next step.");
        assert_eq!(chat_requests(&server), 1);
        store.append_chat(ChatEntry {
            author: "bob".into(),
            text: reply.clone(),
        })?;
        drop(store);
        let reopened = Store::open(dir.path())?;
        assert_eq!(reopened.load_chat()?.last().unwrap().text, reply);
        Ok(())
    }

    #[test]
    fn no_cloud_request_without_session_or_after_revocation() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;

        // Persisted authorization alone never sends.
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Credential replacement revokes the session grant.
        enable(&credentials, &store)?;
        credentials.configure("sk-or-v1-replacement-key-0000".into())?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Credential removal revokes the session grant.
        enable(&credentials, &store)?;
        credentials.remove()?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Persisted permission withdrawn behind an active grant (e.g. restore).
        credentials.configure(KEY.into())?;
        enable(&credentials, &store)?;
        store.set_openrouter_settings(Some(MODEL), true, false)?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Model changed behind an active grant: no silent switch, grant revoked.
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        store.set_openrouter_settings(Some("vendor/other-model"), true, true)?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        assert!(error.to_string().contains("model changed"));
        assert_eq!(credentials.session_model()?, None);

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(server.requests.lock().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn unsuccessful_session_reenable_clears_previous_grant() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        assert!(enable(&credentials, &store)?);
        assert_eq!(credentials.session_model()?, Some(MODEL.to_owned()));

        // Simulate a restored or externally revised policy without using the
        // UI command (which separately revokes on normal settings changes).
        store.set_openrouter_settings(Some(MODEL), false, true)?;
        assert!(enable(&credentials, &store).is_err());
        assert_eq!(credentials.session_model()?, None);
        assert!(!settings_view(&store, &credentials)?.chat_session_enabled);
        Ok(())
    }

    #[test]
    fn metered_session_refuses_uncapped_or_exhausted_keys() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        for (limit, remaining) in [(None, None), (Some(5.0), Some(0.0))] {
            let server = serve_routes(vec![("/key", 200, key_status(limit, remaining))]);
            let transport = InferenceTransport::for_test(server.endpoint.clone())?;
            let error = block_on(credentials.set_session(&store, &transport, true)).unwrap_err();
            assert!(
                error.to_string().contains("uncapped key refused"),
                "{error}"
            );
            assert_eq!(credentials.session_model()?, None);
        }
        // An unreachable key check fails closed.
        let server = serve_routes(vec![("/key", 500, "{}".into())]);
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(block_on(credentials.set_session(&store, &transport, true)).is_err());
        Ok(())
    }

    #[test]
    fn removing_the_key_cap_mid_session_revokes_before_sending() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        let server = serve_routes(vec![
            ("/key", 200, key_status(None, None)),
            ("/chat/completions", 200, completion("unused")),
        ]);
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());
        assert_eq!(credentials.session_model()?, None);
        assert_eq!(chat_requests(&server), 0);
        Ok(())
    }

    #[test]
    fn free_only_mode_needs_no_metered_permission_or_key_cap() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_free_only(true)?;
        store.set_openrouter_settings(Some(MODEL), true, false)?;
        assert!(enable(&credentials, &store).is_err(), "paid model refused");
        store.set_openrouter_settings(Some("vendor/approved-model:free"), true, false)?;
        // Uncapped key is fine here: the price cap, not the key, bounds spend.
        let server = serve_routes(vec![
            ("/key", 200, key_status(None, None)),
            (
                "/chat/completions",
                200,
                completion_costing("Free reply.", 0.0),
            ),
        ]);
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(block_on(credentials.set_session(&store, &transport, true))?);
        let reply = block_on(credentials.generate(&store, &transport, "hi"))?;
        assert_eq!(reply, "Free reply.");
        assert!(
            server
                .requests
                .lock()
                .unwrap()
                .iter()
                .all(|raw| !raw.starts_with("GET /api/v1/key")),
            "free-only mode does not depend on the key check"
        );
        Ok(())
    }

    #[test]
    fn free_only_reply_that_reports_a_charge_revokes_the_session() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_free_only(true)?;
        store.set_openrouter_settings(Some("vendor/approved-model:free"), true, false)?;
        enable(&credentials, &store)?;
        let server = serve(200, completion_costing("billed", 0.01));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        assert!(error.downcast_ref::<FreeOnlyCharged>().is_some());
        assert_eq!(credentials.session_model()?, None);
        assert!(
            !settings_view(&store, &credentials)
                .unwrap()
                .chat_session_enabled
        );
        Ok(())
    }

    #[test]
    fn new_profiles_are_free_only_and_paid_use_needs_a_monthly_limit() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore::default()));
        credentials.configure(KEY.into())?;
        assert!(store.openrouter_free_only()?);
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        assert!(
            enable(&credentials, &store).is_err(),
            "free-only default refuses paid model"
        );
        store.set_openrouter_free_only(false)?;
        let error = enable(&credentials, &store).unwrap_err();
        assert!(error.to_string().contains("spending limit not configured"));
        store.set_openrouter_monthly_cap(Some(spend::DEFAULT_MONTHLY_CAP_MICROS))?;
        assert!(enable(&credentials, &store)?);
        Ok(())
    }

    #[test]
    fn paid_replies_are_charged_and_the_limit_stops_chat() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        let month = spend::current_month();
        let server = serve(200, completion_costing("paid", 0.0125));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        block_on(credentials.generate(&store, &transport, "hi"))?;
        assert_eq!(store.spent_micros("openrouter", &month)?, 12_500);

        store.adjust_spend("openrouter", &month, spend::DEFAULT_MONTHLY_CAP_MICROS)?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        assert!(error.to_string().contains("monthly spending limit reached"));
        assert_eq!(credentials.session_model()?, None, "Chat turned off");
        assert_eq!(chat_requests(&server), 1, "no request past the limit");
        assert!(enable(&credentials, &store).is_err());
        Ok(())
    }

    #[test]
    fn paid_reply_without_a_reported_cost_turns_chat_off() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        let no_cost = serde_json::json!({
            "choices": [{ "message": { "content": "x" }, "finish_reason": "stop" }]
        })
        .to_string();
        let server = serve(200, no_cost);
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        assert!(error.to_string().contains("did not report the cost"));
        assert_eq!(credentials.session_model()?, None);
        Ok(())
    }

    #[test]
    fn restart_does_not_restore_cloud_chat() -> Result<()> {
        let (dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        drop(store);

        // Relaunch: persisted authorization survives, the session grant does not.
        let store = Store::open(dir.path())?;
        let relaunched = OpenRouterCredentials::with_secrets(Arc::clone(&credentials.secrets));
        assert_eq!(
            store.openrouter_settings()?,
            (Some(MODEL.into()), true, true)
        );
        assert!(
            !settings_view(&store, &relaunched)
                .unwrap()
                .chat_session_enabled
        );
        let server = serve(200, completion("unused"));
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        assert!(block_on(relaunched.generate(&store, &transport, "hi")).is_err());
        assert!(server.requests.lock().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn provider_failure_error_is_clear_and_secret_free() -> Result<()> {
        let (_dir, store, credentials) = fixture()?;
        let server = serve(
            402,
            "{\"error\":{\"message\":\"insufficient credits\"}}".into(),
        );
        let transport = InferenceTransport::for_test(server.endpoint.clone())?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("HTTP 402"));
        assert!(!format!("{error:#} {error:?}").contains(KEY));
        Ok(())
    }

    #[test]
    fn exports_and_settings_never_contain_credential() -> Result<()> {
        let (dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        enable(&credentials, &store)?;
        let export =
            crate::export::export_json(store.load()?, store.load_accessibility_preferences()?)?;
        assert!(!export.contains(KEY));
        let settings = serde_json::to_string(&settings_view(&store, &credentials).unwrap())?;
        assert!(!settings.contains(KEY));
        let debug = format!("{:?}", settings_view(&store, &credentials).unwrap());
        assert!(!debug.contains(KEY));
        drop(store);
        for entry in std::fs::read_dir(dir.path())? {
            let path = entry?.path();
            if path.is_file() {
                let bytes = std::fs::read(&path)?;
                assert!(
                    !bytes
                        .windows(KEY.len())
                        .any(|window| window == KEY.as_bytes()),
                    "credential found in {}",
                    path.display()
                );
            }
        }
        Ok(())
    }
}

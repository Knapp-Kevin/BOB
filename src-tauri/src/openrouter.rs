//! OpenRouter credentials remain in the operating-system secret store.
//! No API exposes the credential value to the frontend or writes it to SQLite.
//!
//! Persisted authorization (approved model, cloud and metered permission) is
//! necessary but not sufficient: Chat generation also requires a session grant
//! held only in memory. It starts empty on every launch, is bound to the exact
//! model that was authorized when granted, and is revoked by any change to the
//! model authorization or credential.

use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use crate::inference_transport::InferenceTransport;
use crate::secrets::{OsSecretStore, SecretName, SecretStore};
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

    fn session_model(&self) -> Result<Option<String>> {
        self.session
            .lock()
            .map(|grant| grant.clone())
            .map_err(|_| anyhow!("OpenRouter session lock is poisoned"))
    }

    /// Enabling requires the full persisted authorization plus a stored credential.
    pub fn set_session(&self, store: &Store, enabled: bool) -> Result<bool> {
        if !enabled {
            self.revoke_session();
            return Ok(false);
        }
        let (authorization, model) = persisted_authorization(store)?;
        authorization
            .authorize(InferenceProvider::OpenRouter, &model)
            .map_err(|reason| anyhow!("OpenRouter authorization denied: {reason:?}"))?;
        if !self.status()?.configured {
            bail!("OpenRouter credential not configured");
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
        authorization
            .authorize(InferenceProvider::OpenRouter, &model)
            .map_err(|reason| anyhow!("OpenRouter authorization denied: {reason:?}"))?;
        let key = self
            .secrets
            .read(SecretName::OpenRouterApiKey)?
            .ok_or_else(|| anyhow!("OpenRouter credential not configured"))?;
        transport
            .generate(
                &authorization,
                InferenceProvider::OpenRouter,
                &model,
                prompt,
                Some(key.as_str()),
            )
            .await
    }
}

fn persisted_authorization(store: &Store) -> Result<(InferenceAuthorization, String)> {
    let (model, cloud_enabled, metered_enabled) = store.openrouter_settings()?;
    let model = model.ok_or_else(|| anyhow!("OpenRouter model has not been authorized"))?;
    let mut authorization = InferenceAuthorization {
        cloud_enabled,
        metered_enabled,
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
    chat_session_enabled: bool,
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
    Ok(OpenRouterSettings {
        model_id,
        cloud_enabled,
        metered_enabled,
        chat_session_enabled,
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
) -> std::result::Result<OpenRouterSettings, String> {
    // Every model or billing policy change requires a fresh session opt-in.
    credentials.revoke_session();
    store
        .set_openrouter_settings(model_id.as_deref(), cloud_enabled, metered_enabled)
        .map_err(|error| error.to_string())?;
    settings_view(&store, &credentials)
}

#[tauri::command]
pub fn set_openrouter_chat_session(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
    enabled: bool,
) -> std::result::Result<OpenRouterSettings, String> {
    credentials
        .set_session(&store, enabled)
        .map_err(|error| error.to_string())?;
    settings_view(&store, &credentials)
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
    use crate::inference_transport::mock::{completion, serve};
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

    fn fixture() -> Result<(tempfile::TempDir, Store, OpenRouterCredentials)> {
        let directory = tempfile::tempdir()?;
        let store = Store::open(directory.path())?;
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore::default()));
        Ok((directory, store, credentials))
    }

    fn block_on<T>(future: impl std::future::Future<Output = T>) -> T {
        tauri::async_runtime::block_on(future)
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
        assert!(credentials.set_session(&store, true).is_err());

        credentials.configure(KEY.into())?;
        assert!(credentials.set_session(&store, true).is_err(), "no model");
        store.set_openrouter_settings(Some(MODEL), false, true)?;
        assert!(credentials.set_session(&store, true).is_err(), "no cloud");
        store.set_openrouter_settings(Some(MODEL), true, false)?;
        assert!(credentials.set_session(&store, true).is_err(), "no metered");
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        credentials.remove()?;
        assert!(credentials.set_session(&store, true).is_err(), "no key");
        credentials.configure(KEY.into())?;
        assert!(credentials.set_session(&store, true)?);
        assert!(!credentials.set_session(&store, false)?);
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
        credentials.set_session(&store, true)?;

        let reply = block_on(credentials.generate(&store, &transport, "Where do I start?"))?;
        assert_eq!(reply, "Pick the smallest next step.");
        assert_eq!(server.requests.lock().unwrap().len(), 1);
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
        credentials.set_session(&store, true)?;
        credentials.configure("sk-or-v1-replacement-key-0000".into())?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Credential removal revokes the session grant.
        credentials.set_session(&store, true)?;
        credentials.remove()?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Persisted permission withdrawn behind an active grant (e.g. restore).
        credentials.configure(KEY.into())?;
        credentials.set_session(&store, true)?;
        store.set_openrouter_settings(Some(MODEL), true, false)?;
        assert!(block_on(credentials.generate(&store, &transport, "hi")).is_err());

        // Model changed behind an active grant: no silent switch, grant revoked.
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        credentials.set_session(&store, true)?;
        store.set_openrouter_settings(Some("vendor/other-model"), true, true)?;
        let error = block_on(credentials.generate(&store, &transport, "hi")).unwrap_err();
        assert!(error.to_string().contains("model changed"));
        assert_eq!(credentials.session_model()?, None);

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(server.requests.lock().unwrap().is_empty());
        Ok(())
    }

    #[test]
    fn restart_does_not_restore_cloud_chat() -> Result<()> {
        let (dir, store, credentials) = fixture()?;
        credentials.configure(KEY.into())?;
        store.set_openrouter_settings(Some(MODEL), true, true)?;
        credentials.set_session(&store, true)?;
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
        credentials.set_session(&store, true)?;
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
        credentials.set_session(&store, true)?;
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

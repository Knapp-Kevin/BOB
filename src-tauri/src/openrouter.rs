//! OpenRouter credentials remain in the operating-system secret store.
//! No API exposes the credential value to the frontend or writes it to SQLite.

use crate::secrets::{OsSecretStore, SecretName, SecretStore};
use crate::state::Store;
use crate::inference_policy::{InferenceAuthorization, InferenceProvider};
use crate::inference_transport::InferenceTransport;
use anyhow::{bail, Result};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterCredentialStatus {
    configured: bool,
}

pub struct OpenRouterCredentials {
    secrets: Arc<dyn SecretStore>,
}

impl OpenRouterCredentials {
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(OsSecretStore::default()),
        }
    }

    #[cfg(test)]
    fn with_secrets(secrets: Arc<dyn SecretStore>) -> Self {
        Self { secrets }
    }

    pub fn status(&self) -> Result<OpenRouterCredentialStatus> {
        Ok(OpenRouterCredentialStatus {
            configured: self.secrets.read(SecretName::OpenRouterApiKey)?.is_some(),
        })
    }

    pub fn configure(&self, mut api_key: String) -> Result<OpenRouterCredentialStatus> {
        let normalized = api_key.trim().to_owned();
        api_key.zeroize();
        if normalized.len() < 20
            || normalized.len() > 4096
            || normalized.chars().any(|ch| ch.is_whitespace() || ch.is_control())
        {
            bail!("invalid OpenRouter credential format");
        }
        // Only replace an existing credential after the new input has passed validation.
        let result = self.secrets.write(SecretName::OpenRouterApiKey, &normalized);
        let mut value = normalized;
        value.zeroize();
        result?;
        self.status()
    }

    pub fn remove(&self) -> Result<OpenRouterCredentialStatus> {
        self.secrets.delete(SecretName::OpenRouterApiKey)?;
        self.status()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterSettings {
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
}

#[tauri::command]
pub fn openrouter_settings(store: State<'_, Store>) -> std::result::Result<OpenRouterSettings, String> {
    let (model_id, cloud_enabled, metered_enabled) =
        store.openrouter_settings().map_err(|error| error.to_string())?;
    Ok(OpenRouterSettings { model_id, cloud_enabled, metered_enabled })
}

#[tauri::command]
pub fn configure_openrouter_model(
    store: State<'_, Store>,
    model_id: Option<String>,
    cloud_enabled: bool,
    metered_enabled: bool,
) -> std::result::Result<OpenRouterSettings, String> {
    store.set_openrouter_settings(model_id.as_deref(), cloud_enabled, metered_enabled)
        .map_err(|error| error.to_string())?;
    openrouter_settings(store)
}

#[tauri::command]
pub async fn generate_openrouter(
    store: State<'_, Store>,
    credentials: State<'_, OpenRouterCredentials>,
    prompt: String,
) -> std::result::Result<String, String> {
    let (model, cloud_enabled, metered_enabled) =
        store.openrouter_settings().map_err(|error| error.to_string())?;
    let model = model.ok_or_else(|| "OpenRouter model has not been authorized".to_owned())?;
    let mut authorization = InferenceAuthorization::default();
    authorization.openrouter_models.insert(model.clone());
    authorization.cloud_enabled = cloud_enabled;
    authorization.metered_enabled = metered_enabled;
    authorization.authorize(InferenceProvider::OpenRouter, &model)
        .map_err(|error| format!("OpenRouter authorization denied: {error:?}"))?;
    let key = credentials.secrets.read(SecretName::OpenRouterApiKey)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "OpenRouter credential not configured".to_owned())?;
    let transport = InferenceTransport::new().map_err(|error| error.to_string())?;
    transport.generate(&authorization, InferenceProvider::OpenRouter, &model, &prompt, Some(key.as_str()))
        .await.map_err(|error| error.to_string())
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
    credentials.configure(api_key).map_err(|error| error.to_string())
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
    use anyhow::Result;
    use std::sync::Mutex;
    use zeroize::Zeroizing;

    #[derive(Default)]
    struct FakeSecretStore(Mutex<Option<String>>);

    impl SecretStore for FakeSecretStore {
        fn read(&self, _name: SecretName) -> Result<Option<Zeroizing<String>>> {
            Ok(self.0.lock().unwrap().clone().map(Zeroizing::new))
        }
        fn write(&self, _name: SecretName, value: &str) -> Result<()> {
            *self.0.lock().unwrap() = Some(value.to_owned());
            Ok(())
        }
        fn delete(&self, _name: SecretName) -> Result<()> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn credential_lifecycle_never_returns_secret() -> Result<()> {
        let credentials = OpenRouterCredentials::with_secrets(Arc::new(FakeSecretStore::default()));
        assert!(!credentials.status()?.configured);
        assert!(credentials.configure("short".to_owned()).is_err());
        assert!(!credentials.status()?.configured);
        let status = credentials.configure("sk-or-v1-test-secret-for-tests".to_owned())?;
        assert!(status.configured);
        let json = serde_json::to_string(&status)?;
        assert!(!json.contains("sk-or"));
        assert!(credentials.configure("bad key with spaces".into()).is_err());
        assert!(credentials.status()?.configured);
        assert!(!credentials.remove()?.configured);
        Ok(())
    }
}

use anyhow::{anyhow, Context, Result};
use keyring::{Entry, Error as KeyringError};
use std::sync::Mutex;
use zeroize::Zeroizing;

const SERVICE_NAME: &str = "com.mythologiq.bob";
const GEMINI_ACCOUNT: &str = "gemini-api-key";
const OPENROUTER_ACCOUNT: &str = "openrouter-api-key";

#[derive(Clone, Copy)]
pub enum SecretName {
    GeminiApiKey,
    OpenRouterApiKey,
}

impl SecretName {
    fn account(self) -> &'static str {
        match self {
            Self::GeminiApiKey => GEMINI_ACCOUNT,
            Self::OpenRouterApiKey => OPENROUTER_ACCOUNT,
        }
    }
}

pub trait SecretStore: Send + Sync {
    fn read(&self, name: SecretName) -> Result<Option<Zeroizing<String>>>;
    fn write(&self, name: SecretName, value: &str) -> Result<()>;
    fn delete(&self, name: SecretName) -> Result<()>;
}

#[derive(Default)]
pub struct OsSecretStore {
    operation: Mutex<()>,
}

impl OsSecretStore {
    fn entry(&self, name: SecretName) -> Result<Entry> {
        Entry::new(SERVICE_NAME, name.account()).context("open protected OS credential entry")
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>> {
        self.operation
            .lock()
            .map_err(|_| anyhow!("protected OS credential operation lock is poisoned"))
    }
}

impl SecretStore for OsSecretStore {
    fn read(&self, name: SecretName) -> Result<Option<Zeroizing<String>>> {
        let _guard = self.lock()?;
        match self.entry(name)?.get_password() {
            Ok(value) => Ok(Some(Zeroizing::new(value))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(error).context("read protected OS credential"),
        }
    }

    fn write(&self, name: SecretName, value: &str) -> Result<()> {
        let _guard = self.lock()?;
        self.entry(name)?
            .set_password(value)
            .context("write protected OS credential")
    }

    fn delete(&self, name: SecretName) -> Result<()> {
        let _guard = self.lock()?;
        match self.entry(name)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(error).context("delete protected OS credential"),
        }
    }
}

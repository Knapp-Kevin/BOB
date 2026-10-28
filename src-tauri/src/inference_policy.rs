//! Fail-closed authorization for B.O.B.'s user-selected inference models.
//! Provider selection and remote transport are deliberately separate from this gate.
//! Never derive an allowlist from a provider's public model catalog.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceProvider {
    OpenRouter,
    Ollama,
    LmStudio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBilling {
    Local,
    Metered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceAuthorizationError {
    EmptyModel,
    InvalidModel,
    ModelNotAllowed,
    MeteredDisabled,
    CloudDisabled,
    LocalityMismatch,
}

/// Explicit user configuration, not an automatically populated model catalog.
#[derive(Debug, Clone)]
pub struct InferenceAuthorization {
    pub openrouter_models: BTreeSet<String>,
    pub ollama_models: BTreeSet<String>,
    pub lm_studio_models: BTreeSet<String>,
    pub metered_enabled: bool,
    pub cloud_enabled: bool,
}

impl Default for InferenceAuthorization {
    fn default() -> Self {
        Self {
            openrouter_models: BTreeSet::new(),
            ollama_models: BTreeSet::new(),
            lm_studio_models: BTreeSet::new(),
            metered_enabled: false,
            cloud_enabled: false,
        }
    }
}

impl InferenceAuthorization {
    /// Returns a provider-specific billing/locality class only when the
    /// exact model is explicitly authorized. No wildcard or prefix matching.
    pub fn authorize(
        &self,
        provider: InferenceProvider,
        model: &str,
    ) -> Result<ModelBilling, InferenceAuthorizationError> {
        if model.is_empty() {
            return Err(InferenceAuthorizationError::EmptyModel);
        }
        if model.trim() != model
            || model.len() > 256
            || model.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(InferenceAuthorizationError::InvalidModel);
        }

        let allowed = match provider {
            InferenceProvider::OpenRouter => &self.openrouter_models,
            InferenceProvider::Ollama => &self.ollama_models,
            InferenceProvider::LmStudio => &self.lm_studio_models,
        };
        if !allowed.contains(model) {
            return Err(InferenceAuthorizationError::ModelNotAllowed);
        }
        match provider {
            InferenceProvider::OpenRouter => {
                if !self.cloud_enabled {
                    return Err(InferenceAuthorizationError::CloudDisabled);
                }
                if !self.metered_enabled {
                    return Err(InferenceAuthorizationError::MeteredDisabled);
                }
                Ok(ModelBilling::Metered)
            }
            InferenceProvider::Ollama | InferenceProvider::LmStudio => {
                Ok(ModelBilling::Local)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_policy_blocks_every_provider() {
        let policy = InferenceAuthorization::default();
        for provider in [
            InferenceProvider::OpenRouter,
            InferenceProvider::Ollama,
            InferenceProvider::LmStudio,
        ] {
            assert_eq!(
                policy.authorize(provider, "model"),
                Err(InferenceAuthorizationError::ModelNotAllowed)
            );
        }
    }

    #[test]
    fn provider_allowlists_are_disjoint_and_exact() {
        let mut policy = InferenceAuthorization::default();
        policy.ollama_models.insert("gemma3:4b".into());
        assert_eq!(
            policy.authorize(InferenceProvider::Ollama, "gemma3:4b"),
            Ok(ModelBilling::Local)
        );
        assert_eq!(
            policy.authorize(InferenceProvider::LmStudio, "gemma3:4b"),
            Err(InferenceAuthorizationError::ModelNotAllowed)
        );
        assert_eq!(
            policy.authorize(InferenceProvider::Ollama, "gemma3"),
            Err(InferenceAuthorizationError::ModelNotAllowed)
        );
    }

    #[test]
    fn openrouter_requires_cloud_and_metered_opt_in() {
        let mut policy = InferenceAuthorization::default();
        policy
            .openrouter_models
            .insert("provider/approved-model".into());
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "provider/approved-model"),
            Err(InferenceAuthorizationError::CloudDisabled)
        );
        policy.cloud_enabled = true;
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "provider/approved-model"),
            Err(InferenceAuthorizationError::MeteredDisabled)
        );
        policy.metered_enabled = true;
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "provider/approved-model"),
            Ok(ModelBilling::Metered)
        );
    }

    #[test]
    fn model_identifiers_cannot_contain_whitespace_or_controls() {
        let mut policy = InferenceAuthorization::default();
        policy.lm_studio_models.insert("model".into());
        for invalid in ["", " model", "model ", "model\\nother", "model other"] {
            assert!(policy.authorize(InferenceProvider::LmStudio, invalid).is_err());
        }
        assert_eq!(
            policy.authorize(InferenceProvider::LmStudio, "model"),
            Ok(ModelBilling::Local)
        );
    }
}

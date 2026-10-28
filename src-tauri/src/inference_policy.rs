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
    RouterModelNotAllowed,
}

/// OpenRouter model IDs must name one `vendor/model`. OpenRouter's own
/// `openrouter/*` routers (for example `openrouter/auto`) choose a different
/// underlying model per request, so approving one would not bind the user's
/// model, cost, or privacy choice. They are rejected outright.
pub fn validate_openrouter_model_id(model: &str) -> Result<(), InferenceAuthorizationError> {
    validate_model_id(model)?;
    let Some((vendor, name)) = model.split_once('/') else {
        return Err(InferenceAuthorizationError::InvalidModel);
    };
    if vendor.is_empty() || name.is_empty() || name.contains('/') {
        return Err(InferenceAuthorizationError::InvalidModel);
    }
    if vendor.eq_ignore_ascii_case("openrouter") {
        return Err(InferenceAuthorizationError::RouterModelNotAllowed);
    }
    Ok(())
}

fn validate_model_id(model: &str) -> Result<(), InferenceAuthorizationError> {
    if model.is_empty() {
        return Err(InferenceAuthorizationError::EmptyModel);
    }
    if model.trim() != model
        || model.len() > 256
        || model.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(InferenceAuthorizationError::InvalidModel);
    }
    Ok(())
}

/// Explicit user configuration, not an automatically populated model catalog.
/// The derived default is empty allowlists with cloud and metered use denied.
#[derive(Debug, Clone, Default)]
pub struct InferenceAuthorization {
    pub openrouter_models: BTreeSet<String>,
    pub ollama_models: BTreeSet<String>,
    pub lm_studio_models: BTreeSet<String>,
    pub metered_enabled: bool,
    pub cloud_enabled: bool,
}

impl InferenceAuthorization {
    /// Returns a provider-specific billing/locality class only when the
    /// exact model is explicitly authorized. No wildcard or prefix matching.
    pub fn authorize(
        &self,
        provider: InferenceProvider,
        model: &str,
    ) -> Result<ModelBilling, InferenceAuthorizationError> {
        match provider {
            InferenceProvider::OpenRouter => validate_openrouter_model_id(model)?,
            InferenceProvider::Ollama | InferenceProvider::LmStudio => validate_model_id(model)?,
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
            InferenceProvider::Ollama | InferenceProvider::LmStudio => Ok(ModelBilling::Local),
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
                policy.authorize(provider, "vendor/model"),
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
    fn openrouter_rejects_router_and_malformed_model_ids() {
        let mut policy = InferenceAuthorization {
            cloud_enabled: true,
            metered_enabled: true,
            ..Default::default()
        };
        for id in [
            "openrouter/auto",
            "OpenRouter/auto",
            "no-vendor",
            "/m",
            "v/",
            "a/b/c",
        ] {
            policy.openrouter_models.insert(id.into());
            assert!(
                policy.authorize(InferenceProvider::OpenRouter, id).is_err(),
                "{id} must be rejected even when allowlisted"
            );
        }
        assert_eq!(
            validate_openrouter_model_id("openrouter/auto"),
            Err(InferenceAuthorizationError::RouterModelNotAllowed)
        );
        assert!(validate_openrouter_model_id("meta-llama/llama-3.1-8b-instruct:free").is_ok());
    }

    #[test]
    fn model_identifiers_cannot_contain_whitespace_or_controls() {
        let mut policy = InferenceAuthorization::default();
        policy.lm_studio_models.insert("model".into());
        for invalid in ["", " model", "model ", "model\\nother", "model other"] {
            assert!(policy
                .authorize(InferenceProvider::LmStudio, invalid)
                .is_err());
        }
        assert_eq!(
            policy.authorize(InferenceProvider::LmStudio, "model"),
            Ok(ModelBilling::Local)
        );
    }
}

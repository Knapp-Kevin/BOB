//! Fail-closed authorization for B.O.B.'s user-selected inference models.
//! Provider selection and remote transport are deliberately separate from this gate.
//! Never derive an allowlist from a provider's public model catalog.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceProvider {
    OpenRouter,
    Ollama,
    LmStudio,
    OpenAi,
    Anthropic,
    ClaudeCli,
    CodexCli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBilling {
    Free,
    Local,
    Metered,
    Subscription,
    Unknown,
}

/// Billing class for `claude -p` driven by B.O.B. Anthropic's Claude Code
/// legal page does not clearly permit a third-party application to drive a
/// user's subscription through the CLI, and an API-key or extra-usage login is
/// metered, so the class stays `Unknown` (blocked) until the owner decides.
pub const CLAUDE_CLI_BILLING: ModelBilling = ModelBilling::Unknown;

/// Billing class for `codex exec` driven by B.O.B. OpenAI's Codex auth docs
/// direct programmatic workflows to API keys and do not address third-party
/// applications using ChatGPT-plan sign-in, so the class stays `Unknown`.
pub const CODEX_CLI_BILLING: ModelBilling = ModelBilling::Unknown;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceAuthorizationError {
    EmptyModel,
    InvalidModel,
    ModelNotAllowed,
    MeteredDisabled,
    CloudDisabled,
    LocalityMismatch,
    RouterModelNotAllowed,
    BillingUnknown,
    FreeModelRequired,
}

/// OpenRouter's zero-priced variants carry a `:free` suffix on the model name.
pub fn is_openrouter_free_model(model: &str) -> bool {
    model.ends_with(":free") && model.len() > ":free".len()
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

/// Direct-vendor and CLI model IDs are also passed as process arguments, so
/// they are restricted to a conservative character set and cannot start with
/// `-` (no option injection). Vendor IDs never contain a `/` route prefix.
pub fn validate_vendor_model_id(model: &str) -> Result<(), InferenceAuthorizationError> {
    validate_model_id(model)?;
    let first_ok = model
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphanumeric());
    if !first_ok
        || model.len() > 128
        || !model
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | ':' | '@'))
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
    pub openai_models: BTreeSet<String>,
    pub anthropic_models: BTreeSet<String>,
    pub claude_cli_models: BTreeSet<String>,
    pub codex_cli_models: BTreeSet<String>,
    /// OpenRouter free-only mode: only `:free` models, billed `Free`, and the
    /// transport caps every price at zero so OpenRouter refuses paid routes.
    pub openrouter_free_only: bool,
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
            InferenceProvider::OpenAi
            | InferenceProvider::Anthropic
            | InferenceProvider::ClaudeCli
            | InferenceProvider::CodexCli => validate_vendor_model_id(model)?,
        }

        let allowed = match provider {
            InferenceProvider::OpenRouter => &self.openrouter_models,
            InferenceProvider::Ollama => &self.ollama_models,
            InferenceProvider::LmStudio => &self.lm_studio_models,
            InferenceProvider::OpenAi => &self.openai_models,
            InferenceProvider::Anthropic => &self.anthropic_models,
            InferenceProvider::ClaudeCli => &self.claude_cli_models,
            InferenceProvider::CodexCli => &self.codex_cli_models,
        };
        if !allowed.contains(model) {
            return Err(InferenceAuthorizationError::ModelNotAllowed);
        }
        match provider {
            InferenceProvider::OpenRouter if self.openrouter_free_only => {
                if !is_openrouter_free_model(model) {
                    return Err(InferenceAuthorizationError::FreeModelRequired);
                }
                self.cloud(ModelBilling::Free)
            }
            InferenceProvider::OpenRouter
            | InferenceProvider::OpenAi
            | InferenceProvider::Anthropic => self.cloud(ModelBilling::Metered),
            InferenceProvider::ClaudeCli => self.cloud(CLAUDE_CLI_BILLING),
            InferenceProvider::CodexCli => self.cloud(CODEX_CLI_BILLING),
            InferenceProvider::Ollama | InferenceProvider::LmStudio => Ok(ModelBilling::Local),
        }
    }

    /// Cloud routes always need cloud permission; metered routes also need the
    /// metered permission, and an unclassified route is never allowed.
    fn cloud(&self, billing: ModelBilling) -> Result<ModelBilling, InferenceAuthorizationError> {
        if billing == ModelBilling::Unknown {
            return Err(InferenceAuthorizationError::BillingUnknown);
        }
        if !self.cloud_enabled {
            return Err(InferenceAuthorizationError::CloudDisabled);
        }
        if billing == ModelBilling::Metered && !self.metered_enabled {
            return Err(InferenceAuthorizationError::MeteredDisabled);
        }
        Ok(billing)
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
            InferenceProvider::OpenAi,
            InferenceProvider::Anthropic,
            InferenceProvider::ClaudeCli,
            InferenceProvider::CodexCli,
        ] {
            let model = match provider {
                InferenceProvider::OpenRouter => "vendor/model",
                _ => "model",
            };
            assert_eq!(
                policy.authorize(provider, model),
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
    fn openrouter_free_only_needs_free_model_and_cloud_but_not_metered() {
        let mut policy = InferenceAuthorization {
            openrouter_free_only: true,
            ..Default::default()
        };
        policy.openrouter_models.insert("vendor/model:free".into());
        policy.openrouter_models.insert("vendor/model".into());
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "vendor/model:free"),
            Err(InferenceAuthorizationError::CloudDisabled)
        );
        policy.cloud_enabled = true;
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "vendor/model:free"),
            Ok(ModelBilling::Free)
        );
        policy.metered_enabled = true;
        assert_eq!(
            policy.authorize(InferenceProvider::OpenRouter, "vendor/model"),
            Err(InferenceAuthorizationError::FreeModelRequired),
            "metered permission never widens free-only mode"
        );
        assert!(!is_openrouter_free_model(":free"));
        assert!(!is_openrouter_free_model("vendor/model:freebie"));
        policy.openrouter_models.insert("openrouter/free".into());
        assert!(policy
            .authorize(InferenceProvider::OpenRouter, "openrouter/free")
            .is_err());
    }

    #[test]
    fn direct_vendor_apis_require_cloud_and_metered_opt_in() {
        for (provider, model) in [
            (InferenceProvider::OpenAi, "gpt-test-mini"),
            (InferenceProvider::Anthropic, "claude-test-5"),
        ] {
            let mut policy = InferenceAuthorization::default();
            policy.openai_models.insert("gpt-test-mini".into());
            policy.anthropic_models.insert("claude-test-5".into());
            assert_eq!(
                policy.authorize(provider, model),
                Err(InferenceAuthorizationError::CloudDisabled)
            );
            policy.cloud_enabled = true;
            assert_eq!(
                policy.authorize(provider, model),
                Err(InferenceAuthorizationError::MeteredDisabled)
            );
            policy.metered_enabled = true;
            assert_eq!(policy.authorize(provider, model), Ok(ModelBilling::Metered));
        }
        let mut policy = InferenceAuthorization {
            cloud_enabled: true,
            metered_enabled: true,
            ..Default::default()
        };
        policy.openai_models.insert("gpt-test-mini".into());
        assert_eq!(
            policy.authorize(InferenceProvider::Anthropic, "gpt-test-mini"),
            Err(InferenceAuthorizationError::ModelNotAllowed),
            "allowlists stay per provider"
        );
    }

    #[test]
    fn cli_runtimes_with_unknown_billing_are_always_blocked() {
        assert_eq!(CLAUDE_CLI_BILLING, ModelBilling::Unknown);
        assert_eq!(CODEX_CLI_BILLING, ModelBilling::Unknown);
        let mut policy = InferenceAuthorization {
            cloud_enabled: true,
            metered_enabled: true,
            ..Default::default()
        };
        policy.claude_cli_models.insert("sonnet".into());
        policy.codex_cli_models.insert("gpt-test".into());
        assert_eq!(
            policy.authorize(InferenceProvider::ClaudeCli, "sonnet"),
            Err(InferenceAuthorizationError::BillingUnknown)
        );
        assert_eq!(
            policy.authorize(InferenceProvider::CodexCli, "gpt-test"),
            Err(InferenceAuthorizationError::BillingUnknown)
        );
    }

    #[test]
    fn vendor_model_ids_cannot_inject_options_or_routes() {
        for invalid in ["-p", "--tools", "vendor/model", "a b", "x;y", "m\"x", ""] {
            assert!(validate_vendor_model_id(invalid).is_err(), "{invalid}");
        }
        for valid in [
            "sonnet",
            "claude-test-5",
            "gpt-4.1-mini",
            "o4-mini:2026",
            "m_1@v2",
        ] {
            assert!(validate_vendor_model_id(valid).is_ok(), "{valid}");
        }
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

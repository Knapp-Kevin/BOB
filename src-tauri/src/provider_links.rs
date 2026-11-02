//! Opens a fixed provider setup page (API keys, spending limits, pricing, free
//! models) in the user's default browser. Only the allowlisted URLs below can
//! be opened; the frontend names a page, never a URL.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderPage {
    OpenrouterKeys,
    OpenrouterFreeModels,
    OpenaiKeys,
    OpenaiLimits,
    OpenaiPricing,
    AnthropicKeys,
    AnthropicLimits,
    AnthropicPricing,
}

impl ProviderPage {
    pub fn url(self) -> &'static str {
        match self {
            Self::OpenrouterKeys => "https://openrouter.ai/settings/keys",
            Self::OpenrouterFreeModels => "https://openrouter.ai/models?max_price=0",
            Self::OpenaiKeys => "https://platform.openai.com/api-keys",
            Self::OpenaiLimits => "https://platform.openai.com/settings/organization/limits",
            Self::OpenaiPricing => "https://platform.openai.com/docs/pricing",
            Self::AnthropicKeys => "https://platform.claude.com/settings/keys",
            Self::AnthropicLimits => "https://platform.claude.com/settings/limits",
            Self::AnthropicPricing => "https://platform.claude.com/docs/en/about-claude/pricing",
        }
    }
}

#[cfg(windows)]
fn open_in_browser(url: &str) -> Result<(), String> {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let verb = wide("open");
    let target = wide(url);
    // SAFETY: both buffers are NUL-terminated UTF-16 and outlive the call.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW reports success with a value greater than 32.
    if result as isize > 32 {
        Ok(())
    } else {
        Err("could not open the browser".into())
    }
}

#[cfg(not(windows))]
fn open_in_browser(url: &str) -> Result<(), String> {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|_| "could not open the browser".into())
}

#[tauri::command]
pub fn open_provider_page(page: ProviderPage) -> Result<(), String> {
    open_in_browser(page.url())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_is_a_fixed_https_vendor_url() {
        let pages = [
            ProviderPage::OpenrouterKeys,
            ProviderPage::OpenrouterFreeModels,
            ProviderPage::OpenaiKeys,
            ProviderPage::OpenaiLimits,
            ProviderPage::OpenaiPricing,
            ProviderPage::AnthropicKeys,
            ProviderPage::AnthropicLimits,
            ProviderPage::AnthropicPricing,
        ];
        for page in pages {
            let url = page.url();
            assert!(
                [
                    "https://openrouter.ai/",
                    "https://platform.openai.com/",
                    "https://platform.claude.com/"
                ]
                .iter()
                .any(|origin| url.starts_with(origin)),
                "{url}"
            );
        }
        let parsed: ProviderPage = serde_json::from_str("\"openai_keys\"").unwrap();
        assert_eq!(parsed, ProviderPage::OpenaiKeys);
        assert!(serde_json::from_str::<ProviderPage>("\"https://evil.example\"").is_err());
    }
}

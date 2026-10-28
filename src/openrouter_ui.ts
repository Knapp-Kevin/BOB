import { configureOpenRouterCredential, configureOpenRouterModel, loadOpenRouterCredentialStatus, loadOpenRouterSettings, removeOpenRouterCredential, type OpenRouterSettings } from "./native";

export let openRouterSettings: OpenRouterSettings = { modelId: null, cloudEnabled: false, meteredEnabled: false };
export let openRouterCredentialConfigured = false;
export let openRouterBusy = false;
// Session-only Chat opt-in; starts disabled on every launch.
export let openRouterChatEnabled = false;
export function setOpenRouterChatEnabled(enabled: boolean) { openRouterChatEnabled = enabled; }
export let openRouterStatus = "Cloud inference is disabled until explicitly configured.";

export async function refreshOpenRouter() {
  try {
    const [settings, credential] = await Promise.all([loadOpenRouterSettings(), loadOpenRouterCredentialStatus()]);
    if (settings) openRouterSettings = settings;
    openRouterCredentialConfigured = credential.configured;
    openRouterStatus = settings === null
      ? "Provider configuration is available in the desktop application."
      : "Authorization and credentials are stored locally.";
  } catch {
    openRouterStatus = "Could not read OpenRouter configuration.";
  }
}

export async function saveOpenRouterKey(key: string) {
  if (openRouterBusy) return;
  openRouterBusy = true;
  try {
    const status = await configureOpenRouterCredential(key);
    openRouterCredentialConfigured = status.configured;
    openRouterStatus = "Credential saved to protected operating-system storage.";
  } catch {
    openRouterStatus = "Credential was not saved. Existing credentials were preserved.";
  } finally {
    openRouterBusy = false;
  }
}

export async function removeOpenRouterKey() {
  if (openRouterBusy) return;
  openRouterBusy = true;
  try {
    await removeOpenRouterCredential();
    openRouterCredentialConfigured = false;
    openRouterChatEnabled = false;
    openRouterStatus = "Credential removed.";
  } catch {
    openRouterStatus = "Credential removal failed.";
  } finally {
    openRouterBusy = false;
  }
}

export async function saveOpenRouterAuthorization(settings: OpenRouterSettings) {
  if (openRouterBusy) return;
  openRouterBusy = true;
  try {
    const result = await configureOpenRouterModel(settings);
    if (!result) throw new Error("desktop required");
    openRouterSettings = result;
    // Every model or billing policy change requires a fresh session opt-in.
    openRouterChatEnabled = false;
    openRouterStatus = result.cloudEnabled && result.meteredEnabled
      ? "Cloud and metered inference authorized for the selected model."
      : "OpenRouter spending remains disabled.";
  } catch {
    openRouterStatus = "Model authorization was not saved.";
  } finally {
    openRouterBusy = false;
  }
}

import { configureOpenRouterCredential, configureOpenRouterModel, loadOpenRouterCredentialStatus, loadOpenRouterKeyStatus, loadOpenRouterSettings, removeOpenRouterCredential, setOpenRouterChatSession, type OpenRouterAuthorization, type OpenRouterKeyStatus, type OpenRouterSettings } from "./native";

const disabledSettings = (): OpenRouterSettings => ({ modelId: null, cloudEnabled: false, meteredEnabled: false, freeOnly: false, chatSessionEnabled: false });

export let openRouterSettings: OpenRouterSettings = disabledSettings();
export let openRouterCredentialConfigured = false;
export let openRouterBusy = false;
// Mirrors the Rust-held session grant, which starts empty on every launch.
export let openRouterChatEnabled = false;
export let openRouterStatus = "Cloud inference is disabled until explicitly configured.";

function adopt(settings: OpenRouterSettings) {
  openRouterSettings = settings;
  openRouterChatEnabled = settings.chatSessionEnabled;
}

export const isFreeModel = (modelId: string | null) => Boolean(modelId && modelId.endsWith(":free") && modelId.length > 5);

export const openRouterChatAvailable = () => openRouterCredentialConfigured
  && Boolean(openRouterSettings.modelId)
  && openRouterSettings.cloudEnabled
  && (openRouterSettings.freeOnly ? isFreeModel(openRouterSettings.modelId) : openRouterSettings.meteredEnabled);

export let openRouterKeyStatus: OpenRouterKeyStatus | "checking" | "failed" | null = null;

export async function checkOpenRouterKey() {
  openRouterKeyStatus = "checking";
  try {
    openRouterKeyStatus = await loadOpenRouterKeyStatus();
  } catch {
    openRouterKeyStatus = "failed";
  }
}

const dollars = (value: number) => `$${value.toFixed(2)}`;

export function openRouterKeyCopy(): string {
  const status = openRouterKeyStatus;
  if (status === null) return "Key limits not checked yet.";
  if (status === "checking") return "Checking key limits…";
  if (status === "failed") return "Could not check the key. Paid OpenRouter Chat stays off until the key check succeeds.";
  const cap = status.limit === null
    ? "No credit limit on this key. Paid OpenRouter Chat is refused until you set one at openrouter.ai → Keys."
    : `Credit limit ${dollars(status.limit)}, ${dollars(status.limitRemaining ?? 0)} left.`;
  const free = status.freeRequestsRemainingToday === null ? "" : ` ${status.freeRequestsRemainingToday} free-model requests left today.`;
  return `${cap}${free}`;
}

export async function refreshOpenRouter() {
  try {
    const [settings, credential] = await Promise.all([loadOpenRouterSettings(), loadOpenRouterCredentialStatus()]);
    if (settings) adopt(settings);
    else openRouterChatEnabled = false;
    openRouterCredentialConfigured = credential.configured;
    openRouterStatus = settings === null
      ? "Provider configuration is available in the desktop application."
      : "Authorization and credentials are stored locally.";
  } catch {
    openRouterSettings = disabledSettings();
    openRouterChatEnabled = false;
    openRouterCredentialConfigured = false;
    openRouterStatus = "Could not read OpenRouter configuration. Cloud Chat is disabled.";
  }
}

export async function setOpenRouterChatEnabled(enabled: boolean): Promise<boolean> {
  if (openRouterBusy) return false;
  openRouterBusy = true;
  // Fail closed locally first; only the backend's answer can turn it on.
  if (enabled) openRouterChatEnabled = false;
  try {
    adopt(await setOpenRouterChatSession(enabled));
    return true;
  } catch (error) {
    openRouterChatEnabled = false;
    const detail = String(error);
    openRouterStatus = !enabled
      ? "OpenRouter Chat could not be confirmed off. Restart B.O.B. to clear the session."
      : /uncapped key refused/.test(detail)
        ? "OpenRouter Chat was not enabled: paid use needs a key with a credit limit and credit left. Set one at openrouter.ai → Keys, or switch to free models only."
        : /another cloud Chat backend/.test(detail)
          ? "OpenRouter Chat was not enabled: turn off the other cloud provider first."
          : "OpenRouter Chat could not be enabled. Check the API key and model authorization.";
    return false;
  } finally {
    openRouterBusy = false;
  }
}

export async function saveOpenRouterKey(key: string) {
  if (openRouterBusy) return;
  openRouterBusy = true;
  // The backend revokes the session grant before replacing the credential.
  openRouterChatEnabled = false;
  try {
    const status = await configureOpenRouterCredential(key);
    openRouterCredentialConfigured = status.configured;
    openRouterStatus = "Credential saved to protected operating-system storage. Re-enable cloud Chat if desired.";
  } catch {
    openRouterStatus = "Credential was not saved. Check the key format and try again.";
  } finally {
    openRouterBusy = false;
  }
}

export async function removeOpenRouterKey() {
  if (openRouterBusy) return;
  openRouterBusy = true;
  openRouterChatEnabled = false;
  try {
    await removeOpenRouterCredential();
    openRouterCredentialConfigured = false;
    openRouterStatus = "Credential removed.";
  } catch {
    openRouterStatus = "Credential removal failed.";
  } finally {
    openRouterBusy = false;
  }
}

export async function saveOpenRouterAuthorization(settings: OpenRouterAuthorization) {
  if (openRouterBusy) return;
  openRouterBusy = true;
  // Every model or billing policy change requires a fresh session opt-in.
  openRouterChatEnabled = false;
  try {
    const result = await configureOpenRouterModel(settings);
    if (!result) throw new Error("desktop required");
    adopt(result);
    openRouterStatus = result.freeOnly
      ? (result.cloudEnabled ? "Free models only: every request is capped at a price of zero." : "Free models only. Cloud transmission is still off.")
      : result.cloudEnabled && result.meteredEnabled
        ? "Cloud and metered inference authorized for the selected model. The key must have a credit limit."
        : "OpenRouter spending remains disabled.";
  } catch {
    openRouterStatus = "Model authorization was not saved. Use a vendor/model ID; OpenRouter routers such as openrouter/auto or openrouter/free are not allowed.";
  } finally {
    openRouterBusy = false;
  }
}

/** Maps a backend failure to user-facing copy. Backend errors never carry credentials. */
export function openRouterFailureMessage(error: unknown): string {
  const detail = String(error);
  if (/uncapped key refused/.test(detail)) return "OpenRouter Chat was turned off: the key has no credit limit or no credit left.";
  if (/reported a charge/.test(detail)) return "OpenRouter could not confirm this free-only reply was free, so Chat was turned off.";
  if (/HTTP 404/.test(detail)) return "No OpenRouter host for this model meets B.O.B.'s price and privacy limits.";
  if (/HTTP 401|HTTP 403/.test(detail)) return "OpenRouter rejected the saved API key.";
  if (/HTTP 402/.test(detail)) return "OpenRouter reports insufficient credits.";
  if (/HTTP 429/.test(detail)) return "OpenRouter is rate limiting requests.";
  if (/not enabled|model changed|authorization denied|credential not configured|not been authorized/.test(detail)) {
    return "OpenRouter Chat is no longer authorized. Re-enable it in Settings.";
  }
  return "OpenRouter could not reply.";
}

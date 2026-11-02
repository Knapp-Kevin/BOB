import { configureAdapterCredential, configureAdapterModel, generateWithAdapter, loadAdapterSettings, removeAdapterCredential, setAdapterChatSession, type AdapterId, type AdapterSettings } from "./native";
import { escapeHtml, showToast } from "./model";

type AdapterInfo = { label: string; keyLabel: string; modelPlaceholder: string; about: string };

const ADAPTERS: Record<AdapterId, AdapterInfo> = {
  openai: { label: "OpenAI API", keyLabel: "OpenAI API key", modelPlaceholder: "model ID from your OpenAI account", about: "Calls the OpenAI API directly with your own API key. Usage is billed to that OpenAI account. B.O.B. asks OpenAI not to store the completion and never substitutes another model." },
  anthropic: { label: "Anthropic API", keyLabel: "Anthropic API key", modelPlaceholder: "model ID from your Anthropic Console", about: "Calls the Anthropic Messages API directly with your own API key. Usage is billed to that Anthropic Console account. B.O.B. never substitutes another model." }
};

const ORDER: AdapterId[] = ["openai", "anthropic"];

export let adapterSettings: AdapterSettings[] = [];
export let adapterStatus = "";
let adapterBusy = false;

const settingsFor = (adapter: AdapterId) => adapterSettings.find((entry) => entry.adapter === adapter);
export const adapterLabel = (adapter: AdapterId) => ADAPTERS[adapter].label;

/** The adapter holding this session's Chat grant, mirrored from the Rust-held grant. */
export const activeAdapter = (): AdapterSettings | undefined => adapterSettings.find((entry) => entry.chatSessionEnabled);

const adopt = (settings: AdapterSettings[]) => { adapterSettings = settings; };

export async function refreshAdapters() {
  try {
    const settings = await loadAdapterSettings();
    adopt(settings ?? []);
    adapterStatus = settings === null ? "Provider configuration is available in the desktop application." : "";
  } catch {
    adapterSettings = [];
    adapterStatus = "Could not read provider configuration. These providers are disabled.";
  }
}

async function guarded(action: () => Promise<AdapterSettings[]>, success: string, failure: string) {
  if (adapterBusy) return false;
  adapterBusy = true;
  try {
    adopt(await action());
    adapterStatus = success;
    return true;
  } catch (error) {
    adapterStatus = `${failure}${blockedReason(error)}`;
    await refreshAdapters();
    adapterStatus = `${failure}${blockedReason(error)}`;
    return false;
  } finally {
    adapterBusy = false;
  }
}

function blockedReason(error: unknown): string {
  const detail = String(error);
  if (/another cloud Chat backend/.test(detail)) return " Turn off the other cloud Chat provider first.";
  if (/CloudDisabled/.test(detail)) return " Cloud transmission is not allowed.";
  if (/MeteredDisabled/.test(detail)) return " Separately billed inference is not allowed.";
  if (/credential not configured/.test(detail)) return " Store an API key first.";
  if (/not been authorized/.test(detail)) return " Save a model authorization first.";
  return "";
}

/** Maps a backend failure to user-facing copy. Backend errors never carry credentials or CLI output. */
export function adapterFailureMessage(adapter: AdapterId, error: unknown): string {
  const label = ADAPTERS[adapter].label;
  const detail = String(error);
  if (/HTTP 401|HTTP 403/.test(detail)) return `${label} rejected the saved API key.`;
  if (/HTTP 402/.test(detail)) return `${label} reports a billing problem on the account.`;
  if (/HTTP 429|HTTP 529/.test(detail)) return `${label} is rate limiting or overloaded.`;
  if (/not enabled|model changed|authorization denied|credential not configured|not been authorized/.test(detail)) return `${label} Chat is no longer authorized. Re-enable it in Settings.`;
  return `${label} could not reply.`;
}

export const generateWithActiveAdapter = (adapter: AdapterId, prompt: string): Promise<string> => generateWithAdapter(adapter, prompt);

function adapterCard(adapter: AdapterId) {
  const info = ADAPTERS[adapter];
  const settings = settingsFor(adapter);
  const available = Boolean(settings?.modelId && settings.cloudEnabled && settings.meteredEnabled && settings.credentialConfigured);
  const id = (suffix: string) => `adapter-${adapter}-${suffix}`;
  const credential = `<p>${settings?.credentialConfigured ? "API key configured" : "No API key configured"}</p><form data-adapter-credential="${adapter}"><label class="field">${info.keyLabel}<input id="${id("key")}" type="password" autocomplete="new-password" placeholder="Paste key to replace saved credential"></label><button type="submit" class="primary">Store key securely</button></form><button class="plain" data-adapter-remove-key="${adapter}">Remove ${escapeHtml(info.keyLabel)}</button>`;
  return `<article class="panel prefs openrouter adapter" aria-labelledby="${id("title")}"><small>CLOUD · METERED</small><h2 id="${id("title")}">${info.label}</h2><p>${escapeHtml(info.about)}</p><div class="openrouter-columns"><div><form data-adapter-config="${adapter}"><label class="field">Approved model ID<input id="${id("model")}" autocomplete="off" value="${escapeHtml(settings?.modelId ?? "")}" placeholder="${escapeHtml(info.modelPlaceholder)}"></label><label class="check"><input type="checkbox" id="${id("cloud")}" ${settings?.cloudEnabled ? "checked" : ""}> Allow cloud transmission</label><label class="check"><input type="checkbox" id="${id("metered")}" ${settings?.meteredEnabled ? "checked" : ""}> Allow separately billed inference</label><p>Prompts leave this device. Both permissions are required for metered use; set a spending limit in the provider's own console.</p><button type="submit" class="primary">Save model authorization</button></form></div><div><label class="check"><input type="checkbox" data-adapter-chat="${adapter}" ${settings?.chatSessionEnabled ? "checked" : ""} ${!available || adapterBusy ? "disabled" : ""}> Use ${info.label} in Chat for this session</label><p>Off by default every launch. Only one cloud provider can be on at a time.</p>${credential}</div></div></article>`;
}

export function adapterCards() {
  return `${adapterStatus ? `<p class="adapter-status" role="status">${escapeHtml(adapterStatus)}</p>` : ""}${ORDER.map(adapterCard).join("")}`;
}

export function bindAdapterControls(render: () => void) {
  document.querySelectorAll<HTMLInputElement>("[data-adapter-chat]").forEach((input) => input.addEventListener("change", () => {
    const adapter = input.dataset.adapterChat as AdapterId;
    const label = ADAPTERS[adapter].label;
    if (input.checked && !window.confirm(`Send your Chat prompts to ${label} using your approved model? Prompts leave this device and usage may be billed. This only applies to this app session.`)) {
      render();
      return;
    }
    void guarded(() => setAdapterChatSession(adapter, input.checked), input.checked ? `${label} Chat enabled for this session.` : "Local B.O.B. Chat restored.", `${label} Chat could not be changed.`)
      .then((applied) => { if (applied) showToast(adapterStatus); })
      .finally(render);
  }));
  document.querySelectorAll<HTMLFormElement>("[data-adapter-config]").forEach((form) => form.addEventListener("submit", (event) => {
    event.preventDefault();
    const adapter = form.dataset.adapterConfig as AdapterId;
    const value = (suffix: string) => document.querySelector<HTMLInputElement>(`#adapter-${adapter}-${suffix}`);
    const modelId = value("model")?.value.trim() || null;
    const cloudEnabled = Boolean(value("cloud")?.checked);
    const meteredEnabled = Boolean(value("metered")?.checked);
    if (cloudEnabled && meteredEnabled && !window.confirm(`Authorize sending Chat prompts to ${ADAPTERS[adapter].label} using this model? This can incur separately billed usage.`)) return;
    void guarded(() => configureAdapterModel(adapter, { modelId, cloudEnabled, meteredEnabled }), "Model authorization saved. Re-enable Chat if desired.", "Model authorization was not saved. Use a plain model ID without spaces or slashes.").finally(render);
  }));
  document.querySelectorAll<HTMLFormElement>("[data-adapter-credential]").forEach((form) => form.addEventListener("submit", (event) => {
    event.preventDefault();
    const adapter = form.dataset.adapterCredential as AdapterId;
    const input = document.querySelector<HTMLInputElement>(`#adapter-${adapter}-key`);
    const key = input?.value ?? "";
    if (input) input.value = "";
    void guarded(() => configureAdapterCredential(adapter, key), "Credential saved to protected operating-system storage. Re-enable Chat if desired.", "Credential was not saved. Check the key format and try again.").finally(render);
  }));
  document.querySelectorAll<HTMLButtonElement>("[data-adapter-remove-key]").forEach((button) => button.addEventListener("click", () => {
    const adapter = button.dataset.adapterRemoveKey as AdapterId;
    if (!window.confirm(`Remove the saved ${ADAPTERS[adapter].label} key?`)) return;
    void guarded(() => removeAdapterCredential(adapter), "Credential removed.", "Credential removal failed.").finally(render);
  }));
}

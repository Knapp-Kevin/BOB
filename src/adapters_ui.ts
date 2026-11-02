import { cancelAdapterGeneration, configureAdapterCredential, configureAdapterModel, generateWithAdapter, loadAdapterSettings, loadCliRuntimeStatus, removeAdapterCredential, setAdapterChatSession, type AdapterId, type AdapterSettings, type CliRuntimeStatus } from "./native";
import { escapeHtml, showToast } from "./model";

type AdapterInfo = { label: string; cli: boolean; keyLabel?: string; modelPlaceholder: string; about: string };

const ADAPTERS: Record<AdapterId, AdapterInfo> = {
  openai: { label: "OpenAI API", cli: false, keyLabel: "OpenAI API key", modelPlaceholder: "model ID from your OpenAI account", about: "Calls the OpenAI API directly with your own API key. Usage is billed to that OpenAI account. B.O.B. asks OpenAI not to store the completion and never substitutes another model." },
  anthropic: { label: "Anthropic API", cli: false, keyLabel: "Anthropic API key", modelPlaceholder: "model ID from your Anthropic Console", about: "Calls the Anthropic Messages API directly with your own API key. Usage is billed to that Anthropic Console account. B.O.B. never substitutes another model." },
  claude_cli: { label: "Claude CLI", cli: true, modelPlaceholder: "model alias or ID, e.g. sonnet", about: "Runs your installed Claude Code CLI (claude -p) with all tools, hooks, plugins, and MCP servers off, in an empty folder. Sign in with the CLI itself; B.O.B. never sees that login." },
  codex_cli: { label: "Codex CLI", cli: true, modelPlaceholder: "model ID supported by Codex", about: "Runs your installed Codex CLI (codex exec) in its read-only sandbox, in an empty folder. B.O.B. stops it if it tries to run any command or tool. Sign in with the CLI itself; B.O.B. never sees that login." }
};

const ORDER: AdapterId[] = ["openai", "anthropic", "claude_cli", "codex_cli"];

export let adapterSettings: AdapterSettings[] = [];
export let adapterStatus = "";
let adapterBusy = false;
const cliStatus: Partial<Record<AdapterId, CliRuntimeStatus | "checking" | "failed">> = {};
// True while a provider request is in flight, so Chat can offer to stop it.
export let adapterGenerating = false;

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
  if (/BillingUnknown/.test(detail)) return " Billing for this CLI is unclassified, so B.O.B. blocks it until the owner records a billing decision.";
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
  if (/HTTP 401|HTTP 403|not signed in/.test(detail)) return ADAPTERS[adapter].cli ? `${label} is not signed in.` : `${label} rejected the saved API key.`;
  if (/HTTP 402/.test(detail)) return `${label} reports a billing problem on the account.`;
  if (/HTTP 429|HTTP 529/.test(detail)) return `${label} is rate limiting or overloaded.`;
  if (/timed out/.test(detail)) return `${label} took too long and was stopped.`;
  if (/cancelled/.test(detail)) return `${label} request was stopped.`;
  if (/attempted tool use/.test(detail)) return `${label} tried to use a tool and was stopped.`;
  if (/not installed/.test(detail)) return `${label} is not installed or not on PATH.`;
  if (/not enabled|model changed|authorization denied|credential not configured|not been authorized/.test(detail)) return `${label} Chat is no longer authorized. Re-enable it in Settings.`;
  return `${label} could not reply.`;
}

export async function generateWithActiveAdapter(adapter: AdapterId, prompt: string): Promise<string> {
  adapterGenerating = true;
  window.dispatchEvent(new Event("bob:render"));
  try {
    return await generateWithAdapter(adapter, prompt);
  } finally {
    adapterGenerating = false;
    window.dispatchEvent(new Event("bob:render"));
  }
}

function cliStatusCopy(adapter: AdapterId) {
  const status = cliStatus[adapter];
  if (status === undefined) return "Not checked yet.";
  if (status === "checking") return "Checking…";
  if (status === "failed") return "Could not check the CLI.";
  if (!status.installed) return "Not installed or not on PATH.";
  const version = status.version ? `version ${status.version}` : "version unknown";
  const auth = status.authState === "ready"
    ? `signed in${status.authMethod === "account_session" ? " with an account" : status.authMethod === "api_key" ? " with an API key (metered)" : ""}${status.plan ? ` (${status.plan} plan)` : ""}`
    : status.authState === "missing" ? "not signed in" : "sign-in state unknown";
  return `Installed, ${version}, ${auth}.`;
}

function adapterCard(adapter: AdapterId) {
  const info = ADAPTERS[adapter];
  const settings = settingsFor(adapter);
  const billing = settings?.billing ?? (info.cli ? "unknown" : "metered");
  const available = Boolean(settings?.modelId && settings.cloudEnabled && (billing !== "metered" || settings.meteredEnabled) && billing !== "unknown" && (info.cli || settings.credentialConfigured));
  const tag = billing === "unknown" ? "CLOUD · BILLING UNCLASSIFIED · BLOCKED" : "CLOUD · METERED";
  const id = (suffix: string) => `adapter-${adapter}-${suffix}`;
  const credential = info.cli
    ? `<p>${escapeHtml(cliStatusCopy(adapter))}</p><button class="plain" data-adapter-check="${adapter}">Check CLI</button>`
    : `<p>${settings?.credentialConfigured ? "API key configured" : "No API key configured"}</p><form data-adapter-credential="${adapter}"><label class="field">${info.keyLabel}<input id="${id("key")}" type="password" autocomplete="new-password" placeholder="Paste key to replace saved credential"></label><button type="submit" class="primary">Store key securely</button></form><button class="plain" data-adapter-remove-key="${adapter}">Remove ${escapeHtml(info.keyLabel ?? "key")}</button>`;
  const blockedNote = billing === "unknown" ? `<p><b>Blocked:</b> B.O.B. cannot yet confirm whether this CLI bills your subscription allowance or separately metered usage, so it will not run it. The owner must record a billing decision first.</p>` : "";
  return `<article class="panel prefs openrouter adapter" aria-labelledby="${id("title")}"><small>${tag}</small><h2 id="${id("title")}">${info.label}</h2><p>${escapeHtml(info.about)}</p>${blockedNote}<div class="openrouter-columns"><div><form data-adapter-config="${adapter}"><label class="field">Approved model ID<input id="${id("model")}" autocomplete="off" value="${escapeHtml(settings?.modelId ?? "")}" placeholder="${escapeHtml(info.modelPlaceholder)}"></label><label class="check"><input type="checkbox" id="${id("cloud")}" ${settings?.cloudEnabled ? "checked" : ""}> Allow cloud transmission</label><label class="check"><input type="checkbox" id="${id("metered")}" ${settings?.meteredEnabled ? "checked" : ""}> Allow separately billed inference</label><p>Prompts leave this device. Both permissions are required for metered use; set a spending limit in the provider's own console.</p><button type="submit" class="primary">Save model authorization</button></form></div><div><label class="check"><input type="checkbox" data-adapter-chat="${adapter}" ${settings?.chatSessionEnabled ? "checked" : ""} ${!available || adapterBusy ? "disabled" : ""}> Use ${info.label} in Chat for this session</label><p>Off by default every launch. Only one cloud provider can be on at a time.</p>${credential}</div></div></article>`;
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
  document.querySelectorAll<HTMLButtonElement>("[data-adapter-check]").forEach((button) => button.addEventListener("click", () => {
    const adapter = button.dataset.adapterCheck as AdapterId;
    cliStatus[adapter] = "checking";
    render();
    void loadCliRuntimeStatus(adapter)
      .then((status) => { cliStatus[adapter] = status; })
      .catch(() => { cliStatus[adapter] = "failed"; })
      .finally(render);
  }));
  document.querySelector("#adapter-stop")?.addEventListener("click", () => {
    void cancelAdapterGeneration();
  });
}

import { invoke } from "@tauri-apps/api/core";
import type { AccessibilityPreferences, GeminiCredentialStatus, ItemKind, PersistentWorkState, PlanProjection, ReplanResult } from "./model";

const isTauriRuntime = () => "__TAURI_INTERNALS__" in window;
export async function loadChatHistory(): Promise<{ author: "user" | "bob"; text: string }[] | null> {
  if (!isTauriRuntime()) return null;
  return invoke("load_chat_history");
}

export async function clearChatHistory(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("clear_chat_history");
}

export async function appendChatMessage(entry: { author: "user" | "bob"; text: string }): Promise<{ author: "user" | "bob"; text: string }[] | null> {
  if (!isTauriRuntime()) return null;
  return invoke("append_chat_message", { entry });
}

export type OpenRouterAuthorization = { modelId: string | null; cloudEnabled: boolean; meteredEnabled: boolean };
export type OpenRouterSettings = OpenRouterAuthorization & { chatSessionEnabled: boolean };
export type OpenRouterCredentialStatus = { configured: boolean };

export async function loadOpenRouterSettings(): Promise<OpenRouterSettings | null> {
  if (!isTauriRuntime()) return null;
  return invoke<OpenRouterSettings>("openrouter_settings");
}

export async function configureOpenRouterModel(settings: OpenRouterAuthorization): Promise<OpenRouterSettings | null> {
  if (!isTauriRuntime()) return null;
  return invoke<OpenRouterSettings>("configure_openrouter_model", {
    modelId: settings.modelId,
    cloudEnabled: settings.cloudEnabled,
    meteredEnabled: settings.meteredEnabled
  });
}

export async function setOpenRouterChatSession(enabled: boolean): Promise<OpenRouterSettings> {
  if (!isTauriRuntime()) throw new Error("OpenRouter Chat requires the desktop application");
  return invoke<OpenRouterSettings>("set_openrouter_chat_session", { enabled });
}

export async function loadOpenRouterCredentialStatus(): Promise<OpenRouterCredentialStatus> {
  if (!isTauriRuntime()) return { configured: false };
  return invoke<OpenRouterCredentialStatus>("openrouter_credential_status");
}

export async function configureOpenRouterCredential(apiKey: string): Promise<OpenRouterCredentialStatus> {
  if (!isTauriRuntime()) throw new Error("OpenRouter key storage requires the desktop application");
  return invoke<OpenRouterCredentialStatus>("configure_openrouter_credential", { apiKey });
}

export async function removeOpenRouterCredential(): Promise<OpenRouterCredentialStatus> {
  if (!isTauriRuntime()) throw new Error("OpenRouter credential removal requires the desktop application");
  return invoke<OpenRouterCredentialStatus>("remove_openrouter_credential");
}

export async function generateWithOpenRouter(prompt: string): Promise<string> {
  if (!isTauriRuntime()) throw new Error("OpenRouter inference requires the desktop application");
  return invoke<string>("generate_openrouter", { prompt });
}

const browserGeminiStatus: GeminiCredentialStatus = { configured: false, validation: "notConfigured" };

export async function loadPersistentWorkState(): Promise<PersistentWorkState | null> {
  if (!isTauriRuntime()) return null;
  return invoke<PersistentWorkState>("load_work_state");
}

export async function loadAccessibilityPreferences(): Promise<AccessibilityPreferences | null> {
  if (!isTauriRuntime()) return null;
  return invoke<AccessibilityPreferences>("load_accessibility_preferences");
}

export async function setAccessibilityPreferences(preferences: AccessibilityPreferences): Promise<AccessibilityPreferences | null> {
  if (!isTauriRuntime()) return null;
  return invoke<AccessibilityPreferences>("set_accessibility_preferences", { preferences });
}

export async function planRemainingWork(): Promise<PlanProjection | null> {
  if (!isTauriRuntime()) return null;
  return invoke<PlanProjection>("plan_remaining_work");
}

export async function replanRemainingWork(): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("replan_remaining_work");
}

export async function captureItem(title: string): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("capture_item", { title });
}

export async function classifyInboxItem(itemId: string, kind: ItemKind): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("classify_inbox_item", { itemId, kind });
}

export async function startCurrentWork(): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("start_current_work");
}

export async function deferCurrentWork(): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("defer_current_work");
}

export async function resumeDeferredTask(itemId: string): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("resume_deferred_task", { itemId });
}

export async function toggleTaskCompleted(itemId: string): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("toggle_task_completed", { itemId });
}

export async function selectNextTask(itemId: string): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("select_next_task", { itemId });
}

export async function saveCurrentHandoff(): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("save_current_handoff");
}

export async function clearHandoff(): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("clear_handoff");
}

export async function applyNextActionProposal(targetId: string): Promise<ReplanResult | null> {
  if (!isTauriRuntime()) return null;
  return invoke<ReplanResult>("apply_next_action_proposal", { targetId });
}

export async function assistWithBob(input: string): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  return invoke<string>("bob_assist", { input });
}

export async function exportPortableState(): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  return invoke<string>("export_portable_state");
}

export async function loadGeminiCredentialStatus(): Promise<GeminiCredentialStatus> {
  if (!isTauriRuntime()) return browserGeminiStatus;
  return invoke<GeminiCredentialStatus>("gemini_credential_status");
}

export async function configureGeminiCredential(apiKey: string): Promise<GeminiCredentialStatus> {
  if (!isTauriRuntime()) return browserGeminiStatus;
  return invoke<GeminiCredentialStatus>("configure_gemini_credential", { apiKey });
}

export async function removeGeminiCredential(): Promise<GeminiCredentialStatus> {
  if (!isTauriRuntime()) return browserGeminiStatus;
  return invoke<GeminiCredentialStatus>("remove_gemini_credential");
}

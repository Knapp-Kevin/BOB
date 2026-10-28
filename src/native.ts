import { invoke } from "@tauri-apps/api/core";
import type { AccessibilityPreferences, GeminiCredentialStatus, ItemKind, PersistentWorkState, PlanProjection, ReplanResult } from "./model";

const isTauriRuntime = () => "__TAURI_INTERNALS__" in window;
export async function loadChatHistory(): Promise<{ author: "user" | "bob"; text: string }[] | null> {
  if (!isTauriRuntime()) return null;
  return invoke("load_chat_history");
}

export async function appendChatMessage(entry: { author: "user" | "bob"; text: string }): Promise<{ author: "user" | "bob"; text: string }[] | null> {
  if (!isTauriRuntime()) return null;
  return invoke("append_chat_message", { entry });
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

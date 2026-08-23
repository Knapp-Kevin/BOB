import { invoke } from "@tauri-apps/api/core";
import type { AccessibilityPreferences, GeminiCredentialStatus, ItemKind, PersistentWorkState, PlanProjection, ReplanResult } from "./model";

const isTauriRuntime = () => "__TAURI_INTERNALS__" in window;
const browserGeminiStatus: GeminiCredentialStatus = { configured: false, validation: "notConfigured" };

export type ManagedBackupCandidate = {
  id: string;
  modifiedUnixMs: number | null;
  sizeBytes: number;
};

export type RecoveryBackupPreview = {
  candidateId: string;
  validation: "usable" | "unavailable";
  workItemCount: number | null;
  hasActiveItem: boolean | null;
  message: string;
};

export type StartupStatus = {
  mode: "ready" | "recoveryRequired";
  managedBackupCount: number | null;
  managedBackupCandidates: ManagedBackupCandidate[] | null;
};

export async function loadStartupStatus(): Promise<StartupStatus> {
  if (!isTauriRuntime()) return { mode: "ready", managedBackupCount: 0, managedBackupCandidates: [] };
  return invoke<StartupStatus>("startup_status");
}

export async function validateRecoveryBackup(candidateId: string): Promise<RecoveryBackupPreview> {
  if (!isTauriRuntime()) {
    return {
      candidateId,
      validation: "unavailable",
      workItemCount: null,
      hasActiveItem: null,
      message: "Recovery backup validation is available only in the installed B.O.B. application.",
    };
  }
  return invoke<RecoveryBackupPreview>("validate_recovery_backup_command", { candidateId });
}

export async function restartApplication(): Promise<void> {
  if (!isTauriRuntime()) {
    window.location.reload();
    return;
  }
  return invoke<void>("restart_application");
}

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

import "./styles.css";
import { render } from "./controller";
import { applyAccessibilityPreferences, applyPlanProjection, hydratePersistentWorkState, state } from "./model";
import { loadAccessibilityPreferences, loadGeminiCredentialStatus, loadPersistentWorkState, loadStartupStatus, planRemainingWork } from "./native";
import { installAccessibilityPreferencePersistence } from "./preferences";
import { renderStartupRecovery, renderStartupStatusUnavailable } from "./recovery";

async function bootstrap() {
  let startup: Awaited<ReturnType<typeof loadStartupStatus>>;
  try {
    startup = await loadStartupStatus();
  } catch (error) {
    console.error("Failed to confirm B.O.B. startup status", error);
    renderStartupStatusUnavailable();
    return;
  }

  if (startup.mode === "recoveryRequired") {
    renderStartupRecovery(startup);
    return;
  }

  try {
    const [durable, preferences] = await Promise.all([
      loadPersistentWorkState(),
      loadAccessibilityPreferences()
    ]);
    if (durable) hydratePersistentWorkState(durable);
    if (preferences) applyAccessibilityPreferences(preferences);
  } catch (error) {
    console.error("Failed to load durable B.O.B. local state", error);
    renderStartupStatusUnavailable();
    return;
  }

  installAccessibilityPreferencePersistence();
  render();

  void planRemainingWork()
    .then((plan) => {
      if (!plan) return;
      applyPlanProjection(plan);
      render();
    })
    .catch((error) => console.error("Failed to load deterministic B.O.B. plan projection", error));

  void loadGeminiCredentialStatus()
    .then((status) => {
      state.gemini = status;
      state.geminiStaged = status.validation === "ready";
      render();
    })
    .catch((error) => console.error("Failed to refresh Gemini credential status", error));
}

void bootstrap();

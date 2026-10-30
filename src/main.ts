import "./styles.css";
import { refreshOpenRouter } from "./openrouter_ui";
import { render } from "./controller";
import { applyAccessibilityPreferences, applyPlanProjection, hydratePersistentWorkState, state } from "./model";
import { loadAccessibilityPreferences, loadChatHistory, loadGeminiCredentialStatus, loadPersistentWorkState, planRemainingWork } from "./native";
import { installAccessibilityPreferencePersistence } from "./preferences";

async function bootstrap() {
  try {
    const [durable, preferences] = await Promise.all([
      loadPersistentWorkState(),
      loadAccessibilityPreferences()
    ]);
    if (durable) hydratePersistentWorkState(durable);
    if (preferences) applyAccessibilityPreferences(preferences);
  } catch (error) {
    console.error("Failed to load durable B.O.B. local state", error);
  }

  try {
    const history = await loadChatHistory();
    if (history !== null) state.chat = history;
  } catch (error) {
    console.error("Failed to load canonical B.O.B. conversation", error);
  }

  void refreshOpenRouter().finally(render);
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

import { applyAccessibilityPreferences, currentAccessibilityPreferences, showToast, type AccessibilityPreferences } from "./model";
import { setAccessibilityPreferences } from "./native";

let installed = false;
let writeQueue = Promise.resolve();
let durablePreferences: AccessibilityPreferences = { largerText: false, reducedMotion: false };
let latestRequest = 0;

export function installAccessibilityPreferencePersistence() {
  if (installed) return;
  installed = true;
  durablePreferences = currentAccessibilityPreferences();

  document.addEventListener("change", (event) => {
    const input = event.target;
    if (!(input instanceof HTMLInputElement)) return;
    if (input.id !== "larger-text" && input.id !== "reduced-motion") return;

    const requested = currentAccessibilityPreferences();
    const requestId = ++latestRequest;
    writeQueue = writeQueue.then(async () => {
      try {
        const persisted = await setAccessibilityPreferences(requested);
        durablePreferences = persisted ?? requested;
        // An earlier save must never visually undo a newer user selection.
        if (persisted && requestId === latestRequest) {
          applyAccessibilityPreferences(persisted);
          window.dispatchEvent(new Event("bob:render"));
        }
      } catch (error) {
        console.error("Failed to persist B.O.B. accessibility preferences", error);
        // Only the latest failed request may restore the last durable selection.
        // Newer requests remain queued and retain ownership of the visible state.
        if (requestId === latestRequest) {
          applyAccessibilityPreferences(durablePreferences);
          showToast("B.O.B. could not save that accessibility preference. The last durable setting was restored.");
          window.dispatchEvent(new Event("bob:render"));
        }
      }
    });
  });
}

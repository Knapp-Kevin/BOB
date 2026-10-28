import { activeItem, applyPlanProjection, escapeHtml, focusItems, hydratePersistentWorkState, showToast, state, type ItemKind, type ReplanResult, type Route, type SetupStep } from "./model";
import { applyNextActionProposal, assistWithBob, captureItem, classifyInboxItem, clearHandoff, configureGeminiCredential, deferCurrentWork, exportPortableState, removeGeminiCredential, replanRemainingWork, resumeDeferredTask, saveCurrentHandoff, selectNextTask, startCurrentWork, toggleTaskCompleted } from "./native";
import { renderShell } from "./views";

const root = document.querySelector<HTMLDivElement>("#app")!;
if (!root) throw new Error("Missing app root");

const replyFor = (input: string) => {
  const current = activeItem();
  if (!current) return "Nothing is currently planned. Capture the messy version first, and I’ll help turn it into one useful next move.";

  const lower = input.toLowerCase();
  if (lower.includes("handoff") || lower.includes("resume later") || lower.includes("save my place")) return `Handoff: ${current.title}. Current state: ${current.status}. Next: spend five minutes reopening the context and identifying the first concrete change.`;
  if (lower.includes("overwhelm") || lower.includes("too much")) return `Keep only this: ${current.title}. Everything else can wait.`;
  if (lower.includes("wait") || lower.includes("confus") || lower.includes("reorient") || lower.includes("what?")) return `Short version: ${current.title} is the next useful move. You do not need to solve the rest right now.`;
  if (lower.includes("decid") || lower.includes("priorit")) return "I can research facts, but I should not invent your preference. The decision in front of you is: what outcome matters most today?";
  if (lower.includes("break")) return `Start smaller: open the material for “${current.title}” and spend five minutes identifying the first concrete change.`;
  if (lower.includes("organize") || lower.includes("inbox")) return "I found one likely next action in the inbox. I’ll preview the change before touching your work state.";
  return `The next useful move still looks like “${current.title}.” If that is wrong, tell me what changed and I’ll reorient.`;
};

const queueRender = () => render();
window.addEventListener("bob:render", queueRender);

export function render() {
  root.innerHTML = renderShell();
  bindEvents();
}

function applyBrowserFallbackPlan() {
  state.focusIds = [];
  const focus = focusItems();
  state.focusIds = focus.map((item) => item.id);
  state.activeId = focus[0]?.id ?? "";
}

function applyWorkResult(result: ReplanResult) {
  hydratePersistentWorkState(result.workState);
  applyPlanProjection(result.plan);
}

async function runWorkMutation(
  operation: () => Promise<ReplanResult | null>,
  browserFallback: () => void,
  successMessage?: string,
  failureMessage = "B.O.B. could not apply that change. Canonical work state was left unchanged."
) {
  try {
    const result = await operation();
    if (result) {
      applyWorkResult(result);
    } else {
      browserFallback();
      applyBrowserFallbackPlan();
    }
    if (successMessage) showToast(successMessage);
  } catch (error) {
    console.error("B.O.B. work mutation failed", error);
    showToast(failureMessage);
  }
  render();
}

function pushConversation(text: string) {
  state.chat.push({ author: "user", text });
  render();

  void assistWithBob(text)
    .then((response) => {
      state.chat.push({ author: "bob", text: response ?? replyFor(text) });
    })
    .catch((error) => {
      console.error("Native B.O.B. assist failed; using deterministic browser fallback", error);
      state.chat.push({ author: "bob", text: replyFor(text) });
    })
    .finally(render);
}

function downloadPortableExport(contents: string) {
  const blob = new Blob([contents], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "bob-portable-export-v1.json";
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function openSetup(step: SetupStep = state.geminiStaged ? 3 : 1) {
  state.setupStep = step;
  state.setupOpen = true;
  render();
}

function geminiFailureMessage(validation: typeof state.gemini.validation) {
  if (validation === "invalidCredential") return "Google did not accept that key. The existing credential was left unchanged.";
  if (validation === "quotaLimited") return "Gemini is currently rate or quota limited. B.O.B. did not replace the stored credential.";
  if (validation === "unavailable") return "Gemini could not be verified right now. B.O.B. did not replace the stored credential.";
  return "Gemini is not configured.";
}

function bindEvents() {
  document.querySelectorAll<HTMLElement>("[data-route]").forEach((element) => element.addEventListener("click", () => {
    state.route = element.dataset.route as Route;
    render();
  }));

  const toggleReduced = () => { state.reduced = !state.reduced; render(); };
  document.querySelector("#reduce")?.addEventListener("click", toggleReduced);
  document.querySelector("#header-reduce")?.addEventListener("click", toggleReduced);

  document.querySelector("#capture-form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const input = document.querySelector<HTMLInputElement>("#capture-input");
    const title = input?.value.trim();
    if (!title) return;

    void runWorkMutation(
      () => captureItem(title),
      () => state.items.unshift({ id: `capture-${Date.now()}`, kind: "note", title, priority: "normal", status: "inbox" }),
      "Captured to Inbox. Your next action did not change."
    );
  });

  document.querySelector("#start")?.addEventListener("click", () => {
    const current = activeItem();
    if (!current) return;
    void runWorkMutation(
      startCurrentWork,
      () => { current.status = "doing"; },
      "Started. Everything else can wait for now."
    );
  });

  document.querySelector("#defer")?.addEventListener("click", () => {
    const current = activeItem();
    if (!current) return;
    void runWorkMutation(
      deferCurrentWork,
      () => { current.status = "deferred"; },
      "Deferred without losing it."
    );
  });

  document.querySelector("#replan")?.addEventListener("click", () => {
    void replanRemainingWork()
      .then((result) => {
        if (result) applyWorkResult(result);
        else applyBrowserFallbackPlan();
        showToast("Replanned remaining work. Completed and deferred items stayed untouched.");
      })
      .catch((error) => {
        console.error("Failed to replan remaining B.O.B. work", error);
        showToast("B.O.B. could not replan right now. Your current work state was left unchanged.");
      })
      .finally(render);
  });

  document.querySelectorAll<HTMLElement>("[data-resume-deferred]").forEach((element) => element.addEventListener("click", () => {
    const item = state.items.find((candidate) => candidate.id === element.dataset.resumeDeferred);
    if (!item || item.kind !== "task" || item.status !== "deferred") return;
    void runWorkMutation(
      () => resumeDeferredTask(item.id),
      () => { item.status = "planned"; state.activeId = item.id; },
      "Returned to your plan. The task is ready when you are."
    ).then(() => { state.route = "today"; render(); });
  }));

  document.querySelectorAll<HTMLElement>("[data-complete]").forEach((element) => element.addEventListener("click", () => {
    const item = state.items.find((candidate) => candidate.id === element.dataset.complete);
    if (!item || item.kind !== "task") return;
    void runWorkMutation(
      () => toggleTaskCompleted(item.id),
      () => { item.status = item.status === "done" ? "planned" : "done"; }
    );
  }));

  document.querySelectorAll<HTMLElement>("[data-active]").forEach((element) => element.addEventListener("click", () => {
    const item = state.items.find((candidate) => candidate.id === element.dataset.active);
    if (!item || item.kind !== "task") return;
    void runWorkMutation(
      () => selectNextTask(item.id),
      () => {
        if (item.status === "inbox") item.status = "planned";
        state.activeId = item.id;
      },
      "Made that task the next action."
    ).then(() => {
      state.route = "today";
      render();
    });
  }));

  document.querySelectorAll<HTMLElement>("[data-filter]").forEach((element) => element.addEventListener("click", () => {
    state.filter = element.dataset.filter as "all" | ItemKind;
    render();
  }));

  document.querySelectorAll<HTMLSelectElement>("[data-classify]").forEach((select) => select.addEventListener("change", () => {
    const item = state.items.find((candidate) => candidate.id === select.dataset.classify);
    const kind = select.value as ItemKind;
    if (!item || item.status !== "inbox" || item.kind === kind) return;
    void runWorkMutation(
      () => classifyInboxItem(item.id, kind),
      () => { item.kind = kind; },
      `Classified as ${kind}.`
    );
  }));

  document.querySelector("#organize")?.addEventListener("click", () => {
    const target = state.items.find((item) => item.status === "inbox" && item.kind === "task" && item.priority === "high")
      ?? state.items.find((item) => item.status === "inbox" && item.kind === "task");
    if (target) {
      state.pendingProposal = {
        title: "Make one inbox item the next action",
        summary: `Move “${target.title}” into planned work and make it the next action. Leave every other inbox item untouched.`,
        targetId: target.id
      };
      state.route = "chat";
      pushConversation("Help me organize what is in my inbox.");
    }
  });

  document.querySelectorAll<HTMLElement>("[data-prompt]").forEach((element) => element.addEventListener("click", () => {
    pushConversation(element.dataset.prompt!);
  }));

  document.querySelector("#chat-form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const input = document.querySelector<HTMLInputElement>("#chat-input");
    const text = input?.value.trim();
    if (!text) return;
    pushConversation(text);
  });

  document.querySelector("#create-handoff")?.addEventListener("click", () => {
    const current = activeItem();
    if (!current) {
      showToast("There is no current task to save yet.");
      return;
    }
    void runWorkMutation(
      saveCurrentHandoff,
      () => {
        state.handoff = {
          objective: current.title,
          state: current.status === "doing" ? "In progress" : "Ready to start",
          next: `Reopen the context for “${current.title}” and spend five minutes on the first concrete change.`
        };
      },
      "Session handoff saved locally for restart recovery."
    );
    pushConversation("Save my place so I can resume later.");
  });

  document.querySelector("#clear-handoff")?.addEventListener("click", () => {
    void runWorkMutation(
      clearHandoff,
      () => { state.handoff = undefined; },
      "Saved handoff cleared."
    );
  });

  document.querySelector("#apply-proposal")?.addEventListener("click", () => {
    const proposal = state.pendingProposal;
    if (!proposal) return;

    void applyNextActionProposal(proposal.targetId)
      .then((result) => {
        if (result) {
          applyWorkResult(result);
        } else {
          const target = state.items.find((item) => item.id === proposal.targetId);
          if (!target || target.kind !== "task" || ["done", "deferred"].includes(target.status)) {
            showToast("That proposal is stale or no longer safe to apply. Nothing changed.");
            return;
          }
          if (target.status === "inbox") target.status = "planned";
          state.activeId = target.id;
          applyBrowserFallbackPlan();
        }
        state.pendingProposal = undefined;
        state.route = "today";
        showToast("Applied the previewed change. Everything else stayed put.");
      })
      .catch((error) => {
        console.error("B.O.B. rejected a previewed state-change proposal", error);
        showToast("That proposal is stale or no longer safe to apply. Nothing changed.");
      })
      .finally(render);
  });

  document.querySelector("#dismiss-proposal")?.addEventListener("click", () => {
    state.pendingProposal = undefined;
    showToast("Proposal dismissed. Nothing changed.");
    render();
  });

  document.querySelector("#export-data")?.addEventListener("click", () => {
    void exportPortableState()
      .then((contents) => {
        if (!contents) {
          showToast("Portable export is available in the native B.O.B. app.");
          return;
        }
        downloadPortableExport(contents);
        showToast("Portable export created. Protected credentials are not included.");
      })
      .catch((error) => {
        console.error("Failed to create B.O.B. portable export", error);
        showToast("B.O.B. could not create the portable export. Canonical state was not changed.");
      });
  });

  document.querySelector("#setup")?.addEventListener("click", () => openSetup());
  document.querySelector("#replace-key")?.addEventListener("click", () => openSetup(2));
  document.querySelector("#remove-key")?.addEventListener("click", () => {
    if (state.geminiBusy) return;
    state.geminiBusy = true;
    render();
    void removeGeminiCredential()
      .then((status) => {
        state.gemini = status;
        state.geminiStaged = false;
        state.setupOpen = false;
        showToast("Gemini credential removed. Deterministic B.O.B. remains available.");
      })
      .catch((error) => {
        console.error("Failed to remove Gemini credential", error);
        showToast("B.O.B. could not remove the credential from protected storage.");
      })
      .finally(() => {
        state.geminiBusy = false;
        render();
      });
  });

  document.querySelector("#close-setup")?.addEventListener("click", () => { state.setupOpen = false; render(); });
  document.querySelector("#setup-have-key")?.addEventListener("click", () => { state.setupStep = 2; render(); });
  document.querySelector("#setup-back")?.addEventListener("click", () => { state.setupStep = Math.max(1, state.setupStep - 1) as SetupStep; render(); });
  document.querySelector("#stage-key")?.addEventListener("click", () => {
    if (state.geminiBusy) return;
    const input = document.querySelector<HTMLInputElement>("#gemini-key");
    const value = input?.value ?? "";
    if (input) input.value = "";
    if (value.trim().length < 20) {
      showToast("That does not look like a complete key. Nothing was stored.");
      render();
      return;
    }

    state.geminiBusy = true;
    render();
    void configureGeminiCredential(value)
      .then((status) => {
        state.gemini = status;
        state.geminiStaged = status.validation === "ready";
        if (status.validation === "ready") {
          state.setupStep = 3;
          showToast("Gemini key validated and stored in protected OS credential storage.");
        } else {
          showToast(geminiFailureMessage(status.validation));
        }
      })
      .catch((error) => {
        console.error("Gemini credential setup failed", error);
        showToast("B.O.B. could not validate or store that credential. Nothing was replaced.");
      })
      .finally(() => {
        state.geminiBusy = false;
        render();
      });
  });

  document.querySelector("#continue-setup")?.addEventListener("click", () => {
    state.setupOpen = false;
    state.route = "today";
    render();
  });

  document.querySelector<HTMLInputElement>("#larger-text")?.addEventListener("change", (event) => { state.largerText = (event.target as HTMLInputElement).checked; render(); });
  document.querySelector<HTMLInputElement>("#reduced-motion")?.addEventListener("change", (event) => { state.reducedMotion = (event.target as HTMLInputElement).checked; render(); });
}

window.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    state.route = "today";
    render();
    requestAnimationFrame(() => document.querySelector<HTMLInputElement>("#capture-input")?.focus());
  }
  if (event.key === "Escape" && state.setupOpen) {
    state.setupOpen = false;
    render();
  }
});

export const debugState = () => escapeHtml(JSON.stringify(state));
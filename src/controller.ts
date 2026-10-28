import { activeItem, applyPlanProjection, escapeHtml, focusItems, hydratePersistentWorkState, showToast, state, type ItemKind, type ReplanResult, type Route, type SetupStep } from "./model";
import { generateWithOpenRouter, appendChatMessage, clearChatHistory, applyNextActionProposal, assistWithBob, captureItem, classifyInboxItem, clearHandoff, configureGeminiCredential, deferCurrentWork, exportPortableState, removeGeminiCredential, replanRemainingWork, resumeDeferredTask, saveCurrentHandoff, selectNextTask, startCurrentWork, toggleTaskCompleted } from "./native";
import { renderShell } from "./views";
import { saveOpenRouterAuthorization, saveOpenRouterKey, removeOpenRouterKey, openRouterChatEnabled, openRouterFailureMessage, refreshOpenRouter, setOpenRouterChatEnabled } from "./openrouter_ui";

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

// The Chat panel has a fixed height; keep the reader's scroll position across
// full re-renders and follow the conversation only when its content changes.
let renderedChatLength = -1;

export function render() {
  const previousScroll = root.querySelector<HTMLElement>(".messages")?.scrollTop;
  root.innerHTML = renderShell();
  const messages = root.querySelector<HTMLElement>(".messages");
  if (messages) {
    const changed = state.chat.length !== renderedChatLength || previousScroll === undefined;
    messages.scrollTop = changed ? messages.scrollHeight : previousScroll;
  }
  renderedChatLength = state.chat.length;
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

let chatQueue = Promise.resolve();
const MAX_CHAT_BYTES = 8_000;
const chatByteLength = (value: string) => new TextEncoder().encode(value).length;

function pushConversation(text: string) {
  if (!text.trim() || chatByteLength(text) > MAX_CHAT_BYTES) {
    showToast("Message is too long. Keep it under 8,000 UTF-8 bytes.");
    render();
    return;
  }
  // Serialize conversations so responses and durable history remain ordered.
  chatQueue = chatQueue.then(async () => {
    try {
      const saved = await appendChatMessage({ author: "user", text });
      if (saved) state.chat = saved;
      else state.chat.push({ author: "user", text });
    } catch (error) {
      console.error("Could not save B.O.B. message", error);
      showToast("Message could not be saved. Conversation was not sent.");
      render();
      return;
    }
    render();
    let response: string;
    try {
      if (openRouterChatEnabled) {
        if (chatByteLength(text) > 4_000) {
          showToast("OpenRouter accepts up to 4,000 UTF-8 bytes per prompt. Nothing was sent to the provider.");
          response = "That request is too long for the selected cloud model. Shorten it or turn off OpenRouter in Settings.";
        } else {
          try {
            response = await generateWithOpenRouter(text);
          } catch (error) {
            console.error("OpenRouter inference unavailable", error);
            showToast(`${openRouterFailureMessage(error)} B.O.B. is using local guidance for this response.`);
            // Resynchronize with the backend grant, which may have been revoked.
            await refreshOpenRouter();
            response = (await assistWithBob(text)) ?? replyFor(text);
          }
        }
      } else {
        response = (await assistWithBob(text)) ?? replyFor(text);
      }
    } catch (error) {
      console.error("B.O.B. assist failed; using deterministic guidance", error);
      response = replyFor(text);
    }
    try {
      const saved = await appendChatMessage({ author: "bob", text: response });
      if (saved) state.chat = saved;
      else state.chat.push({ author: "bob", text: response });
    } catch (error) {
      console.error("Failed to save B.O.B. response", error);
      state.chat.push({ author: "bob", text: response });
      showToast("B.O.B. replied, but the reply could not be saved.");
    }
    render();
  }).catch((error) => console.error("B.O.B. conversation queue failed", error));
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

  document.querySelector("#clear-chat")?.addEventListener("click", () => {
    if (!window.confirm("Clear the saved B.O.B. conversation? This cannot be undone. Your tasks and handoffs will remain.")) return;
    chatQueue = chatQueue.then(async () => {
      try {
        await clearChatHistory();
        state.chat = [];
        showToast("Conversation cleared. Tasks and handoffs were kept.");
      } catch (error) {
        console.error("B.O.B. could not clear saved conversation", error);
        showToast("Conversation could not be cleared. Your saved history was preserved.");
      }
      render();
    });
  });

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

  document.querySelector("#openrouter-chat")?.addEventListener("change", (event) => {
    const enabled = (event.target as HTMLInputElement).checked;
    if (enabled && !window.confirm("Send your Chat prompts to your approved OpenRouter model? Usage may incur separate charges. This only applies to this app session.")) {
      render();
      return;
    }
    void setOpenRouterChatEnabled(enabled).then((applied) => {
      if (applied) showToast(openRouterChatEnabled ? "OpenRouter Chat enabled for this session." : "Local B.O.B. Chat restored.");
    }).finally(render);
  });
  document.querySelector("#openrouter-config")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const modelId = document.querySelector<HTMLInputElement>("#openrouter-model")?.value.trim() || null;
    const cloudEnabled = Boolean(document.querySelector<HTMLInputElement>("#openrouter-cloud")?.checked);
    const meteredEnabled = Boolean(document.querySelector<HTMLInputElement>("#openrouter-metered")?.checked);
    if (cloudEnabled && meteredEnabled && !window.confirm("Authorize sending Chat prompts to OpenRouter using the selected model? This can incur separately billed API usage.")) return;
    void saveOpenRouterAuthorization({ modelId, cloudEnabled, meteredEnabled }).finally(render);
  });
  document.querySelector("#openrouter-credential")?.addEventListener("submit", (event) => {
    event.preventDefault();
    const input = document.querySelector<HTMLInputElement>("#openrouter-key");
    const key = input?.value ?? "";
    if (input) input.value = "";
    void saveOpenRouterKey(key).finally(render);
  });
  document.querySelector("#openrouter-remove-key")?.addEventListener("click", () => {
    if (!window.confirm("Remove the saved OpenRouter API key?")) return;
    void removeOpenRouterKey().finally(render);
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
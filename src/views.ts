import { activeItem, escapeHtml, focusItems, state, type ItemKind, type Route } from "./model";

const navButton = (route: Route, icon: string, label: string) => `<button class="nav ${state.route === route ? "active" : ""}" data-route="${route}"><span>${icon}</span><strong>${label}</strong></button>`;
const header = (title: string, subtitle: string) => `<header><div><h1>${title}</h1><p>${subtitle}</p></div><button class="ghost" id="header-reduce">${state.reduced ? "Show full view" : "Reduce information"}</button></header>`;
const capture = () => `<form class="panel capture" id="capture-form"><span class="capture-icon">↓</span><label><b>QUICK CAPTURE</b><input id="capture-input" placeholder="Dump a thought, task, or reminder…" autocomplete="off"></label><kbd>Ctrl K</kbd><button class="teal">Capture</button></form>`;

export function renderShell() {
  document.documentElement.classList.toggle("large-text", state.largerText);
  document.documentElement.classList.toggle("reduce-motion", state.reducedMotion);
  return `<div class="shell ${state.reduced ? "reduced" : ""}">
    <aside class="sidebar">
      <div class="brand"><img src="/bob-mascot-transparent.png" alt="B.O.B. mascot"><b>B.O.B.</b><small>Better Organized Brain</small></div>
      <nav>${navButton("today", "⌂", "Today")}${navButton("inbox", "▱", "Inbox")}${navButton("chat", "◫", "Chat")}${navButton("settings", "⚙", "Settings")}</nav>
      <button class="overwhelmed" id="reduce"><span>◉</span><div><b>${state.reduced ? "Show full view" : "Overwhelmed mode"}</b><small>${state.reduced ? "Bring the rest back." : "Reduce noise. One thing at a time."}</small></div></button>
    </aside>
    <main>${renderRoute()}</main>
    ${state.toast ? `<div class="toast" role="status" aria-live="polite">${escapeHtml(state.toast)}</div>` : ""}
    ${state.setupOpen ? renderSetup() : ""}
  </div>`;
}

function renderRoute() {
  if (state.route === "today") return renderToday();
  if (state.route === "inbox") return renderInbox();
  if (state.route === "chat") return renderChat();
  return renderSettings();
}

function renderToday() {
  const focus = focusItems();
  const next = focus[0];
  const hasPlan = Boolean(next);
  const totalEstimated = focus.reduce((total, item) => total + (item.estimate ?? 0), 0);
  const recent = state.items
    .filter((item) => item.kind === "task" && item.id !== next?.id && !["done", "deferred"].includes(item.status))
    .slice(0, 3);
  const planRows = focus.length
    ? focus.map((item, index) => `<div class="time-row"><span>${index + 1}<small>${item.estimate ? `${item.estimate} min` : "Unestimated"}</small></span><div class="block neutral"><strong>${escapeHtml(item.title)}</strong>${item.due ? `<small>${escapeHtml(item.due)}</small>` : ""}</div></div>`).join("")
    : `<div class="empty"><b>No remaining planned work.</b><span>Capture first, then turn an Inbox item into a task when it is actually actionable.</span></div>`;

  return `${header("Today", "Let’s make the next move obvious.")}
  <section class="grid top-grid">
    <article class="panel next"><small>NEXT ACTION</small><div class="next-body"><span class="star">★</span><div>${hasPlan && next ? `<h2>${escapeHtml(next.title)}</h2><p>${next.estimate ? `~${next.estimate} min` : "Small step"} · ${next.priority === "high" ? "High impact" : "Useful progress"}</p><div class="actions"><button class="primary" id="start" ${next.status === "doing" ? "disabled" : ""}>${next.status === "doing" ? "In progress" : "Do it now"}</button><button class="plain" id="defer">◷ Not now</button></div>` : `<h2>Nothing is demanding attention.</h2><p>Your plan is empty. Capture the messy version without having to organize it first.</p>`}</div></div></article>
    <article class="panel focus" data-secondary><div class="panel-title"><small>WHAT MATTERS TODAY</small><span>Up to three</span></div>${focus.length ? focus.map((item, index) => `<button data-complete="${item.id}" aria-pressed="${item.status === "done"}"><i>${index + 1}</i><strong class="${item.status === "done" ? "done" : ""}">${escapeHtml(item.title)}</strong><span>${item.estimate ?? ""}${item.estimate ? " min" : ""}</span><em>${item.status === "done" ? "✓" : ""}</em></button>`).join("") : `<div class="empty"><b>No focus items remain.</b><span>B.O.B. will not manufacture busywork to fill the card.</span></div>`}</article>
  </section>
  ${capture()}
  <section class="grid bottom-grid" data-secondary>
    <article class="panel day"><div class="panel-title"><div><small>REMAINING PLAN</small><h3>${focus.length ? `${focus.length} focus item${focus.length === 1 ? "" : "s"}${totalEstimated ? ` · ~${totalEstimated} min estimated` : ""}` : "No active plan"}</h3></div><button class="ghost small" id="replan" ${focus.length ? "" : "disabled"}>Replan</button></div>${planRows}<p class="fixture-note">This order comes from B.O.B.’s deterministic Rust planner. Clock-time blocks will appear only after fixed commitments and availability have typed scheduling data.</p></article>
    <div class="stack"><article class="panel resume"><small>RESUME WHERE I LEFT OFF</small>${recent.length ? recent.map((item) => `<button data-active="${item.id}"><span>↗</span><strong>${escapeHtml(item.title)}</strong><b>›</b></button>`).join("") : `<div class="empty"><span>No other unfinished task is competing for attention.</span></div>`}</article><article class="encourage"><div><h3>You’ve got this.</h3><p>Progress beats perfection. B.O.B. keeps the rest recoverable.</p></div><img src="/bob-mascot-transparent.png" alt=""></article></div>
  </section>${state.reduced ? `<div class="reduced-note"><b>Reduced-information mode is on.</b> Only the next action and capture are competing for attention.</div>` : ""}`;
}

function kindOptions(current: ItemKind) {
  const options: { value: ItemKind; label: string }[] = [
    { value: "task", label: "Task" },
    { value: "idea", label: "Idea" },
    { value: "note", label: "Note" },
    { value: "reminder", label: "Reminder" }
  ];
  return options.map(({ value, label }) => `<option value="${value}" ${value === current ? "selected" : ""}>${label}</option>`).join("");
}

function renderInbox() {
  const kinds: ("all" | ItemKind)[] = ["all", "task", "idea", "note", "reminder"];
  const visible = state.items.filter((item) => item.status === "inbox" && (state.filter === "all" || item.kind === state.filter));
  const icon = (kind: ItemKind) => kind === "task" ? "✓" : kind === "idea" ? "✦" : kind === "note" ? "≡" : "◷";
  return `${header("Inbox", "Capture first. Decide what it is later.")}${capture()}<section class="inbox-grid"><article class="panel inbox-list"><div class="filters">${kinds.map((kind) => `<button class="${state.filter === kind ? "active" : ""}" data-filter="${kind}">${kind === "all" ? "All items" : `${kind}s`}</button>`).join("")}</div>${visible.length ? visible.map((item) => `<div class="item"><i class="kind ${item.kind}">${icon(item.kind)}</i><div><small>${item.kind.toUpperCase()}</small><strong>${escapeHtml(item.title)}</strong><p>${item.due ? `<span>${escapeHtml(item.due)}</span>` : ""}${item.estimate ? `<span>${item.estimate} min</span>` : ""}<span class="${item.priority}">${item.priority}</span></p><label class="kind-picker"><span>Type</span><select data-classify="${item.id}" aria-label="Classify ${escapeHtml(item.title)}">${kindOptions(item.kind)}</select></label></div>${item.kind === "task" ? `<button title="Make next action" aria-label="Make ${escapeHtml(item.title)} the next action" data-active="${item.id}">›</button>` : `<span aria-hidden="true"></span>`}</div>`).join("") : `<div class="empty"><b>Nothing waiting in this view.</b><span>Capture first. Classification is optional until it helps.</span></div>`}</article><aside data-secondary><article class="panel side"><small>INBOX SNAPSHOT</small><h3>${visible.length} things waiting</h3><p><span>Actionable tasks</span><b>${visible.filter((item) => item.kind === "task").length}</b></p><p><span>Needs a decision</span><b>${visible.filter((item) => item.kind !== "task").length}</b></p><button class="primary" id="organize" ${visible.some((item) => item.kind === "task") ? "" : "disabled"}>Organize with B.O.B.</button></article><article class="panel side"><small>CAPTURE RULE</small><h3>Do not classify it until classification helps.</h3><p>Get it out of your head first. When you are ready, change its type right in the Inbox.</p></article></aside></section>`;
}

function proposalCard() {
  if (!state.pendingProposal) return "";
  return `<article class="proposal-card"><small>PREVIEW BEFORE APPLY</small><h3>${escapeHtml(state.pendingProposal.title)}</h3><p>${escapeHtml(state.pendingProposal.summary)}</p><div class="actions"><button class="primary" id="apply-proposal">Apply this change</button><button class="plain" id="dismiss-proposal">Keep things as they are</button></div></article>`;
}

function handoffCard() {
  if (!state.handoff) return "";
  return `<article class="handoff-card"><div class="panel-title"><small>SAVED HANDOFF</small><button class="ghost small" id="clear-handoff">Clear</button></div><dl><div><dt>Objective</dt><dd>${escapeHtml(state.handoff.objective)}</dd></div><div><dt>State</dt><dd>${escapeHtml(state.handoff.state)}</dd></div><div><dt>Next</dt><dd>${escapeHtml(state.handoff.next)}</dd></div></dl><p>Stored in B.O.B.’s local canonical state so the objective and next move can survive a full restart.</p></article>`;
}

function renderChat() {
  const current = activeItem();
  return `${header("B.O.B. Chat", "One assistant, with the current work close at hand.")}<section class="chat-grid"><div class="conversation"><div class="messages">${state.chat.map((message) => `<div class="message ${message.author}">${message.author === "bob" ? `<img src="/bob-mascot-transparent.png" alt="B.O.B.">` : ""}<p>${escapeHtml(message.text)}</p></div>`).join("")}${proposalCard()}${handoffCard()}</div><form class="composer" id="chat-form"><input id="chat-input" placeholder="Tell B.O.B. what is messy, blocked, or unclear…" autocomplete="off"><button aria-label="Send">↑</button></form><p class="fixture-note">The native app routes deterministic Assist through B.O.B.’s local Rust core. Browser-only preview mode uses a non-authoritative fallback. Optional provider inference remains separate and policy-gated; B.O.B. stays useful without it.</p></div><aside class="panel frontier" data-secondary><small>CURRENT CONTEXT</small><h2>${current ? escapeHtml(current.title) : "Nothing selected yet"}</h2><p>${current ? "B.O.B. can reduce, break down, reorient, or help decide while remaining one assistant." : "Capture the messy version first. B.O.B. can help once there is something real to work with."}</p><div class="frontier-actions"><button data-prompt="I’m overwhelmed. Show me one thing."><b>Reduce this</b><span>Hide everything that is not immediately useful.</span></button><button data-prompt="Break this into the smallest useful first step."><b>Break it down</b><span>Find a smaller starting move.</span></button><button data-prompt="Wait, what? Reorient me in plain language."><b>Reorient me</b><span>Restate the plan without adding new complexity.</span></button><button data-prompt="Help me decide what matters most here."><b>Help me decide</b><span>Separate facts from the decision I own.</span></button><button id="create-handoff" ${current ? "" : "disabled"}><b>Save my place</b><span>Store a compact local handoff for later resumption.</span></button></div></aside></section>`;
}

function geminiStatusCopy() {
  if (state.gemini.validation === "ready") return "Optional Gemini API credential validated and stored in protected OS credential storage. Using the adapter remains separately policy-gated.";
  if (state.gemini.configured && state.gemini.validation === "invalidCredential") return "A protected Gemini API credential exists, but the provider rejected it. Replace or remove it; B.O.B. remains deterministic meanwhile.";
  if (state.gemini.configured && state.gemini.validation === "quotaLimited") return "A protected Gemini API credential exists, but the provider is currently quota or rate limited. B.O.B. remains usable without inference.";
  if (state.gemini.configured && state.gemini.validation === "unavailable") return "A protected Gemini API credential exists, but the provider could not be verified right now. B.O.B. remains usable without inference.";
  return "Optional advanced provider. Connect a Gemini Developer API key only if you want to configure this adapter; B.O.B. remains useful without it.";
}

function renderSettings() {
  return `${header("Settings", "Keep configuration out of the way until it matters.")}<section class="settings-grid"><article class="panel gemini"><div class="gemini-icon">✦</div><div><small>CONNECTED INTELLIGENCE</small><h2>Gemini API (advanced)</h2><p>${geminiStatusCopy()}</p></div><button class="primary" id="setup" ${state.geminiBusy ? "disabled" : ""}>${state.gemini.configured ? "Review setup" : "Connect Gemini"}</button>${state.gemini.configured ? `<button class="plain" id="replace-key" ${state.geminiBusy ? "disabled" : ""}>Replace key</button><button class="plain" id="remove-key" ${state.geminiBusy ? "disabled" : ""}>Remove key</button>` : ""}</article><article class="panel prefs"><small>ACCESSIBILITY</small><h2>Make B.O.B. easier to read</h2><label><span><b>Larger interface text</b><small>Increase the base UI scale.</small></span><input id="larger-text" type="checkbox" ${state.largerText ? "checked" : ""}></label><label><span><b>Reduced motion</b><small>Suppress nonessential transitions.</small></span><input id="reduced-motion" type="checkbox" ${state.reducedMotion ? "checked" : ""}></label></article><article class="panel prefs"><small>DATA OWNERSHIP</small><h2>Take your B.O.B. state with you</h2><p>Export a versioned JSON package of ordinary user-owned state in stable product terms. Protected provider credentials are never included.</p><button class="primary" id="export-data">Export my data</button></article></section>`;
}

function setupProgress() {
  return `<div class="setup-progress" aria-label="Gemini setup progress">${[1,2,3].map((step) => `<span class="${state.setupStep === step ? "active" : state.setupStep > step ? "done" : ""}"><b>${step}</b><small>${step === 1 ? "Get key" : step === 2 ? "Bring it back" : "Ready"}</small></span>`).join("")}</div>`;
}

function renderSetupStep() {
  if (state.setupStep === 1) return `<article class="setup-step"><div class="step-number">1</div><small>GET YOUR KEY</small><h3>Go straight to Google AI Studio</h3><p>This advanced provider path uses a Gemini Developer API key. It is not required to use B.O.B., and B.O.B. remains deterministic without it.</p><div class="setup-callout"><b>What this step does</b><span>Google handles sign-in and key creation. B.O.B. only receives the key you intentionally paste back here.</span></div><div class="setup-actions"><a class="primary link" href="https://aistudio.google.com/app/apikey" target="_blank" rel="noreferrer">Get Gemini API key ↗</a><button class="plain" id="setup-have-key">I have my key</button></div></article>`;
  if (state.setupStep === 2) return `<article class="setup-step"><div class="step-number">2</div><small>BRING IT BACK</small><h3>Paste it once</h3><p>B.O.B. sends the key directly to the Rust core, validates it against Gemini, and stores it only if validation succeeds. A failed replacement leaves any existing protected credential unchanged.</p><label for="gemini-key">Gemini API key</label><input id="gemini-key" type="password" placeholder="Paste key here" autocomplete="off" spellcheck="false"><div class="setup-actions"><button class="ghost" id="setup-back" ${state.geminiBusy ? "disabled" : ""}>Back</button><button class="teal" id="stage-key" ${state.geminiBusy ? "disabled" : ""}>${state.geminiBusy ? "Checking…" : "Validate and store"}</button></div><p class="tiny">The raw key is never written to ordinary B.O.B. state and is never returned to the frontend.</p></article>`;
  return `<article class="setup-step complete-step"><div class="step-number success-number">✓</div><small>CONNECTED</small><h3>Gemini API credential is ready</h3><p>The credential was validated and stored through the OS credential store. This setup flow does not send tasks, plans, chat history, or other user work context to Gemini.</p><div class="connected-state"><i class="dot ok"></i><div><b>Gemini API connected</b><span>Protected OS credential storage</span></div></div><div class="setup-actions"><button class="ghost" id="setup-back">Replace key</button><button class="primary" id="continue-setup">Continue to Today</button></div></article>`;
}

function renderSetup() {
  return `<div class="modal" role="dialog" aria-modal="true" aria-labelledby="setup-title"><section class="setup-modal"><button class="close" id="close-setup" aria-label="Close">×</button><div class="setup-head"><img src="/bob-mascot-transparent.png" alt=""><div><small>ADVANCED PROVIDER SETUP</small><h2 id="setup-title">Connect the optional Gemini API adapter</h2><p>One clear step at a time. Provider plumbing stays out of your daily workflow.</p></div></div>${setupProgress()}${renderSetupStep()}<div class="privacy"><b>Provider-use and privacy boundary</b><br>Connecting a key only validates the credential and sends no user work context. Context-bearing inference is separately gated by professional/business-use acknowledgement, exact-payload non-sensitive confirmation, and current Free-Tier project confirmation before any provider request.</div></section></div>`;
}
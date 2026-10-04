import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

// Hosted Vite/browser evidence only. Native Windows acceptance remains governed by docs/VALIDATION.md.
const baseUrl = process.env.BOB_RENDER_BASE_URL ?? "http://127.0.0.1:4173";
const debugPort = Number(process.env.BOB_CHROME_DEBUG_PORT ?? "9222");
const outputDir = process.env.BOB_RENDER_OUTPUT_DIR ?? "rendered-artifacts";
const debugBase = `http://127.0.0.1:${debugPort}`;
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

await mkdir(outputDir, { recursive: true });

async function waitForJson(url, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) return response.json();
      lastError = new Error(`${url} returned ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await sleep(100);
  }
  throw lastError ?? new Error(`Timed out waiting for ${url}`);
}

const targets = await waitForJson(`${debugBase}/json/list`);
const target = targets.find((candidate) => candidate.type === "page");
if (!target?.webSocketDebuggerUrl) throw new Error("Chrome did not expose a page target");

const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});

let commandId = 0;
const pending = new Map();

socket.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  if (!message.id) return;
  const waiter = pending.get(message.id);
  if (!waiter) return;
  pending.delete(message.id);
  if (message.error) waiter.reject(new Error(`${waiter.method}: ${message.error.message}`));
  else waiter.resolve(message.result ?? {});
});

function command(method, params = {}) {
  const id = ++commandId;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject, method });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const result = await command("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
    userGesture: true
  });
  if (result.exceptionDetails) {
    const text = result.exceptionDetails.exception?.description ?? result.exceptionDetails.text ?? "Runtime evaluation failed";
    throw new Error(text);
  }
  return result.result?.value;
}

async function waitFor(expression, description, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await evaluate(expression)) return;
    await sleep(75);
  }
  throw new Error(`Timed out waiting for ${description}`);
}

async function setViewport(width, height) {
  await command("Emulation.setDeviceMetricsOverride", {
    width,
    height,
    deviceScaleFactor: 1,
    mobile: false,
    screenWidth: width,
    screenHeight: height
  });
}

async function navigate(url = baseUrl) {
  await command("Page.navigate", { url });
  await waitFor(
    `document.readyState === "complete" && Boolean(document.querySelector("#app")?.children.length)`,
    "B.O.B. application render"
  );
  await sleep(100);
}

async function click(selector) {
  const clicked = await evaluate(`(() => {
    const element = document.querySelector(${JSON.stringify(selector)});
    if (!element) return false;
    element.click();
    return true;
  })()`);
  if (!clicked) throw new Error(`Missing rendered control: ${selector}`);
  await sleep(80);
}

async function capture(name) {
  const result = await command("Page.captureScreenshot", {
    format: "png",
    fromSurface: true,
    captureBeyondViewport: false
  });
  await writeFile(join(outputDir, `${name}.png`), Buffer.from(result.data, "base64"));
}

async function audit(label, expectedRoute) {
  const result = await evaluate(`(() => {
    const visible = (element) => {
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    };
    const focusables = Array.from(document.querySelectorAll('a[href], button, input, select, textarea, [tabindex]')).filter(visible);
    const clippedFocusables = focusables
      .map((element) => {
        const rect = element.getBoundingClientRect();
        return {
          tag: element.tagName.toLowerCase(),
          id: element.id || null,
          text: (element.textContent || element.getAttribute('aria-label') || '').trim().slice(0, 80),
          left: Math.round(rect.left),
          right: Math.round(rect.right),
          top: Math.round(rect.top),
          bottom: Math.round(rect.bottom)
        };
      })
      .filter((rect) => rect.left < -1 || rect.right > window.innerWidth + 1)
      .slice(0, 12);
    const activeRoute = document.querySelector('[data-route][aria-current="page"]')?.getAttribute('data-route')
      ?? document.querySelector('[data-route].active')?.getAttribute('data-route')
      ?? null;
    return {
      title: document.title,
      activeRoute,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      documentSize: { width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight },
      horizontalOverflow: document.documentElement.scrollWidth > window.innerWidth + 1,
      clippedFocusables,
      focusableCount: focusables.length,
      dialogVisible: Boolean(Array.from(document.querySelectorAll('[role="dialog"]')).find(visible))
    };
  })()`);

  result.label = label;
  result.expectedRoute = expectedRoute;
  result.routeMatches = expectedRoute ? result.activeRoute === expectedRoute : true;
  return result;
}

async function exerciseKeyboardFocus() {
  await evaluate(`(() => {
    document.body.setAttribute("tabindex", "-1");
    document.body.focus();
    return document.activeElement === document.body;
  })()`);
  const key = { key: "Tab", code: "Tab", windowsVirtualKeyCode: 9, nativeVirtualKeyCode: 9 };
  await command("Input.dispatchKeyEvent", { type: "keyDown", ...key });
  await command("Input.dispatchKeyEvent", { type: "keyUp", ...key });
  await sleep(75);
  return evaluate(`(() => {
    const element = document.activeElement;
    if (!element || element === document.body) return { progressed: false };
    const style = getComputedStyle(element);
    return {
      progressed: true,
      tag: element.tagName.toLowerCase(),
      id: element.id || null,
      text: (element.textContent || element.getAttribute('aria-label') || '').trim().slice(0, 80),
      outlineStyle: style.outlineStyle,
      outlineWidth: style.outlineWidth,
      boxShadow: style.boxShadow
    };
  })()`);
}

await command("Page.enable");
await command("Runtime.enable");

const report = {
  generatedAt: new Date().toISOString(),
  baseUrl,
  evidenceClass: "hosted-vite-rendered",
  nativeWindowsEvidence: false,
  audits: [],
  keyboard: [],
  recovery: []
};
const failures = [];
const viewports = [
  { name: "normal", width: 1400, height: 900 },
  { name: "minimum", width: 900, height: 640 }
];
const routes = ["today", "inbox", "chat", "settings"];

for (const viewport of viewports) {
  await setViewport(viewport.width, viewport.height);
  await navigate();

  for (const route of routes) {
    await click(`[data-route="${route}"]`);
    const label = `${viewport.name}-${route}`;
    const auditResult = await audit(label, route);
    report.audits.push(auditResult);
    await capture(label);
    if (!auditResult.routeMatches) failures.push(`${label}: route did not become active`);
    if (auditResult.horizontalOverflow) failures.push(`${label}: document has horizontal overflow`);
    if (auditResult.clippedFocusables.length) failures.push(`${label}: focusable controls are clipped horizontally`);
  }

  await click('[data-route="today"]');
  await click("#reduce");
  const reducedLabel = `${viewport.name}-today-reduced`;
  const reducedAudit = await audit(reducedLabel, "today");
  report.audits.push(reducedAudit);
  await capture(reducedLabel);
  if (reducedAudit.horizontalOverflow || reducedAudit.clippedFocusables.length) failures.push(`${reducedLabel}: reduced-information layout overflow/clipping`);

  await click("#reduce");
  await click('[data-route="settings"]');
  const focusResult = await exerciseKeyboardFocus();
  focusResult.label = `${viewport.name}-settings-keyboard`;
  report.keyboard.push(focusResult);
  if (!focusResult.progressed) failures.push(`${viewport.name}: keyboard Tab did not reach an interactive control`);

  await click("#larger-text");
  await click("#reduced-motion");
  const accessibleLabel = `${viewport.name}-settings-larger-text-reduced-motion`;
  const accessibleAudit = await audit(accessibleLabel, "settings");
  report.audits.push(accessibleAudit);
  await capture(accessibleLabel);
  if (accessibleAudit.horizontalOverflow || accessibleAudit.clippedFocusables.length) failures.push(`${accessibleLabel}: accessibility-preference layout overflow/clipping`);

  await click("#setup");
  const setupLabel = `${viewport.name}-settings-gemini-setup`;
  const setupAudit = await audit(setupLabel, "settings");
  report.audits.push(setupAudit);
  await capture(setupLabel);
  if (!setupAudit.dialogVisible) failures.push(`${setupLabel}: expected setup dialog is not visible`);
  if (setupAudit.horizontalOverflow || setupAudit.clippedFocusables.length) failures.push(`${setupLabel}: setup dialog overflow/clipping`);

  await navigate(`${baseUrl}?bobRecoveryFixture=1`);
  await waitFor(`Boolean(document.querySelector("#startup-recovery-title"))`, "startup recovery fixture");
  const recoveryLabel = `${viewport.name}-startup-recovery`;
  const recoveryAudit = await audit(recoveryLabel, null);
  const recoveryState = await evaluate(`(() => ({
    titleFocused: document.activeElement?.id === "startup-recovery-title",
    candidateButtons: document.querySelectorAll("[data-validate-backup]").length,
    boundedDisclosure: document.querySelector(".startup-recovery__candidates > p")?.textContent?.includes("Showing the 8 newest.") ?? false
  }))()`);
  report.audits.push(recoveryAudit);
  report.recovery.push({ label: recoveryLabel, ...recoveryState });
  await capture(recoveryLabel);
  if (recoveryAudit.horizontalOverflow) failures.push(`${recoveryLabel}: recovery layout has horizontal overflow`);
  if (recoveryAudit.clippedFocusables.length) failures.push(`${recoveryLabel}: recovery controls are clipped horizontally`);
  if (!recoveryState.titleFocused) failures.push(`${recoveryLabel}: recovery heading did not receive startup focus`);
  if (recoveryState.candidateButtons !== 8) failures.push(`${recoveryLabel}: expected eight bounded backup controls`);
  if (!recoveryState.boundedDisclosure) failures.push(`${recoveryLabel}: bounded backup disclosure is missing`);

  await click('[data-validate-backup="0"]');
  await waitFor(`document.querySelector('[data-backup-result="0"]')?.textContent?.includes("7 work items")`, "recovery preview result");
  const previewLabel = `${viewport.name}-startup-recovery-preview`;
  const previewAudit = await audit(previewLabel, null);
  report.audits.push(previewAudit);
  await capture(previewLabel);
  if (previewAudit.horizontalOverflow || previewAudit.clippedFocusables.length) failures.push(`${previewLabel}: recovery preview layout overflow/clipping`);

  await click('[data-validate-backup="1"]');
  await waitFor(`document.querySelector('[data-backup-result="1"]')?.textContent?.includes("could not validate this backup")`, "unavailable recovery preview result");
  const unavailableLabel = `${viewport.name}-startup-recovery-preview-unavailable`;
  const unavailableAudit = await audit(unavailableLabel, null);
  const unavailableState = await evaluate(`(() => ({
    retryEnabled: !document.querySelector('[data-validate-backup="1"]')?.disabled,
    retryCopyRestored: document.querySelector('[data-validate-backup="1"]')?.textContent?.trim() === "Check backup"
  }))()`);
  report.audits.push(unavailableAudit);
  report.recovery.push({ label: unavailableLabel, ...unavailableState });
  await capture(unavailableLabel);
  if (unavailableAudit.horizontalOverflow || unavailableAudit.clippedFocusables.length) failures.push(`${unavailableLabel}: unavailable recovery preview layout overflow/clipping`);
  if (!unavailableState.retryEnabled) failures.push(`${unavailableLabel}: unavailable backup check did not re-enable its control`);
  if (!unavailableState.retryCopyRestored) failures.push(`${unavailableLabel}: unavailable backup check did not restore retry copy`);
}

await writeFile(join(outputDir, "audit.json"), `${JSON.stringify(report, null, 2)}\n`);

socket.close();

if (failures.length) {
  console.error("Rendered UI diagnostic failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exitCode = 1;
} else {
  console.log(`Rendered UI diagnostic passed with ${report.audits.length} audited states, ${report.keyboard.length} keyboard checks, and ${report.recovery.length} recovery checks.`);
}

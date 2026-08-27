# B.O.B. Roadmap

**Status:** Accepted  
**Product stage:** **Alpha Product**

The roadmap is capability-oriented rather than calendar-oriented. A release advances when its acceptance criteria are satisfied, not because a date arrived and demanded tribute.

B.O.B. is now classified as an **Alpha Product**. The first runnable alpha waypoint has been crossed at the product-stage level; the remaining Windows recovery, installer, and convergence work is alpha release qualification. Current Wayfinder maps and accepted repository authority determine the next build frontier.

## Landed foundation

The revived desktop architecture and core planning product are now implemented on `master`:

- Tauri 2 desktop shell with Rust privileged core;
- framework-free TypeScript + Vite frontend;
- Windows-first packaging/validation authority;
- Rust-owned SQLite canonical state, migrations, recovery, backup/restore, and portable non-secret export;
- Today, Inbox, quick capture, item lifecycle, focus and deterministic planning/replanning;
- durable handoff/restart continuity;
- B.O.B. Assist core and preview-before-apply proposal authority;
- accessibility preferences and visible keyboard-focus corrections;
- OS-backed secret storage;
- advanced optional Gemini API credential/context capability behind fail-closed cost/privacy/provider-use policy;
- accepted provider-independent runtime contract and Rust-owned runtime policy;
- first non-user-facing fail-closed Ollama tracer behind that contract;
- converged normal-mode Settings, Today, Inbox, and Chat presentation with hosted rendered evidence at normal/minimum sizes and relevant accessibility states;
- dependency-free hosted rendered-UI diagnostics that complement, but do not replace, native Windows acceptance.

These are current product surfaces, not future milestones.

## Recently completed: calm primary workflow

**Goal:** Make B.O.B. feel like one calm prioritized assistant at normal information density.

Wayfinder #86 is complete. Its bounded implementation owners landed as:

- Settings/provider positioning: PR #89;
- Today hierarchy/density: #87 / PR #93;
- Inbox and Chat density/conversation-workspace refinement: #88 / PR #106.

Reduced-information mode remains additive simplification rather than a rescue mechanism for a cluttered default experience. Future presentation defects should receive new bounded owners rather than keeping completed Wayfinder work permanently open.

## Current priority: alpha qualification

**Goal:** Turn the current Alpha Product into repeatable, native, reviewable release evidence and remove the remaining qualification false greens.

The immediate frontier is intentionally narrow:

1. **Startup recovery:** #85 / draft PR #103 owns visible fail-closed startup recovery. Its hosted frontend, Windows Rust, and normal rendered regression gates are green on its current exact head, but it remains draft until the real Windows corrupt-state, backup-preview, process-restart, and recovery-surface accessibility behavior is exercised.
2. **Windows package acceptance:** #84 is open and owns the native Windows 11 x64 NSIS install/launch/relaunch/icon/uninstall/retained-user-data evidence. Hosted package creation and a written runbook are not equivalent to native install acceptance.
3. **Alpha convergence audit:** after those owners are truthfully dispositioned, re-evaluate the alpha qualification criteria and remaining native credential/provider/restart evidence before declaring the integrated build release-qualified.

Supporting readiness work includes:

- locked/reproducible npm and Cargo dependency state;
- frontend production build/type validation;
- Rust fmt/clippy/tests and Tauri build on capable environments;
- Windows persistence/restart/recovery exercises;
- Windows Credential Manager behavior;
- rendered desktop/accessibility regression at supported sizes;
- provider-boundary validation where live inference is exercised;
- exact-head evidence after merges or rebases that invalidate prior validation.

Small CI remains a safety net. Native/recovery/provider/package evidence is not replaced by green hosted checks.

## Current priority after qualification: provider-independent inference

**Goal:** Make inference replaceable without making provider plumbing B.O.B.'s product identity.

Governed by Wayfinder #79.

Current accepted direction:

- B.O.B. remains the single user-facing agent and owns state, continuity, deterministic services, routing, authority, privacy, and billing policy;
- Gemini Developer API remains a supported advanced optional adapter and first-alpha proof point;
- #80 established that ordinary Gemini consumer entitlement is not currently exposed through a direct embeddable Google OAuth inference API for B.O.B.; Antigravity remains only an optional external-runtime candidate;
- #81 established the local-runtime direction: a Rust-owned `LocalRuntimeAdapter`, initially favoring an in-process Rust engine with GGUF support while retaining optional Ollama/LM Studio compatibility;
- RFC-0002 plus the landed runtime-policy foundation normalize only the runtime identity, readiness, auth, billing, locality/privacy, capability, lifecycle, resource, and failure fields B.O.B. actually needs;
- the landed Ollama tracer proves one adapter behind that seam without making Ollama mandatory;
- preserve deterministic useful operation with no inference configured;
- preserve no-surprise billing and no silent provider/model fallback.

The next runtime step should promote one already-authorized path with real native evidence after the active recovery/package qualification frontier is reconciled. Do not respond to a cleaner queue by proliferating adapters or placeholder provider controls.

## Next capability candidates

These are directional and require accepted product/architecture authority before implementation where not already governed.

### Additional supported inference paths

- officially supported account-backed runtimes whose entitlement and integration contract are established;
- B.O.B.-native local inference through the accepted `LocalRuntimeAdapter` direction;
- optional compatibility with user-owned local runtime installations such as Ollama or LM Studio where the fail-closed locality/billing contract can be proven;
- runtime switching while preserving one B.O.B. identity and continuity.

### Portable B.O.B. capabilities and harness adapters

ADR-0006 accepts the direction that harness-neutral B.O.B. behavior should be portable behind B.O.B.-owned contracts while the Tauri desktop application remains a first-party host rather than the only possible host.

Design-intent issue #109 is complete because the direction and sequencing are durably recorded. **Proposed RFC-0004** now owns the unresolved implementation contract.

The first proving target is a **B.O.B.-owned DeepSeek Harness adapter**, implemented entirely in this repository. The useful DeepSeek lesson is capability/service-definition separation and explicit lifecycle, not a wholesale dependency on Cordis or DeepSeek's current API shape. RFC-0004 must settle the exact portable slice, cross-language bridge, protocol versioning, supported DeepSeek version range, and validation matrix before substantial portability code lands.

QOR Agent and GG-CORE are complementary integration targets/reference boundaries:

- a QOR Agent adapter may translate portable B.O.B. behavior onto QOR's public model/tool/interceptor/session/observer/external-runtime seams without changing QOR Agent itself;
- GG-CORE may satisfy B.O.B.'s inference/runtime port through its supported Rust or authenticated local IPC surface while B.O.B. retains state, policy, proposal, and tool authority;
- Cloudflare remains one possible QOR host/proving implementation, not the definition of the QOR Agent harness.

Sequence this work **after alpha stabilization and convergence**, beginning with RFC-0004 P0 boundary classification and then extracting only the minimum portable vertical slice required by a real second host. This is first-party composition, not authorization for a broad plugin marketplace.

### Bounded delegated work

Potential future scope:

- explicit Assist versus Delegate authority;
- bounded workspace/capability grants;
- execution lifecycle/cancellation where supported;
- result/evidence capture without granting ordinary chat broad tool authority.

### Broader personal-work continuity

Potential later work must earn a PRD or governing Wayfinder destination, including:

- calendar integration;
- recurring routines;
- notification scheduling;
- richer continuity summaries;
- selective document context;
- mobile companion or additional surfaces;
- encrypted/shared continuity where an explicit sync/trust model is accepted.

## Explicit non-goals without later authority

- peer-agent or multi-agent swarm UX;
- generalized RAG/knowledge-center infrastructure;
- cognitive profiling or diagnostic behavior;
- broad plugin marketplaces;
- cloud sync or multi-user state by implication;
- mandatory dependency on Google, Anthropic, OpenAI, Ollama, LM Studio, DeepSeek Harness, QOR Agent, GG-CORE, or another provider/runtime/harness client;
- silent metered fallback;
- decorative dashboard expansion that increases cognitive load without improving the primary workflow.

The project should remain smaller than the problem space around it.

# B.O.B. Roadmap

**Status:** Accepted

The roadmap is capability-oriented rather than calendar-oriented. A release advances when its acceptance criteria are satisfied, not because a date arrived and demanded tribute.

The first runnable alpha is a waypoint, not the end of development. Current Wayfinder maps and accepted repository authority determine the next build frontier.

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
- first non-user-facing fail-closed Ollama tracer behind that contract.

These are current product surfaces, not future milestones.

## Current priority: calm primary workflow

**Goal:** Make B.O.B. feel like one calm prioritized assistant at normal information density.

Governed by Wayfinder #86.

Current bounded slices:

- Settings/provider-positioning cleanup under #82;
- Today hierarchy/density convergence under #87;
- Inbox and Chat density/empty-state refinement under #88;
- rendered desktop/accessibility evidence after each material UI slice.

Reduced-information mode is additive simplification, not a rescue mechanism for a cluttered default experience.

## Current priority: provider-independent inference

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

The near-term build priority is evidence, not adapter proliferation: finish active native/rendered recovery and UX acceptance plus Windows installer smoke, then promote one already-authorized inference path with real native evidence. Do not invent unsupported account entitlement or add placeholder provider controls merely to make Settings look future-ready.

## Current priority: executable readiness

**Goal:** Turn landed source into repeatable, native, reviewable product evidence.

Includes:

- locked/reproducible npm and Cargo dependency state;
- frontend production build/type validation;
- Rust fmt/clippy/tests and Tauri build on capable environments;
- Windows persistence/restart/recovery exercises;
- Windows Credential Manager behavior;
- rendered desktop/accessibility regression at supported sizes;
- provider-boundary validation where live inference is exercised;
- NSIS package install/launch/uninstall smoke;
- exact-head evidence after merges or rebases that invalidate prior validation.

Small CI remains a safety net. Native/rendered/recovery/provider evidence is not replaced by green hosted checks.

## Next capability candidates

These are directional and require accepted product/architecture authority before implementation where not already governed:

### Additional supported inference paths

- officially supported account-backed runtimes whose entitlement and integration contract are established;
- B.O.B.-native local inference through the accepted `LocalRuntimeAdapter` direction;
- optional compatibility with user-owned local runtime installations such as Ollama or LM Studio where the fail-closed locality/billing contract can be proven;
- runtime switching while preserving one B.O.B. identity and continuity.

### Portable B.O.B. capabilities and harness adapters

ADR-0006 accepts the direction that harness-neutral B.O.B. behavior should be portable behind B.O.B.-owned contracts while the Tauri desktop application remains a first-party host rather than the only possible host.

The first proving target is a **B.O.B.-owned DeepSeek Harness adapter**, implemented entirely in this repository. The useful DeepSeek lesson is capability/service-definition separation and explicit lifecycle, not a wholesale dependency on Cordis or DeepSeek's current API shape. RFC-0004 remains Proposed while the exact portable slice, cross-language bridge, protocol versioning, and supported DeepSeek version range are proven.

QOR Agent and GG-CORE are complementary integration targets/reference boundaries:

- a QOR Agent adapter may translate portable B.O.B. behavior onto QOR's public model/tool/interceptor/session/observer/external-runtime seams without changing QOR Agent itself;
- GG-CORE may satisfy B.O.B.'s inference/runtime port through its supported Rust or authenticated local IPC surface while B.O.B. retains state, policy, proposal, and tool authority;
- Cloudflare remains one possible QOR host/proving implementation, not the definition of the QOR Agent harness.

Sequence this work **after current alpha stabilization**, then extract only the minimum portable vertical slice required by the real second host. This is first-party composition, not authorization for a broad plugin marketplace.

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

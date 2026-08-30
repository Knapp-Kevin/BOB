# Current Implementation Plan

**Status:** Active sequencing context  
**Product stage:** **Alpha Product**  
**Objective:** Qualify the landed desktop Alpha Product through truthful native Windows recovery, packaging, and convergence evidence while allowing the explicitly authorized, disjoint DeepSeek host tracer to prove B.O.B.'s portable-host boundary without weakening those gates.

## Strategy

The revival architecture is no longer hypothetical. `master` contains the Tauri 2 + Rust application foundation, Rust-owned SQLite state, Today/Inbox/Chat/Settings, deterministic planning, B.O.B. Assist/proposal validation, protected credentials, recovery/export foundations, accessibility preferences, provider-independent runtime policy, an advanced optional Gemini API adapter, a bounded Ollama tracer, locked Windows packaging, and hosted rendered diagnostics.

B.O.B. is classified as an **Alpha Product**. The remaining native Windows work is alpha release qualification, not pre-alpha feature construction. Implementation follows the live Wayfinder build frontier plus accepted RFC/ADR authority and explicit owner sequencing decisions. Consequential unresolved decisions block only work that depends on them. Safe disjoint implementation, review, validation, security/privacy hardening, packaging/readiness work, documentation reconciliation, and blocker removal should continue.

Architectural invariant:

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, host harnesses, and tools are capabilities behind B.O.B.**

ADR-0006 and Accepted RFC-0004 permit genuinely harness-neutral B.O.B. capabilities to run behind multiple first-party hosts without transferring B.O.B. semantic ownership, state rules, identity, or authority to the host.

**B.O.T.** means **Bag of Tools**: an agent-like aggregate of capabilities without coherent identity, state, continuity, authority, policy, and user experience. B.O.B. must not collapse into a B.O.T.

## Governing maps and active owners

- **#30:** desktop alpha qualification waypoint and executable-readiness owner;
- **#79:** provider-independent inference and account onboarding;
- **#118:** bounded first DeepSeek host tracer under the explicit 2026-08-30 owner sequencing override;
- **#86:** calm primary workflow and progressive disclosure, **completed** after Settings, Today, Inbox, and Chat convergence landed with rendered evidence.

Closed/completed maps remain settled authority where relevant but do not remain active implementation owners forever.

## Landed baseline

Current `master` includes, at minimum:

- Tauri 2 desktop shell with Rust privileged core and framework-free TypeScript/Vite frontend;
- Rust-owned SQLite canonical state, migrations, managed backup/restore, and portable non-secret export;
- Today, Inbox, deterministic planning/replanning, durable restart handoff, and persisted accessibility preferences;
- B.O.B. Assist core with Rust-enforced typed proposal validation and preview-before-apply;
- OS-backed credential/secret-store boundary;
- advanced optional Gemini API credential/context-inference path with fail-closed provider-use/privacy/billing policy;
- accepted provider-independent runtime contract from RFC-0002 plus Rust-owned runtime policy;
- first non-user-facing Ollama tracer behind that contract;
- calm normal-mode Settings, Today, Inbox, and Chat presentation from merged PRs #89, #93, and #106;
- dependency-free hosted rendered diagnostics for normal/minimum window sizes, reduced-information mode, larger text, reduced motion, keyboard focus, advanced Gemini setup, and bounded recovery fixture states;
- committed desktop npm/Cargo lockfiles and the accepted targeted-clean Windows NSIS package-build path.

These are current implementation surfaces, not future phases.

## Primary alpha qualification frontier

### 1. Finish startup recovery acceptance

Issue #85 and draft PR #103 own the fail-closed user-reachable startup recovery surface.

The current recovery implementation provides a restricted recovery mode before ordinary commands run, preserves unreadable/corrupt canonical state, discovers only B.O.B.-managed backup candidates, previews candidates through the governed restore boundary, retains bounded candidate metadata, reports unusable candidates without destructive guessing, and provides a real application-process retry/restart path.

Hosted frontend, Windows Rust, and rendered recovery diagnostics are green on the exact branch heads recorded by that PR. They do **not** replace remaining native Windows 11 x64 obligations:

- corrupt canonical SQLite visibly enters recovery rather than exiting silently;
- healthy and invalid/corrupt managed backups exercise the real preview path;
- original corrupt canonical bytes remain unchanged;
- managed-backup/recovery-validation filesystem boundaries fail closed where safely reproducible;
- the real application-process retry/restart path works;
- optional credential-store initialization failure does not prevent deterministic launch where reproducible;
- the actual recovery surface is usable at normal/minimum supported sizes and relevant accessibility states.

Do not widen #103 into automatic restore, arbitrary import, filesystem browsing, provider work, or unrelated UI redesign merely to make the PR mergeable.

### 2. Complete native Windows NSIS lifecycle acceptance

Issue #84 owns the remaining native Windows 11 x64 install/launch/relaunch/icon/uninstall/retained-user-data acceptance. Hosted package creation and source-level runbook/tooling are not equivalent to native install acceptance.

Draft PR #115 owns only the separate native-smoke evidence helper/runbook. It may harden evidence collection but does not itself satisfy issue #84.

Close #84 only after the native acceptance contract in `docs/WINDOWS_NSIS_SMOKE.md` is actually executed and recorded, including default install path, launch, canonical-state location, state survival across full quit/relaunch, packaged/installed executable identity, embedded icon identity, uninstall, stopped process, retained user data, and truthful uninstall messaging.

### 3. Run the alpha qualification convergence checkpoint

After #103 and #84 receive truthful terminal dispositions, run a fresh exact-state convergence audit against Wayfinder #30 and the accepted alpha qualification criteria.

The checkpoint must answer:

- whether the current Alpha Product is genuinely native-qualified rather than merely implemented;
- whether any native credential/provider/restart evidence remains unowned;
- whether README, roadmap, validation, traceability, and current issue/PR state agree with the executable product;
- whether any in-scope consequential uncertainty would force the next implementation agent to invent policy;
- whether the correct next frontier is provider promotion, a newly exposed defect, or another already-authorized readiness action.

Do not call the integrated desktop alpha build release-qualified from merged-PR count or hosted CI alone.

### 4. Promote one already-authorized inference path with native evidence

After the desktop qualification/convergence frontier is reconciled, Wayfinder #79 becomes the primary provider build frontier again.

Current provider/runtime rules:

- Gemini Developer API remains a working advanced optional adapter and historical first seam proof;
- #80 established that ordinary Gemini consumer entitlement is not currently exposed through a direct embeddable Google OAuth inference API for B.O.B.;
- #81 established the B.O.B.-owned `LocalRuntimeAdapter` direction, initially favoring an in-process Rust engine with GGUF support while preserving optional Ollama/LM Studio compatibility;
- RFC-0002 and the Rust runtime policy define the minimum provider-independent identity/auth/billing/locality/capability/lifecycle contract;
- the landed Ollama tracer is an implementation proof, not a mandatory runtime or product identity;
- deterministic B.O.B. remains useful with no inference configured;
- no silent paid/provider/model fallback.

Promote **one** already-authorized path with real native evidence. Do not respond to a cleaner queue by inventing another adapter or fake provider controls.

## Parallel bounded portability frontier: issue #118 / draft PR #119

On 2026-08-30 the repository owner explicitly overrode the prior sequencing rule for one disjoint tracer. The override permits #118 to proceed before desktop alpha convergence provided the slice has no ownership overlap with #103/#115 and does not weaken or claim satisfaction of their native acceptance gates.

The target relationship is:

> **B.O.B. is the agent. DeepSeek is the harness.**

This is not a parallel B.O.B. implementation and not a Bag-of-Tools facade.

### QOR/CoreLogic planning result

The QOR planning/audit pass resolved the first implementation slice as a small vertical proof rather than a platform migration:

1. **Portable planning semantics:** extract only the deterministic remaining-work projection and its minimum input/result vocabulary.
2. **Desktop conformance:** keep Tauri/SQLite state ownership in the desktop host and map canonical work into the same portable semantic source.
3. **Bounded process protocol:** expose protocol version 1 through a one-request Rust stdio host with message bounds, request identity, explicit method/version checks, and fail-closed external-input validation.
4. **Thin DeepSeek edge:** register exactly one planning tool that invokes an explicit absolute sidecar path with shell evaluation disabled.
5. **No state or authority import:** do not create alternate-host persistence or inherit DeepSeek filesystem, shell, credentials, jobs, subagents, or arbitrary tools.
6. **Compatibility isolation:** pin/document the current DeepSeek developer-preview target and keep every Cordis/DeepSeek type inside `integrations/deepseek-harness`.

The adversarial audit recorded **PASS** after tightening executable configuration from an arbitrary command to an explicit absolute executable path.

### Current #119 implementation shape

The feature branch contains:

- `crates/bob-core`: `PlanningRequest`, `PlanningItem`, `PlanProjection`, validation, and deterministic ordering/capping semantics;
- `src-tauri/src/planner.rs`: canonical-state projection into the same portable planning source while persistence and mutation remain Tauri/Rust-host responsibilities;
- `crates/bob-capability-host`: bounded protocol version 1 plus real process-level stdin/stdout round-trip test;
- `integrations/deepseek-harness`: one `bob_plan_remaining_work` Cordis tool adapter, compatibility metadata, explicit absolute `hostPath`, cancellation, response-size bound, request/version validation, and no shell evaluation;
- `README.md`, `docs/ARCHITECTURE.md`, ADR-0006, RFC-0004, roadmap, and this plan reconciled with the host-portability and B.O.T. rules.

The tracer intentionally compiles the portable planning source into the desktop planner without changing the existing locked desktop Cargo dependency graph. That source inclusion is a temporary tracer mechanism, not the desired mature package boundary.

### #119 completion condition

The bounded implementation slice is complete when the exact PR head demonstrates:

- existing locked desktop Rust fmt/Clippy/tests remain green;
- frontend production build remains green;
- `bob-core` formatting/tests pass independently;
- capability-host formatting/tests and the real process-level stdio round trip pass;
- DeepSeek protocol-edge tests pass under pinned Node;
- compare against #103 and #115 remains zero changed-file overlap;
- documentation truthfully distinguishes the stateless tracer from production-ready stateful hosted B.O.B.

Do **not** claim from these checks that the plugin has been installed/unloaded successfully against DeepSeek Harness. That is the next compatibility-hardening evidence layer.

### Portability hardening after the tracer

Further portability work is not automatically authorized merely because #119 exists. Evidence from the tracer should drive the next bounded slice. Likely sequence:

1. replace source inclusion with a normal Cargo dependency/workspace boundary and lock the portable dependency graph;
2. package/discover `bob-capability-host` without shell command interpretation or arbitrary executable discovery;
3. install/unload against the pinned DeepSeek Harness target and validate plugin lifecycle, cancellation, malformed config, and adapter removal;
4. add a compatibility matrix only when more than one real harness version earns support;
5. design a fuller DeepSeek B.O.B. profile/identity surface only after the adapter boundary proves stable enough to justify it;
6. define **stateful hosted B.O.B.** only after explicit canonical state owner, namespace, migration/versioning, synchronization/conflicts, recovery, deletion/export, credential separation, and identity rules are accepted;
7. consider QOR Agent or additional host adapters only when a concrete use case creates conformance pressure.

GG-CORE remains an inference/runtime integration below B.O.B.'s inference port. QOR Agent remains an optional harness integration target through supported public seams. Neither becomes B.O.B.'s canonical state or product identity.

## Completed calm-workflow frontier

Wayfinder #86 is complete:

- Settings/provider positioning: PR #89;
- Today hierarchy/density: issue #87 / PR #93;
- Inbox/Chat density and conversation workspace: issue #88 / PR #106.

Future presentation defects should receive new bounded owners rather than reopening completed slices by default.

## Continuous tracer-bullet model

Select one bounded slice per governed cycle when possible. A good slice:

- is authorized by accepted product/architecture/governance, a current Wayfinder build-frontier owner, or an explicit owner disposition;
- has no overlapping active PR;
- produces a user-visible, security, validation, recovery, documentation, or executable-readiness improvement;
- fits one fresh coding-agent context;
- preserves explicit authority/security/cost/state boundaries;
- leaves a precise next action.

A native or human-only acceptance gate blocks only that item. Scheduled cycles should continue safe disjoint validation, hardening, evidence tooling, stale-state cleanup, documentation reconciliation, or next-frontier preparation rather than repeatedly reporting the same wait.

Do not manufacture speculative expansion merely to keep the loop occupied.

## Deferred or separately governed expansion

The following require later accepted authority or current Wayfinder convergence before implementation:

- additional account-backed runtime adapters beyond established supported integration facts;
- broader user-facing local inference beyond the accepted `LocalRuntimeAdapter` direction and already-landed tracer work;
- stateful B.O.B. hosting in DeepSeek or another external harness before explicit state ownership, namespace, migration, recovery, synchronization, deletion/export, credential, and identity semantics are accepted;
- mobile clients and cloud/shared continuity;
- Delegate/tool execution beyond bounded accepted authority;
- generalized RAG/knowledge-center infrastructure;
- broad plugin marketplaces or arbitrary dynamic plugin loading;
- peer-agent/multi-agent UX;
- cognitive profiling or diagnostic behavior.

## Legacy and historical material

The archive branch and Git history preserve retired implementation. `docs/pre-alpha/` and `docs/inspiration/` preserve prototype/design history. None compete with current product/architecture authority.

Do not keep obsolete runtime implementations in the active tree for sentimental continuity. Preserve still-required behavior by implementing it through current architecture and authority.

## Validation ownership

`docs/VALIDATION.md` is the accepted Windows-first desktop alpha qualification evidence contract. Implementing agents own truthful exact-head validation. Portable-host tests supplement that contract for #118; they do not redefine native Windows acceptance. Missing tooling or unavailable native/provider environments are explicit remaining debt, not passing evidence.

Repository CI remains deliberately small unless a stronger gate is justified by demonstrated risk.
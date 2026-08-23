# Current Implementation Plan

**Status:** Active sequencing context  
**Objective:** Continue advancing the accepted B.O.B. product through bounded, governed tracer bullets toward a coherent, validated desktop application.

## Strategy

The revival architecture is no longer hypothetical. The active tree contains the Tauri 2 + Rust application foundation, Rust-owned SQLite state, Today/Inbox deterministic workflows, B.O.B. Assist/proposal boundaries, secure secret storage, recovery/export foundations, accessibility preferences, Windows packaging authority, provider-independent runtime policy, an advanced optional Gemini API adapter, and a hosted rendered-UI diagnostic for presentation regressions.

Implementation follows the live Wayfinder **build frontier**, not a pre-build phase gate. Consequential unresolved decisions block only work that depends on them. Safe disjoint implementation, review, validation, UX/accessibility work, security/privacy hardening, packaging/readiness work, and documentation reconciliation should continue.

Architectural invariant:

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

ADR-0006 clarifies that this standalone-product invariant does not require every harness-neutral B.O.B. capability to remain physically coupled to the Tauri desktop host. Portable capabilities may later be consumed by other first-party host adapters while preserving explicit identity, state, and authority ownership.

## Current governing maps and completed destinations

- #30: first runnable B.O.B. alpha waypoint and current executable-readiness owner;
- #79: provider-independent inference and account onboarding;
- #86: calm primary workflow and progressive disclosure, **completed** after the Settings, Today, Inbox, and Chat convergence slices landed with rendered evidence.

Maps are decision/destination indexes, not permanent project trackers. Closed maps remain settled authority where relevant but must not be treated as active implementation owners.

## Landed foundation

Current `master` includes, at minimum:

- Tauri 2 desktop shell with Rust privileged core and TypeScript/Vite frontend;
- Rust-owned SQLite canonical work state, migrations, recovery copies, managed backup/restore, and portable non-secret export;
- Today, Inbox, deterministic planning/replanning, durable restart handoff, and accessibility preferences;
- B.O.B. Assist core with Rust-enforced preview-before-apply proposal authority;
- OS-backed credential/secret-store boundary;
- Windows-first packaging and validation contract;
- advanced optional Gemini API credential and context-inference path with fail-closed billing/privacy/provider-use policy;
- accepted provider-independent runtime contract from RFC-0002 plus Rust-owned fail-closed runtime policy;
- first non-user-facing Ollama tracer behind that contract, with conservative loopback/locality classification and no fallback or canonical-state authority;
- calm normal-mode Settings, Today, Inbox, and Chat presentation from merged PRs #89, #93, and #106;
- dependency-free hosted Chrome rendered diagnostics for normal/minimum window sizes, reduced-information mode, larger text, reduced motion, keyboard focus, and bounded setup-dialog presentation evidence.

These are current implementation surfaces, not future phases.

## Active build frontier

### 1. Finish executable recovery and Windows release acceptance

This is the highest-priority alpha-closing frontier.

#### Startup recovery

Issue #85 and draft PR #103 own the fail-closed startup recovery surface. PR #103 is the only open implementation PR at this reconciliation point.

Its implementation branch has been reduced to one bounded eight-file recovery delta on current `master`. Exact-head frontend build, Windows Rust format/Clippy/tests, and normal hosted rendered regression are green. Its PR body owns the exact current head and run IDs.

PR #103 remains draft because hosted CI cannot substitute for the remaining native recovery obligations:

- corrupt canonical SQLite must visibly enter recovery on Windows 11 x64;
- healthy and invalid/corrupt managed backup candidates must be exercised through the real preview path;
- original corrupt canonical bytes must remain unchanged;
- the real process-restart retry path must be exercised;
- optional credential-store initialization failure should leave deterministic B.O.B. launchable where reproducible;
- the actual recovery surface needs normal/minimum-window and accessibility evidence under native recovery state.

Do not widen #103 into automatic restore, arbitrary import, filesystem browsing, provider work, or unrelated UI redesign merely to make the PR mergeable.

#### Windows NSIS smoke

Issue #84 is intentionally open. Its earlier closure was corrected because hosted package creation and the merged smoke runbook did not satisfy native Windows install acceptance.

Remaining evidence is the exact Windows 11 x64 install, launch, canonical-state-location, quit/relaunch, embedded-icon, uninstall, retained-user-data, and truthful-uninstall-message contract in `docs/WINDOWS_NSIS_SMOKE.md`.

Do not close #84 again until that evidence is actually recorded.

### 2. Promote one already-authorized inference path with native evidence

Wayfinder #79 supersedes the idea that one provider is B.O.B.'s permanent destination.

Current rules:

- Gemini Developer API remains a working advanced optional adapter and first-alpha proof point;
- #80 resolved the Google/account-backed factual boundary: no direct embeddable Google OAuth inference API currently exposes ordinary Gemini consumer entitlement to B.O.B.; Antigravity remains only an optional external-runtime candidate behind B.O.B.-owned state/routing/policy;
- #81 resolved the local-runtime direction: prefer a Rust-owned `LocalRuntimeAdapter`, initially targeting an in-process Rust engine with GGUF support while preserving optional Ollama/LM Studio compatibility;
- RFC-0002 and the Rust runtime-policy foundation define the minimum provider-independent runtime/auth/billing/locality contract;
- the landed Ollama tracer is an implementation proof behind that contract, not a mandatory runtime or product identity;
- every path remains behind B.O.B.-owned routing, continuity, state, privacy, authority, and cost policy;
- deterministic B.O.B. remains useful with no inference configured;
- no silent paid/provider/model fallback.

After the active recovery/package acceptance frontier is reconciled, promote one already-authorized inference path with real native evidence. Do not respond to a clean queue by inventing another provider adapter.

### 3. Run the alpha convergence checkpoint before portability extraction

Once #103 is either merged with its required evidence or explicitly blocked on a genuine unavailable native acceptance surface, and #84 has truthful native disposition, run a fresh exact-state convergence audit against #30 and #79.

The checkpoint should answer:

- whether the first runnable alpha success criteria are actually evidenced rather than merely implemented;
- whether any native credential/provider/restart evidence remains unowned;
- whether README/roadmap/validation state matches the executable product;
- whether the next safe frontier is provider promotion, portability extraction, or a newly exposed defect.

Do not call the alpha complete from merged-PR count or hosted CI alone.

## Completed calm-workflow frontier

Wayfinder #86 and its bounded implementation owners are complete:

- Settings/provider positioning: PR #89;
- Today hierarchy/density: issue #87 / PR #93;
- Inbox/Chat density and conversation workspace: issue #88 / PR #106.

These slices were merged after exact-head frontend validation plus hosted rendered artifacts inspected at supported normal/minimum sizes and relevant accessibility states. Future presentation defects should receive new bounded owners rather than reopening completed slices by default.

## Authorized post-alpha portability frontier

Completed design-intent issue #109 and ADR-0006 establish the future **portable capability core + first-party host adapters** direction. The issue is closed because the architectural intent and sequencing are durably recorded; implementation authority remains in Proposed RFC-0004.

Before substantial portability code lands, RFC-0004 must settle the first portable capability slice, cross-language bridge, protocol/versioning shape, supported DeepSeek Harness version range, and validation matrix.

Expected bounded sequence after alpha convergence:

1. perform RFC-0004 P0 boundary audit and classify current Rust modules without moving code;
2. extract one deterministic vertical slice that can run without Tauri;
3. define only the host ports/conformance fixtures required by that slice;
4. move the existing desktop path onto the same contract and prove no standalone regression;
5. implement a thin B.O.B.-owned DeepSeek Harness tracer adapter against a pinned supported harness version;
6. harden compatibility, lifecycle/cancellation, packaging, failure behavior, and adapter removal;
7. consider QOR Agent or additional adapters only when they prove a concrete requirement rather than expanding the abstraction surface by anticipation.

GG-CORE remains an inference/runtime integration below B.O.B.'s inference port. QOR Agent remains an optional harness integration target through its public seams. Neither requires a write outside this repository for B.O.B. portability work.

## Continuous tracer-bullet model

Select one bounded slice per governed cycle when possible. A good slice:

- is authorized by accepted product/architecture/governance or a current Wayfinder build-frontier owner;
- has no overlapping active PR;
- produces a user-visible, security, validation, recovery, documentation, or executable-readiness improvement;
- fits one fresh coding-agent context;
- preserves explicit authority/security/cost/state boundaries;
- leaves a precise next action.

A native or human-only acceptance gate blocks only that item. The automation should continue safe disjoint validation, hardening, evidence tooling, documentation reconciliation, or next-frontier preparation rather than repeatedly reporting the same wait.

Do not manufacture speculative expansion merely to keep the loop occupied.

## Deferred or separately governed expansion

The following require later accepted authority or current Wayfinder convergence before implementation:

- additional account-backed runtime adapters beyond currently established supported integration facts;
- broader local inference implementation beyond the accepted `LocalRuntimeAdapter` direction and already-landed tracer work;
- stateful B.O.B. embedding in an external harness before explicit state ownership, namespace, migration, recovery, and synchronization semantics are accepted;
- mobile clients and cloud/shared continuity;
- Delegate/tool execution beyond bounded accepted authority;
- generalized RAG/knowledge-center infrastructure;
- broad plugin marketplaces or arbitrary dynamic plugin loading;
- peer-agent/multi-agent UX;
- cognitive profiling or diagnostic behavior.

## Legacy cleanup policy

Git history and the named archive branch preserve retired experiments. Do not keep obsolete runtime implementations in the active tree for sentimental continuity. Preserve still-required behavior by implementing it through current architecture and authority.

## Validation ownership

`docs/VALIDATION.md` is the accepted Windows-first evidence contract. Implementing agents own truthful exact-head validation. Missing tooling or unavailable native/provider environments are explicit remaining debt, not passing evidence.

Repository CI remains deliberately small unless a stronger gate is justified by demonstrated risk.

# Current Implementation Plan

**Status:** Active sequencing context  
**Objective:** Advance B.O.B. from the current landed local-first desktop baseline through truthful first-alpha acceptance, then continue into the next already-authorized runtime and portability frontiers without speculative widening.

## Strategy

The revival architecture is no longer hypothetical. `master` contains the Tauri 2 + Rust application foundation, Rust-owned SQLite state, Today/Inbox/Chat/Settings, deterministic planning, B.O.B. Assist/proposal validation, protected credentials, recovery/export foundations, accessibility preferences, provider-independent runtime policy, an advanced optional Gemini API adapter, a bounded Ollama tracer, locked Windows packaging, and hosted rendered diagnostics.

Implementation follows the live Wayfinder **build frontier**, not a pre-build phase gate. Consequential unresolved decisions block only work that depends on them. Safe disjoint implementation, review, validation, security/privacy hardening, packaging/readiness work, documentation reconciliation, and blocker removal should continue.

Architectural invariant:

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

ADR-0006 additionally permits genuinely harness-neutral B.O.B. capabilities to become portable behind B.O.B.-owned typed contracts without transferring standalone product identity, state, or authority to another host.

## Governing maps

- **#30:** first runnable B.O.B. alpha waypoint and current executable-readiness owner;
- **#79:** provider-independent inference and account onboarding;
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
- committed npm/Cargo lockfiles and the accepted targeted-clean Windows NSIS package-build path.

These are current implementation surfaces, not future phases.

## Active first-alpha build frontier

### 1. Finish startup recovery acceptance

Issue #85 and draft PR #103 own the fail-closed user-reachable startup recovery surface. PR #103 is the sole open product implementation PR for this frontier.

The current recovery implementation provides a restricted recovery mode before ordinary commands run, preserves unreadable/corrupt canonical state, discovers only B.O.B.-managed backup candidates, previews candidates through the governed restore boundary, retains bounded candidate metadata, reports unusable candidates without destructive guessing, and provides a real application-process retry/restart path.

Hosted frontend, Windows Rust, and rendered recovery diagnostics are green on the current exact branch head recorded by the PR. They do **not** replace remaining native Windows 11 x64 obligations:

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

### 3. Run the first-alpha convergence checkpoint

After #103 and #84 receive truthful terminal dispositions, run a fresh exact-state convergence audit against Wayfinder #30 and the accepted first-alpha success criteria.

The checkpoint must answer:

- whether the first runnable alpha is genuinely evidenced rather than merely implemented;
- whether any native credential/provider/restart evidence remains unowned;
- whether README, roadmap, validation, traceability, and current issue/PR state agree with the executable product;
- whether any in-scope consequential uncertainty would force the next implementation agent to invent policy;
- whether the correct next frontier is provider promotion, a newly exposed defect, or another already-authorized readiness action.

Do not call the alpha complete from merged-PR count or hosted CI alone.

### 4. Promote one already-authorized inference path with native evidence

Only after the first-alpha readiness/convergence frontier is reconciled should Wayfinder #79 become the primary build frontier again.

Current provider/runtime rules:

- Gemini Developer API remains a working advanced optional adapter and historical first seam proof;
- #80 established that ordinary Gemini consumer entitlement is not currently exposed through a direct embeddable Google OAuth inference API for B.O.B.;
- #81 established the B.O.B.-owned `LocalRuntimeAdapter` direction, initially favoring an in-process Rust engine with GGUF support while preserving optional Ollama/LM Studio compatibility;
- RFC-0002 and the Rust runtime policy define the minimum provider-independent identity/auth/billing/locality/capability/lifecycle contract;
- the landed Ollama tracer is an implementation proof, not a mandatory runtime or product identity;
- deterministic B.O.B. remains useful with no inference configured;
- no silent paid/provider/model fallback.

Promote **one** already-authorized path with real native evidence. Do not respond to a cleaner queue by inventing another adapter or fake provider controls.

## Completed calm-workflow frontier

Wayfinder #86 is complete:

- Settings/provider positioning: PR #89;
- Today hierarchy/density: issue #87 / PR #93;
- Inbox/Chat density and conversation workspace: issue #88 / PR #106.

Future presentation defects should receive new bounded owners rather than reopening completed slices by default.

## Authorized post-alpha portability frontier

ADR-0006 and completed design-intent issue #109 establish the future **portable capability core + first-party host adapters** direction. Proposed RFC-0004 owns unresolved implementation choices.

Substantial portability implementation begins only after first-alpha stabilization/convergence and RFC-0004 acceptance.

Expected bounded sequence:

1. classify current Rust modules against the portable/host-specific boundary without moving code;
2. extract one deterministic vertical slice that can run without Tauri;
3. define only the host ports/conformance fixtures that slice actually requires;
4. move the desktop path onto the same contract and prove no standalone regression;
5. implement a thin B.O.B.-owned DeepSeek Harness tracer against a pinned supported harness version;
6. harden compatibility, lifecycle/cancellation, packaging, failure behavior, and adapter removal;
7. consider QOR Agent or additional adapters only when they prove a concrete requirement.

GG-CORE remains an inference/runtime integration below B.O.B.'s inference port. QOR Agent remains an optional harness integration target through supported public seams. Neither becomes B.O.B.'s canonical state or product identity.

## Continuous tracer-bullet model

Select one bounded slice per governed cycle when possible. A good slice:

- is authorized by accepted product/architecture/governance or a current Wayfinder build-frontier owner;
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
- stateful B.O.B. embedding in an external harness before explicit state ownership, namespace, migration, recovery, synchronization, deletion/export, and credential semantics are accepted;
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

`docs/VALIDATION.md` is the accepted Windows-first evidence contract. Implementing agents own truthful exact-head validation. Missing tooling or unavailable native/provider environments are explicit remaining debt, not passing evidence.

Repository CI remains deliberately small unless a stronger gate is justified by demonstrated risk.

# Current Implementation Plan

**Status:** Active sequencing context  
**Objective:** Continue advancing the accepted B.O.B. product through bounded, governed tracer bullets toward a coherent, validated desktop application.

## Strategy

The revival architecture is no longer hypothetical. The active tree contains the Tauri 2 + Rust application foundation, Rust-owned SQLite state, Today/Inbox deterministic workflows, B.O.B. Assist/proposal boundaries, secure secret storage, recovery/export foundations, accessibility preferences, Windows packaging authority, and an advanced optional Gemini API adapter.

Implementation therefore follows the live Wayfinder **build frontier**, not a pre-build phase gate. Consequential unresolved decisions block only work that depends on them. Safe disjoint implementation, review, validation, UX/accessibility work, security/privacy hardening, packaging/readiness work, and documentation reconciliation should continue.

Architectural invariant:

> **B.O.B. is the agent. Models, inference runtimes, provider APIs/CLIs, and tools are capabilities behind B.O.B.**

## Current governing maps

- #30: first runnable B.O.B. alpha waypoint and settled foundational authority;
- #79: provider-independent inference and account onboarding;
- #86: calm primary workflow and progressive disclosure.

Maps are decision/destination indexes, not permanent project trackers. Use their current build-frontier owners and open PR state to avoid duplicate work.

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
- first non-user-facing Ollama tracer behind that contract, with conservative loopback/locality classification and no fallback or canonical-state authority.

These are current implementation surfaces, not future phases.

## Active build-frontier priorities

### 1. Calm everyday workflow

Wayfinder #86 owns the current rendered UX convergence. Preferred bounded sequence when live ownership permits:

1. Settings/provider positioning cleanup under #82;
2. Today hierarchy/density convergence under #87;
3. Inbox and Chat density/empty-state refinement under #88.

Material UI changes require rendered evidence at normal and minimum supported Windows sizes, including normal/reduced-information modes where relevant, larger text, keyboard focus, reduced motion, clipping/overlap, dead space, and hierarchy.

### 2. Provider-independent inference

Wayfinder #79 supersedes the idea that one provider is B.O.B.'s permanent destination.

Current rules:

- Gemini Developer API remains a working advanced optional adapter and first-alpha proof point;
- #80 resolved the Google/account-backed factual boundary: no direct embeddable Google OAuth inference API currently exposes ordinary Gemini consumer entitlement to B.O.B.; Antigravity is only an optional external-runtime candidate behind B.O.B.-owned state/routing/policy;
- #81 resolved the local-runtime direction: prefer a Rust-owned `LocalRuntimeAdapter`, initially targeting an in-process Rust engine with GGUF support while preserving optional Ollama/LM Studio compatibility;
- RFC-0002 and the Rust runtime-policy foundation define the minimum provider-independent runtime/auth/billing/locality contract;
- the landed Ollama tracer is an implementation proof behind that contract, not a mandatory runtime or product identity;
- every path remains behind B.O.B.-owned routing, continuity, state, privacy, authority, and cost policy;
- deterministic B.O.B. remains useful with no inference configured;
- no silent paid/provider/model fallback.

Do not invent unsupported account-backed entitlement, new adapters, or future-provider controls. The current priority is to finish active native/rendered recovery, UX, and packaging acceptance before widening provider surface area, then promote one already-authorized inference path with real native evidence.

### 3. Executable readiness

Source review and small hosted CI are not release evidence by themselves. Continue closing exact-head readiness debt with the strongest available evidence:

- `npm run validate` and locked dependency reproducibility;
- Rust format/clippy/test and Tauri build on capable environments;
- Windows restart/persistence/recovery exercises;
- Windows Credential Manager set/replace/remove/restart behavior;
- rendered accessibility/UX regression;
- provider-boundary behavior where a live provider is relevant;
- NSIS package install/launch/uninstall smoke.

If a merge/rebase changes a previously validated head, rerun the affected evidence rather than inheriting it cosmetically.

## Continuous tracer-bullet model

Select one bounded slice per governed cycle when possible. A good slice:

- is authorized by accepted product/architecture/governance or a current Wayfinder build-frontier owner;
- has no overlapping active PR;
- produces a user-visible, security, validation, recovery, documentation, or executable-readiness improvement;
- fits one fresh coding-agent context;
- preserves explicit authority/security/cost/state boundaries;
- leaves a precise next action.

Do not manufacture speculative expansion merely to keep the loop occupied.

## Deferred or separately governed expansion

The following require later accepted authority or current Wayfinder convergence before implementation:

- additional account-backed runtime adapters beyond currently established supported integration facts;
- broader local inference implementation beyond the accepted `LocalRuntimeAdapter` direction and already-landed tracer work;
- mobile clients and cloud/shared continuity;
- Delegate/tool execution beyond bounded accepted authority;
- generalized RAG/knowledge-center infrastructure;
- plugin marketplaces;
- peer-agent/multi-agent UX;
- cognitive profiling or diagnostic behavior.

## Legacy cleanup policy

Git history and the named archive branch preserve retired experiments. Do not keep obsolete runtime implementations in the active tree for sentimental continuity. Preserve still-required behavior by implementing it through current architecture and authority.

## Validation ownership

`docs/VALIDATION.md` is the accepted Windows-first evidence contract. Implementing agents own truthful exact-head validation. Missing tooling or unavailable native/provider environments are explicit remaining debt, not passing evidence.

Repository CI remains deliberately small unless a stronger gate is justified by demonstrated risk.
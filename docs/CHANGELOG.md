# Changelog

All notable changes to the revived B.O.B. project are recorded here. This file describes the active revival line, not every historical prototype experiment.

## Unreleased

### Product-stage classification

- classified B.O.B. as an **Alpha Product** based on the current implemented desktop product baseline;
- replaced active `pre-alpha` product-stage language in canonical documentation with `Alpha Product`;
- retained native Windows startup-recovery, NSIS lifecycle, and convergence work as **alpha release qualification debt**, not as a reason to revert the product-stage classification;
- retained `docs/pre-alpha/` and related references only as historical/prototype provenance.

### Runnable local-first desktop foundation

- implemented the Tauri 2 desktop host with a Rust privileged core and framework-free TypeScript/Vite frontend;
- implemented Rust-owned SQLite canonical state with monotonic migrations and transactional mutation boundaries;
- implemented Today and Inbox work surfaces, quick capture, task lifecycle, deterministic planning/replanning, and restart handoff continuity;
- implemented B.O.B. Assist with typed proposal validation and preview-before-apply for important state changes;
- implemented persisted accessibility/preferences behavior through the Rust-owned state boundary;
- implemented managed backup/restore foundations and versioned non-secret portable export;
- implemented protected credential handling through the OS-backed secret-store boundary, using Windows Credential Manager on the Windows-first path.

### Inference and provider independence

- used Gemini Developer API Free as the first working inference/credential/privacy/cost tracer while retaining it as an advanced optional adapter rather than the B.O.B. product identity;
- established the accepted provider-independent runtime/auth/billing/locality contract in RFC-0002;
- rejected the original fixed subscription-first ordering from ADR-0003 and reconciled PRD-0003 to current provider-independent policy;
- implemented Rust-owned fail-closed runtime policy;
- implemented a bounded non-user-facing Ollama tracer behind the runtime contract;
- resolved the current account-backed-provider research boundary and the B.O.B.-owned `LocalRuntimeAdapter` direction through Wayfinder #79/#80/#81;
- preserved deterministic B.O.B. operation with no inference configured and no silent paid/provider/model fallback.

### Calm primary workflow

- converged ordinary Settings through PR #89, removing persistent provider-specific status and development-governance exposition from normal product chrome while retaining Gemini as an advanced optional capability;
- converged Today hierarchy/density through issue #87 / PR #93;
- converged Inbox and B.O.B. Chat density/conversation-workspace behavior through issue #88 / PR #106;
- completed Wayfinder #86 for the calm primary workflow and progressive-disclosure direction;
- added hosted rendered diagnostics at normal/minimum supported sizes, including reduced-information mode, larger text, reduced motion, keyboard focus, and bounded advanced-provider setup presentation.

### Validation and Windows packaging

- committed deterministic npm and Cargo lockfiles and moved validation to locked dependency resolution;
- added path-scoped frontend and Windows Rust diagnostics rather than a broad expensive CI matrix;
- added the hosted rendered-UI diagnostic and hardened Chrome startup for repeatable presentation evidence;
- established the accepted targeted-clean Windows NSIS packaging path to prevent stale native resource reuse;
- produced reproducible hosted Windows package-build evidence while explicitly keeping native install acceptance separate;
- retained issue #84 for the real Windows 11 x64 install/launch/relaunch/icon/uninstall/retained-user-data acceptance contract;
- added draft PR #115 as separate native-smoke evidence tooling/runbook without representing that tooling as completed native acceptance.

### Startup recovery

- implemented the bounded fail-closed startup-recovery surface in draft PR #103;
- preserve unreadable/corrupt canonical state rather than silently resetting or deleting it;
- restrict recovery to B.O.B.-managed backup candidates and bounded recovery-validation staging;
- provide governed candidate preview, truthful bounded-list disclosure, safe failure states, and a real application-process retry/restart path;
- added hosted rendered recovery-state coverage while retaining native Windows filesystem/lifecycle/accessibility behavior as outstanding acceptance debt under issue #85.

### Architecture and future portability

- accepted ADR-0006 for a future portable B.O.B. capability core with first-party host adapters while preserving standalone B.O.B. identity, state, and authority ownership;
- completed design-intent issue #109 and moved unresolved portability implementation choices to Proposed RFC-0004;
- sequenced substantial portability/DeepSeek-harness work after alpha stabilization and convergence rather than ahead of executable readiness.

### Public open-source baseline

- made the repository public and adopted the MIT License;
- added B.O.B. hero artwork, application/mascot asset roles, and public-facing status/design badges;
- added contribution, security, support, code-of-conduct, issue, and pull-request guidance;
- documented intentionally minimal GitHub Actions usage and developer/coding-agent-owned validation;
- added public repository maintenance and private-vulnerability-reporting guidance.

### Repository reset and historical preservation

- established B.O.B. as Better Organized Brain, a single-agent local-first personal AI workbench;
- retired the Electron/Ollama/RAG-era implementation from the active tree;
- removed the legacy dependency graph, duplicate scripts, checked-in embedding model, Python skeleton, historical version directories, generated inventories, and other prototype artifacts from `master`;
- preserved the complete pre-cleanup tree on `archive/pre-revival-cleanup-2026-08-19`;
- retained generated pre-alpha inspiration/prototype artifacts as explicitly non-authoritative historical/reference material.

## Current alpha qualification frontier

The current release-qualification sequence is:

1. native Windows startup-recovery acceptance for issue #85 / draft PR #103;
2. native Windows NSIS lifecycle acceptance for issue #84 / draft PR #115;
3. Wayfinder #30 convergence audit against the full alpha qualification criteria;
4. after truthful convergence, promote one already-authorized inference path with native evidence.

B.O.B. is an **Alpha Product** now. Hosted CI, source review, or package creation alone are not represented as native release qualification, but outstanding native qualification work does not change the product stage back to pre-alpha.

## Legacy alpha history

The original alpha changelog is preserved as [`legacy/ALPHA_CHANGELOG.md`](legacy/ALPHA_CHANGELOG.md). Historical implementation remains available in older Git history and the named archive branch.

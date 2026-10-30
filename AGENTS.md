# AGENTS.md

Binding repository instructions for coding agents and agent-assisted development.

## Mission

Build B.O.B. as a small, vendor-neutral personal AI workbench with ADHD-friendly interaction design.

> **B.O.B. is the agent. Models, inference runtimes, and tools are capabilities behind B.O.B.**

## Read before changing anything

Read, in order:

1. `README.md`
2. `docs/governance/GOVERNANCE.md`
3. `docs/governance/WAYFINDER.md` when a Wayfinder map is active or implementation derives from one
4. `docs/PRODUCT.md`
5. `docs/ARCHITECTURE.md`
6. `docs/DESIGN.md` when user-facing behavior is involved
7. the governing PRD, RFC, ADR, implementation-plan sections, owning issue, and current active-work claims for the change

The active tree is authoritative. Historical implementation is preserved on `archive/pre-revival-cleanup-2026-08-19` and must not be treated as current architecture unless a governing record explicitly brings a concept forward.

When an active Wayfinder map intentionally changes a destination-specific requirement, explicit owner dispositions in resolved Wayfinder tickets govern that map destination until binding documentation is reconciled. Treat conflicts as stale-document debt, not permission to reopen a settled decision silently.

Full Wayfinder convergence is required before declaring the map complete or treating its final synthesized waypoint specification as fully reconciled authority. It is **not** a blanket freeze on implementation whose required decisions are already settled. Safe disjoint tracer bullets, review, validation, security/privacy hardening, packaging/readiness work, and blocker removal may proceed while unrelated consequential decisions remain open, provided the work does not depend on those unresolved choices or conflict with an active implementation claim.

## Non-negotiable boundaries

- B.O.B. is the only user-facing agent identity.
- Multiple LLMs, inference runtimes, vendor CLIs, and tools are internal capabilities, not peer agents.
- B.O.B. owns canonical task, plan, preference, conversation continuity, and application state.
- Runtime adapters do not become systems of record.
- Core capture, task, and planning behavior remains useful without AI inference.
- Prefer configured zero-cost or already-included inference paths that fit the active product boundary; metered inference requires explicit user enablement.
- Unknown provider cost classification fails closed.
- Normal Assist mode does not grant shell, filesystem, credential, repository, or arbitrary application authority.
- Delegate mode means the user delegates bounded authority to B.O.B., which may use an approved runtime or tool inside that grant.
- Model/runtime output is untrusted until B.O.B. validates proposed application actions.
- Provider/runtime detail may be visible when relevant, but must not become a multi-agent interaction model.
- Do not add cognitive profiling, diagnostic scoring, inferred neurodivergence traits, guilt mechanics, or productivity scoring disguised as wellbeing support.
- Accessibility and low cognitive load are product requirements, not post-launch polish.
- Do not add a service, daemon, database, framework, runtime, or dependency without a current requirement and documented architectural reason.
- Prefer deletion and a narrow typed boundary over parallel implementations.

## Change classification

Use the smallest record set that makes the change traceable:

| Change | Record |
| --- | --- |
| Material user capability or behavior | PRD |
| Significant implementation mechanism, protocol, integration, or migration | RFC |
| Durable architectural decision | ADR |
| Routine bug, test, copy, or dependency maintenance | PR only |

A record marked `Proposed` is not permission to pretend an unresolved consequential decision is settled. If implementation depends on it, resolve that decision first.

When Wayfinder is active, resolved decision tickets identify what must be reconciled into PRDs/RFCs/ADRs. A decision ticket is not a permanent substitute for the repository records required by this table.

## Engineering discipline

- Keep UI, B.O.B. agent core, deterministic services, persistence, policy, inference routing, runtime adapters, and tool gateway separated by explicit boundaries.
- Prefer typed data contracts at every trust boundary.
- Keep deterministic planning logic deterministic and directly tested.
- Never put secrets in frontend state, logs, prompts, screenshots, fixtures, or ordinary application data.
- Make destructive state changes recoverable where practical.
- Treat provider CLIs and external runtimes as external processes with failure, version, auth, cancellation, and output-validation concerns.
- Do not silently widen workspace or filesystem permissions.
- Do not silently change provider, runtime, model, or cost class where that change materially affects user intent, privacy, or billing.
- Preserve B.O.B. conversation identity when the inference backend changes.
- Update documentation in the same change that makes a documented statement true or false.
- Before starting a slice, inspect current active implementation claims and open PRs; do not duplicate overlapping work.

## Validation responsibility

GitHub Actions and required CI gates are intentionally minimal for this small project. The implementing developer or coding agent owns validation before requesting review.

Run the checks that are relevant to the change, such as builds, unit tests, linting, type checks, targeted integration tests, restart/recovery checks, packaging checks, and manual/rendered UI validation. Do not invent checks merely to fill a template. A pull request must state exactly what was run, what passed, what was not run, and why.

Repository CI is a safety net when present, not a substitute for implementation-time validation.

## Repository hygiene

The repository root is intentionally sparse. Do not place implementation experiments, generated inventories, model artifacts, temporary scripts, screenshots, research dumps, or historical copies in the root.

Expected root entries are limited to core repository surfaces such as `README.md`, `AGENTS.md`, `LICENSE`, `.gitignore`, `.github/`, `docs/`, and active application directories/configuration required to build the revived product.

Git history and the named archive branch are the archive. Do not recreate a museum in `master`.

## Pull requests

A material pull request must state:

- user problem and intended outcome;
- scope and explicit non-goals;
- governing PRD/RFC/ADR and Wayfinder references when applicable;
- architecture, data, authority, security, and inference-cost impact;
- validation performed and evidence obtained;
- documentation changed with the behavior;
- active-work coordination when another implementation slice is in flight.

No AI attribution, generated-by footer, Co-Authored-By marker, or agent branding is required in repository artifacts.
## Windows code signing

Windows binaries and installers this repository distributes **must be Authenticode-signed at release time** with the MythologIQ Labs Azure Artifact Signing profile. Signing is available and verified, so don't ship an unsigned public Windows release.

- Endpoint `https://eus.codesigning.azure.net/`, account `MythologIQ-Labs-Signing`, certificate profile `mythologiq-labs-public-release` (Public Trust). Publisher shown to users: `MythologIQ Labs, LLC`. Timestamp server: `http://timestamp.acs.microsoft.com`.
- Canonical procedure, verified tooling and framework integration: `MythologIQ-Labs-LLC/Myth-Tech-Forge` at `docs/WINDOWS_CODE_SIGNING_STANDARD.md`. Follow it; don't invent another signing path.
- Sign release builds only (release tag or manual release), never PR or routine CI builds. Windows runners bill at 2x Actions minutes.
- Signing requires the Artifact Signing Certificate Profile Signer role. Today only the owner's local `az login` identity has it. A CI signing identity (GitHub OIDC) doesn't exist yet, and creating one, or any secret or role assignment, needs owner approval. Never commit credentials.
- A release is signed only after `signtool verify /pa` passes, the signer is `MythologIQ Labs, LLC`, and the signature is timestamped. Record that output in the release evidence.

**This repository (Tauri v2, NSIS installer):** sign the app binary and the NSIS installer through `bundle.windows.signCommand` in a release-only Tauri config, as the standard describes. Keep `docs/WINDOWS_NSIS_SMOKE.md` checks for signed builds too.

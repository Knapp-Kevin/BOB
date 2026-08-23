# B.O.B. Alpha Validation and Packaging

**Status:** Accepted first-alpha validation contract  
**Wayfinder:** #39  
**Primary platform:** Windows 11 x64

This document defines the evidence required before B.O.B. can be described as a runnable, reviewable first alpha. CI is deliberately small. The implementing developer or coding agent owns the stronger local, native, rendered, and failure-path evidence.

## Core command gate

From a clean repository checkout:

```powershell
npm ci
npm run validate
```

`npm ci` is the canonical frontend dependency installation path for validation. It must consume the committed `package-lock.json` without rewriting dependency resolution.

`npm run validate` executes the current frontend build/type gate and the Rust formatting, lint, and test gate:

```text
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Do not report an unexecuted command as passing. Source review is evidence, not execution.

## Dependency reproducibility

The canonical npm and Cargo lockfiles are committed repository inputs:

- `package-lock.json` owns frontend dependency resolution;
- `src-tauri/Cargo.lock` owns Rust dependency resolution.

Clean validation must install frontend dependencies with `npm ci` and run dependency-resolving Cargo validation with `--locked`. A manifest/lockfile mismatch is a validation failure, not permission to refresh dependencies silently.

When dependency changes are intentional, regenerate the applicable lockfile in a toolchain-capable environment, review the dependency delta, commit manifest and lockfile changes together, and rerun the locked gates.

The Rust dependency baseline currently requires Rust 1.88 or newer.

## Native Windows smoke gate

For changes touching the Rust/Tauri boundary, persistence, credentials, or packaging, exercise the application on Windows 11 x64:

1. launch the Tauri app;
2. navigate Today, Inbox, Chat, and Settings;
3. perform a state mutation and verify the UI reflects it;
4. quit the application completely;
5. relaunch and verify canonical state survives;
6. verify failure handling leaves deterministic Today/Inbox behavior usable.

A Vite/browser preview is useful for presentation work but cannot satisfy a native-boundary acceptance criterion.

## Recovery failure-path evidence

Persistence/recovery work must prove not only that a replacement snapshot can be created, but that failure while replacing recovery state does not destroy the last verified recovery artifact.

For any bounded `pending -> promoted` recovery rotation, exercise or deterministically simulate these boundaries where the platform/tooling permits:

1. pending snapshot creation failure leaves the previously promoted known-good snapshot intact;
2. pending snapshot integrity-validation failure leaves the previously promoted known-good snapshot intact;
3. final promotion/replacement failure leaves at least one verified recoverable snapshot intact and does not silently report success;
4. failed candidate restore rolls canonical state back to the verified pre-restore snapshot;
5. cleanup of stale pending artifacts cannot delete the currently promoted known-good recovery snapshot.

Source review of rename/replacement code is useful but does not replace native Windows filesystem behavior where recovery correctness depends on platform replacement semantics.

## Product acceptance matrix

| Surface / boundary | Required evidence |
| --- | --- |
| Today | next action, completion, defer, focus list, capture |
| Inbox | capture, filtering, promote-to-next-action, organize/preview |
| Chat | deterministic behavior; bounded live inference only after provider-purpose/data-use acknowledgement |
| Preview-before-apply | proposal can be dismissed without mutation; apply changes only the previewed state |
| Accessibility | keyboard reachability, visible focus, larger text, reduced motion, readable hierarchy at minimum supported window |
| Persistence | mutation survives full quit/restart |
| Recovery | migration/snapshot/rotation failure cannot silently reset canonical data; known-good copies survive failed snapshot creation, validation, and promotion |
| Credentials | set, restart/status, failed replacement preserves prior key, valid replacement, explicit removal |
| Runtime failure | invalid auth, quota/rate limit, provider/network outage, timeout leave deterministic B.O.B. usable |
| Rendered desktop | Today, Inbox, Chat, Settings at normal and minimum supported window sizes |
| Packaging | install, launch, uninstall Windows 11 x64 NSIS package |

## Credential/privacy/provider evidence

Never use a real production credential in screenshots, fixtures, issue bodies, test output, or logs.

Windows credential acceptance must prove:

```text
set -> restart -> status/read/validate -> failed replace preserves old credential
    -> valid replace -> delete
```

The raw secret must never appear in SQLite, portable exports, ordinary frontend state, or a command that reads it back to the UI.

Before claiming context-bearing Gemini Developer API Free inference as accepted, native/rendered evidence must also prove:

1. the user is told that this unpaid provider path is for professional or business use rather than general consumer use;
2. the user is told that unpaid-service content and generated responses may be used by Google for product/model improvement and may be reviewed by humans;
3. the user is explicitly told not to send sensitive, confidential, or personal information through this unpaid path;
4. no context-bearing inference is sent until the user affirmatively acknowledges that boundary;
5. declining or withholding acknowledgement leaves deterministic B.O.B. fully usable;
6. quota/auth/provider failure never silently transitions to paid or different inference;
7. the current Gemini API terms and regional/billing applicability are rechecked at release validation time.

The governing provider boundary is in `docs/governance/AI_COST_AND_PROVIDER_POLICY.md`; issue #57 records the 2026-08-21 reconciliation of the current Gemini API Additional Terms.

## Packaging

The first alpha package is a Windows 11 x64 NSIS installer.

Build it on Windows with:

```powershell
npm ci
npm run package:windows
```

The packaging script first runs Cargo metadata with `--locked`, so a Rust manifest/lockfile mismatch fails before packaging work begins. It then runs a targeted `cargo clean --manifest-path src-tauri/Cargo.toml -p bob` before invoking the Tauri NSIS build. The targeted clean is required release evidence, not optional housekeeping: native validation on PR #83 demonstrated that a warm Cargo target can otherwise reuse stale Tauri resource objects after application-icon changes. The targeted clean removes B.O.B.'s package artifacts without turning routine packaging into a full dependency-cache purge.

Acceptance requires:

- installer is produced successfully from the locked, targeted-clean packaging path;
- the packaged executable embeds the canonical current application icon/resources rather than a warm-target predecessor;
- default per-user installation works without unnecessary administrator elevation;
- installed application launches;
- local canonical state is stored in the application data location rather than the install directory;
- uninstall completes without claiming to remove user data unless that behavior is explicitly implemented and documented.

When collecting Windows shell/icon evidence, account for Explorer/icon caching separately from the executable's embedded resource so cached presentation is not mistaken for build output.

MSI is deferred for the first alpha. It duplicates the installer surface and adds WiX/VBScript requirements without a current product requirement.

macOS/Linux installers, cross-platform matrices, store publication, auto-update infrastructure, and broad release automation are deferred until a later release requires them.

## Code signing

A clearly identified local/developer alpha may be built and reviewed before a production signing certificate exists. Broad external distribution must not be described as production-ready until the Windows signing/distribution policy is implemented and validated.

## Hosted CI boundary

Do not create a large hosted matrix.

The current diagnostic safety net is intentionally path-scoped and consumes committed lockfiles deterministically:

- frontend changes under `src/**` plus frontend manifest/lock/config changes run Node 22 `npm ci` followed by `npm run build`;
- Rust changes under `src-tauri/**` run Rust 1.88 format plus Clippy and tests with locked dependency resolution.

These jobs are useful build/type/format/lint/test execution evidence. They do not replace the stronger platform and product evidence required by the affected change.

Routine hosted CI does not replace Windows Credential Manager testing, restart/recovery testing, rendered UX review, installer verification, or live-provider evidence.

## PR evidence template

Every material PR must state:

- governing PRD/RFC/ADR/Wayfinder decisions;
- commands actually executed and exact results;
- native/manual flows actually exercised;
- rendered evidence obtained for material UI changes;
- tests/checks not run and why;
- security, privacy, authority, persistence, and inference-cost impact;
- dependency and lockfile impact;
- remaining acceptance debt;
- active-work/stack coordination when other implementation is in flight.

Green CI is not proof of the intended user outcome. The alpha is runnable when the real application has been built, exercised, restarted, failed safely, packaged, and reviewed on its primary platform.

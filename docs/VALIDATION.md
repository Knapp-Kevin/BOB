# B.O.B. Alpha Validation and Packaging

**Status:** Accepted alpha qualification contract  
**Product stage:** **Alpha Product**  
**Wayfinder:** #39  
**Primary platform:** Windows 11 x64

This document defines the evidence required to qualify B.O.B.'s current Alpha Product as a runnable, reviewable Windows alpha build and to support any alpha release claim. CI is deliberately small. The implementing developer or coding agent owns the stronger local, native, rendered, and failure-path evidence.

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

The current alpha package is a Windows 11 x64 NSIS installer.

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

When the local host cannot build or run unsigned release binaries (for example under Smart App Control), the owner may run `.github/workflows/windows-installer-qualification.yml` on a disposable hosted Windows runner. It runs only on a pushed `qualify/installer-*` branch or a manual dispatch, never on pull requests. It builds through `npm run package:windows`, installs the NSIS package silently, and runs `scripts/qualify-installed-windows.ps1`. That script covers launch, console absence, the PE checker, second-instance handoff, minimized restore and focus, simultaneous launches, persistence across restart and forced termination, a corrupt-database fail-closed start, and uninstall/reinstall without data loss. It changes no security policy and uploads evidence only, not the installer. Because it writes to the host's real B.O.B. profile, it refuses to run where a profile already exists. Never run it on a machine whose data matters.

Windows Sandbox is the local disposable host for that script. `scripts/new-windows-sandbox-qualification.ps1` refuses an installer whose SHA-256 does not match the expected value, then stages a kit and a `.wsb` configuration with networking, clipboard, printer, audio/video input and vGPU disabled. It maps only the kit input folder (read-only) and an evidence output folder, never a user profile. The Sandbox has no 7-Zip, so the kit takes the known SHA-256 of the installer's `bob.exe` payload instead of extracting it. Windows Sandbox must be enabled on the host by its owner; the kit changes no host security setting.

Where host Smart App Control blocks the unsigned release build tools, `scripts/new-windows-sandbox-build.ps1` stages a separate build Sandbox. It maps a read-only `git archive` of a committed ref and an output folder, and keeps clipboard, printer, audio/video input and vGPU off. It keeps networking on to download the toolchain. Windows Installer waits a fixed ~120 s per package inside Windows Sandbox, so when the host already has the MSVC Build Tools and the Windows 10/11 SDK, the kit maps those two folders read-only and the build uses them in place. Otherwise `scripts/build-in-windows-sandbox.ps1` installs the Windows SDK (desktop x64 only) and the MSVC compiler from the Build Tools (both Authenticode-checked), which takes over an hour. It then installs Rust 1.88.0 and Node 22 (checksum-checked) from their official sources and runs `npm ci` and `npm run package:windows`. It records the installer, payload and target `bob.exe` hashes in `build-manifest.json` for the offline qualification kit.

A signed release uses the same locked, targeted-clean path with the release-only config appended: `npm run package:windows -- --config src-tauri/tauri.release-windows.conf.json`. Only `.github/workflows/release-windows.yml` runs it; see [Signed releases](#signed-releases). Its acceptance adds: every signed file passes `scripts/verify-windows-signature.ps1`, the installed `bob.exe` is byte-identical to the signed build output, and the draft release carries the installer with its `.sha256`.

MSI is deferred for the current alpha. It duplicates the installer surface and adds WiX/VBScript requirements without a current product requirement.

macOS/Linux installers, cross-platform matrices, store publication, auto-update infrastructure, and broad release automation are deferred until a later release requires them.

## Code signing and release gate separation

A Windows signing certificate is **not a prerequisite** for compiling, packaging, locally installing, or qualifying a clearly labeled developer/controlled alpha. Treat these as separate outcomes:

1. **Unsigned native engineering acceptance:** build the exact locked NSIS package, record its SHA-256, and complete the Windows 11 x64 install/launch/relaunch/state-retention/uninstall evidence under issue #84. Report SmartScreen or publisher warnings as observed evidence; never silently bypass system policy or instruct users to disable security protections.
2. **Controlled alpha readiness:** after recovery issue #85 and native acceptance #84 are truthfully dispositioned, run the integrated alpha convergence audit #30. The unsigned build may be shared only in an explicitly controlled test setting with disclosed publisher/reputation warnings, verified artifact provenance, and an informed participant. This is not general distribution approval.
3. **Broad distribution readiness:** requires a signed release from the process below, plus the publisher reputation, installer provenance and distribution policy that apply. Signing does not itself establish application correctness, and its absence must not falsely block engineering qualification.

Never mark a native gate passed because signing is pending, and never mark native execution blocked *solely* because a signature is unavailable. If platform policy actually prevents unsigned execution, record the precise error and use a permitted, properly isolated test environment (such as the Windows Sandbox kits above) rather than weakening OS protections.

The readiness record must distinguish: `package built`, `native acceptance evidenced`, `controlled alpha qualified`, `release signed and verified`, and `broad distribution authorized`. Each claim requires its own evidence.

### Signed releases

B.O.B. follows `MythologIQ-Labs-LLC/Myth-Tech-Forge` `docs/WINDOWS_CODE_SIGNING_STANDARD.md`: distributed Windows binaries and installers are signed by **MythologIQ Labs, LLC** through Azure Artifact Signing, timestamped, and only for release builds.

- `npm run package:windows` and every development, PR, qualification and Sandbox build stay **unsigned** and need no signing tools. The default `src-tauri/tauri.conf.json` has no signing configuration.
- Release builds add `--config src-tauri/tauri.release-windows.conf.json`. Its `bundle.windows.signCommand` runs `scripts/sign-windows-release.ps1` for every file Tauri signs: `bob.exe`, the NSIS plugin DLLs, the uninstaller and the installer. The script takes the signing client from `BOB_SIGNING_CLIENT_DIR` and the Azure CLI sign-in, contains no machine-specific path, and fails the build when either is missing.
- `scripts/verify-windows-signature.ps1` is the release gate. A file passes only if `signtool verify /pa` succeeds, the Authenticode status is Valid, the signer is exactly `MythologIQ Labs, LLC`, the signature is timestamped, and the chain ends at Microsoft Identity Verification Root Certificate Authority 2020. Do not call a build signed until it passes.
- Signing in CI runs only in `.github/workflows/release-windows.yml`, for `v*` tags, after the owner approves the `release` environment. The procedure, version scheme and one-time owner setup are in `docs/PUBLIC_RELEASE.md`.

An unsigned build must not be published as a GitHub release or broadly distributed; Smart App Control hosts block it.

## Hosted CI boundary

Do not create a large hosted matrix.

The current diagnostic safety net is intentionally path-scoped and consumes committed lockfiles deterministically:

- frontend changes under `src/**` plus frontend manifest/lock/config changes run Node 22 `npm ci` followed by `npm run build`;
- Rust changes under `src-tauri/**` run Rust 1.88 format plus Clippy and tests with locked dependency resolution;
- material frontend changes also run the bounded rendered diagnostic in `.github/workflows/pr-rendered-ui-diagnostic.yml`, which serves the production Vite build and drives the hosted runner's existing Chrome/Chromium binary through the dependency-free `scripts/rendered-ui-diagnostic.mjs` Chrome DevTools Protocol harness.

The rendered diagnostic captures Today, Inbox, Chat, and Settings at the Tauri-configured normal `1400x900` and minimum `900x640` window sizes. It also exercises reduced-information mode, larger text, reduced motion, keyboard Tab progression, and the advanced Gemini setup dialog without using a provider credential. The job uploads screenshots, a machine-readable `audit.json`, and browser/Vite logs. It fails on missing route activation, document-level horizontal overflow, horizontally clipped focusable controls, a missing setup dialog, or broken keyboard focus progression.

A successful rendered diagnostic is legitimate **hosted Vite/browser presentation evidence**, not self-proving visual acceptance. The screenshots and audit artifact must still be inspected for hierarchy, dead space, accidental stacking, readability, and other qualitative defects the automated checks cannot judge. It is not Windows/Tauri evidence and cannot prove native lifecycle, persistence/restart, Credential Manager, recovery filesystem behavior, provider behavior, or packaging.

The build, Rust, and rendered jobs are useful execution evidence. They do not replace the stronger platform and product evidence required by the affected change.

Routine hosted CI does not replace Windows Credential Manager testing, restart/recovery testing, native rendered UX review where the Tauri boundary matters, installer verification, or live-provider evidence.

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

Green CI is not proof of the intended user outcome. The Alpha Product is release-qualified when the real application has been built, exercised, restarted, failed safely, packaged, and reviewed on its primary platform.

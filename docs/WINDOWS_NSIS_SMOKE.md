# Windows 11 x64 NSIS Native Smoke Runbook

**Authority:** `docs/VALIDATION.md`, issue #84  
**Purpose:** Collect the remaining native installer acceptance evidence without changing product or packaging behavior.

This runbook is intentionally manual and evidence-oriented. A successful hosted NSIS build proves package creation, not Windows install, launch, relaunch, uninstall, icon, or retained-user-data behavior.

The repository includes `scripts/windows-native-smoke-evidence.ps1` to capture deterministic machine-observed fields such as exact Git head, Windows build and architecture, installer hash, package-built executable hash, expected canonical-state location, the package-derived default NSIS installation target, process state, and post-uninstall retention. The helper does **not** automate or waive the manual observations for elevation, visible launch behavior, UI state survival, embedded-icon identity, uninstall success, shortcut/installer-owned artifact removal, or truthful uninstall messaging.

The helper is deliberately bound to the first-alpha packaging contract. It requires Windows 11 x64 and Tauri NSIS `currentUser` behavior. It derives the stock per-user target from the exact checkout's `productName` under `%LOCALAPPDATA%`, normalizes trailing periods/spaces using ordinary Windows path semantics, and fails closed if a platform-specific Tauri Windows config, custom NSIS template, installer hooks, `perMachine`, or `both` mode makes that derivation unsafe. For the current `productName` `B.O.B.`, the normalized target is expected to be `%LOCALAPPDATA%\B.O.B` rather than an operator-selected custom directory.

`-InstallPath` remains optional only as an assertion. If supplied during `package`, `installed`, or `uninstalled`, it must equal the derived default target. A custom install path does not satisfy #84.

The helper redacts local filesystem identity from the evidence file before it is written. Paths beneath the current profile are represented with roots such as `%LOCALAPPDATA%`, `%APPDATA%`, or `%USERPROFILE%`; other absolute paths are replaced with a redacted marker. Keep raw local paths only for commands that need them. Do not post usernames, home-directory names, checkout locations, or other workstation-specific absolute paths to the public issue.

The evidence/session sinks are also fail-closed. `-EvidencePath` must resolve outside the B.O.B. repository checkout and must not equal the local protected smoke-session path. The protected `%TEMP%\bob-windows-native-smoke-session.dpapi` path must itself also resolve outside the checkout, so a customized `%TEMP%` beneath the repository is rejected before any session state is written. After the exact Tauri acceptance targets are derived, both public evidence and protected session sinks must also remain outside B.O.B.'s canonical application-data directory and outside the package-derived default installation target. This prevents the helper from creating evidence or continuity files inside clean first-run state, managed recovery state, or the clean installation target before native acceptance begins. The helper additionally rejects a repository checkout, evidence sink, or protected-session sink whose existing path or ancestor chain contains a Windows reparse point. This prevents a junction or symbolic link from making a sink look lexically external while redirecting the actual write into repository state or another unintended location. The default `%TEMP%\bob-windows-native-smoke-evidence.md` and ordinary system `%TEMP%` session location satisfy this boundary. Do not redirect evidence or temporary session state through junctions/symbolic links, into the checkout, into B.O.B.'s application-data directory, or into the default install target.

The canonical application-data acceptance target is independently fail-closed against redirection. The helper requires `%APPDATA%\<identifier>\bob.sqlite3` to remain lexically beneath the current `%APPDATA%` root and rejects any existing component below that trusted profile root that is a Windows reparse point. The `%APPDATA%` root itself is treated as the trusted Windows profile boundary rather than being reclassified by this helper. This blocks an app-specific junction or symbolic link from redirecting canonical B.O.B. state elsewhere while preserving ordinary Windows profile-root behavior.

The package-derived default installation target receives the same app-specific protection. The helper requires the derived `%LOCALAPPDATA%\<productName>` target to remain beneath the current `%LOCALAPPDATA%` root and rejects any existing component below that trusted profile root that is a Windows reparse point. Once installed, `<default install path>\bob.exe` is independently checked before hashing and process evidence so the executable itself cannot be a symlink, junction-backed file, or other reparse-point redirect. The same executable identity and reparse check is repeated during relaunch before fresh-process evidence is accepted. This prevents a redirected or replaced executable from making install or relaunch provenance appear valid while the accepted payload actually changed between phases.

The `package` phase starts a fresh smoke session and stores a local-only continuity record in `%TEMP%\bob-windows-native-smoke-session.dpapi`. Its JSON payload is protected at rest with Windows DPAPI scoped to the current user before being written. Later phases must successfully unprotect and parse that record before they can validate continuity or append evidence; tampered, corrupt, empty, or different-user session data fails closed and requires a new package phase. The raw session values are never written to the public evidence file.

The executable identity check deliberately assumes the NSIS installation preserves the package-built `bob.exe` bytes. The first native execution must validate that assumption. If the installed or relaunched executable hash differs, do not weaken the check or reinterpret the run as passing. Leave #84 open, preserve both hashes, and determine whether NSIS legitimately transforms the payload before replacing this mechanism with an equivalent deterministic identity check.

Process evidence is also fail-closed. A genuine `Get-Process` result showing no `bob.exe` is distinct from a process-enumeration failure. If process enumeration fails, or if any discovered `bob.exe` cannot expose its executable path for exact install-target filtering, the helper treats process state as unknown and stops before evidence append or session advancement. Do not reinterpret an inspection failure as proof that zero bound B.O.B. processes remain.

## Preconditions

- Windows 11 x64 host.
- Clean checkout at the exact commit being accepted.
- Node 22 and Rust 1.88 or newer.
- Disposable test profile with no production credentials or sensitive user data.
- No platform-specific `src-tauri/tauri.windows.conf.json` or custom NSIS template/hooks unless #84 is first reconciled to an explicit effective-path policy.
- The exact canonical `%APPDATA%\<identifier>\bob.sqlite3` target must contain no filesystem object before installation. A regular file, directory, reparse-backed object, or any target whose absence cannot be proven fails the clean-profile precondition. Use another disposable profile; do not delete real user data or other existing state merely to satisfy this smoke test.
- The app-specific canonical application-data target beneath `%APPDATA%` must not be redirected through a junction or symbolic link.
- The package-derived default installation target beneath `%LOCALAPPDATA%` must not be redirected through a junction or symbolic link.
- The package-derived default installation target must be absent before installation. If it exists, use another disposable profile. Do not delete a real installation merely to make acceptance green.
- Public evidence and protected session sinks must be ordinary external temporary locations, not paths inside the checkout, B.O.B. application data, or the derived default install target.

Record the exact Git commit before building:

```powershell
git rev-parse HEAD
```

## 1. Build and bind package evidence

From the repository root:

```powershell
.\scripts\windows-native-smoke-evidence.ps1 -Phase package
```

The package phase:

1. verifies Windows 11 x64;
2. verifies the checkout is clean and records exact HEAD;
3. validates that the checkout, public evidence sink, and protected session sink do not traverse Windows reparse points, that both sinks are outside the checkout, and that the two sinks are distinct;
4. reads the exact Tauri config and requires the accepted stock `currentUser` NSIS path policy;
5. derives the normalized default per-user installation target under `%LOCALAPPDATA%` and the canonical application-data target under `%APPDATA%`;
6. verifies the canonical application-data target remains beneath `%APPDATA%` and rejects any existing app-specific path component below that trusted root when it is a Windows reparse point;
7. verifies the package-derived default installation target remains beneath `%LOCALAPPDATA%` and rejects any existing app-specific path component below that trusted root when it is a Windows reparse point;
8. rejects public evidence or protected session sinks inside either B.O.B.'s canonical application-data directory or the derived default installation target;
9. fails if the default installation target already exists;
10. fails closed unless the exact canonical `bob.sqlite3` target contains no filesystem object and its absence can be proven;
11. runs locked `npm ci`;
12. rechecks HEAD and repository cleanliness;
13. runs `npm run package:windows` through the accepted targeted-clean path;
14. rechecks HEAD and repository cleanliness;
15. requires exactly one generated NSIS installer and hashes it;
16. hashes the package-built `src-tauri\target\release\bob.exe`;
17. starts a fresh evidence file and DPAPI-protected smoke-session record bound to the package, host, profile, default target, evidence path, and evidence digest.

If you want an additional assertion, this is valid only when the path equals the derived default target:

```powershell
.\scripts\windows-native-smoke-evidence.ps1 -Phase package -InstallPath "$env:LOCALAPPDATA\B.O.B"
```

Do not choose a different empty directory merely because the installer permits it. #84 specifically requires the default per-user path.

Record:

- exact commit SHA;
- Windows 11 version/build and x64 architecture;
- verified NSIS install mode `currentUser`;
- package-derived default installation path and proof it was absent before installation;
- installer filename and SHA-256;
- package-built `bob.exe` SHA-256.

## 2. Install with default per-user behavior

Launch the exact produced NSIS installer normally from Explorer or PowerShell. Do not use `Run as administrator` unless the installer itself explicitly requires elevation. Accept the installer's default path. If the installer presents or uses a path different from the helper-derived target, stop and leave #84 open rather than relabeling the session.

Record:

- whether Windows requested elevation;
- the default path shown/used by the installer;
- whether installation completed successfully;
- any SmartScreen or unsigned-alpha warning separately from installer failure.

An unsigned alpha may surface Windows reputation warnings. That is not equivalent to installation failure, but the warning must not be omitted from evidence.

## 3. Launch and capture the installed snapshot

Launch installed B.O.B. from the created shortcut or executable. While B.O.B. remains running, confirm:

1. the application window appears;
2. Today, Inbox, Chat, and Settings are reachable;
3. a simple non-sensitive test item can be captured or changed.

B.O.B. resolves canonical state through Tauri `app_data_dir()`. The helper reads the bundle identifier from the exact checkout and checks the corresponding `%APPDATA%\<identifier>\bob.sqlite3` rather than recursively searching the profile. The helper rechecks the app-specific canonical path and the package-derived default install target for reparse-point redirection on every phase before using either path as evidence.

Capture machine evidence while the application remains running:

```powershell
.\scripts\windows-native-smoke-evidence.ps1 -Phase installed -InstallerPath '<installer file used for installation>'
```

The installed phase requires the same package session and derived default target. It verifies that the **supplied installer file** matches the package-phase SHA-256, rejects `<default install path>\bob.exe` if that executable itself is reparse-backed, verifies the ordinary installed executable against the package-built executable hash, requires exactly one `bob.exe` process from that target, requires canonical state to exist outside the install directory, and recursively verifies that no `bob.sqlite3` exists beneath the installation directory. That recursive absence check is fail-closed: if any part of the installation tree cannot be enumerated, the helper stops and leaves #84 open rather than treating an incomplete scan as proof of absence. Process discovery is likewise fail-closed: enumeration or executable-path inspection failure is unknown evidence and cannot satisfy the sole-process requirement. The helper does not observe which installer process was actually executed, so actual installer execution remains part of the manual native observation in step 2.

If the installed executable is reparse-backed or its hash differs from the package-built executable hash, preserve the observed non-secret details and leave #84 open. Do not replace the failed deterministic check with an operator assertion.

After the installed snapshot succeeds, quit B.O.B. normally and verify the process exits.

## 4. Quit and relaunch

Verify B.O.B. is fully stopped, then relaunch it from the installed application.

```powershell
Get-Process bob -ErrorAction SilentlyContinue
```

Confirm the previously created test state is still visible. While the relaunched app remains running:

```powershell
.\scripts\windows-native-smoke-evidence.ps1 -Phase relaunched
```

Before accepting process evidence, the helper revalidates `<default install path>\bob.exe` against the package-built executable SHA-256 and rejects file-level reparse redirection. It then requires exactly one `bob.exe` from the bound default installation target, with a different PID and later start time than the installed snapshot, plus the same canonical-state file. This rejects an executable replaced or redirected between phases, a never-quit original process, and duplicate concurrent instances. If executable identity, process enumeration, or executable-path inspection fails, relaunch evidence fails closed rather than treating the process state as acceptable. UI-visible state survival remains a manual product observation.

## 5. Verify embedded executable icon identity

Verify the installed executable's embedded icon independently from Explorer's cached presentation. At minimum:

1. inspect the installed executable in Windows Properties or another local resource-inspection method that reads the executable resource;
2. compare it with the current canonical application icon described in `docs/assets/README.md`;
3. if Explorer shows a stale icon, clear/rebuild shell icon cache or verify the executable resource through a second method before diagnosing a packaging failure.

Record the method and result. A cached shell thumbnail alone is not sufficient evidence either way.

## 6. Uninstall

Quit B.O.B. completely, then use the normal Windows uninstall entry created by NSIS. Do not manually delete the installation directory as a substitute for uninstall acceptance.

Record:

- whether uninstall completed successfully;
- whether installed files/shortcuts were removed;
- whether zero `bob.exe` processes from the accepted installation remain running;
- whether retained user data still exists;
- whether any uninstall UI/message falsely claimed user data would be removed.

Immediately after uninstall:

```powershell
.\scripts\windows-native-smoke-evidence.ps1 -Phase uninstalled
```

The helper fails closed unless zero bound `bob.exe` processes remain, the package-derived default installation directory is gone, and the exact canonical state file remains. Process absence is accepted only when process enumeration succeeds and every discovered `bob.exe` can be inspected for exact install-target filtering. An enumeration/path-inspection failure leaves process state unknown and therefore leaves #84 open. `-InstallPath` may be supplied only as a matching assertion, not to redirect this check to some unrelated missing directory.

## 7. Evidence record

Attach or record the following on #84 before closing it. By default the helper writes machine-observed rows to `%TEMP%\bob-windows-native-smoke-evidence.md`. The local `%TEMP%\bob-windows-native-smoke-session.dpapi` continuity record is DPAPI-protected private state, not public evidence.

| Evidence | Result |
| --- | --- |
| Exact commit SHA | |
| Windows 11 version/build | |
| Windows architecture | `X64` |
| Acceptance platform verified | `Windows 11 x64` |
| NSIS install mode verified | `currentUser` |
| Package build path | `npm ci` + `npm run package:windows` |
| Package-derived default install path | |
| Default install target absent before install | |
| Installer filename | |
| Installer SHA-256 | |
| Package-built bob.exe SHA-256 | |
| Supplied installer matches package-phase SHA-256 | |
| Installed bob.exe is ordinary (not reparse-backed) | |
| Installed bob.exe matches package-built SHA-256 | |
| Install path matches package-derived default target | |
| Expected canonical state absent before install | |
| Elevation requested | |
| Install result | |
| First launch result | |
| Installed B.O.B. process from default install path | |
| Expected canonical state path | |
| Expected canonical state exists | |
| Database absent from install directory | |
| Quit/relaunch result | |
| Relaunched bob.exe matches package-built SHA-256 | |
| Fresh B.O.B. process observed after relaunch | |
| State survived relaunch | |
| Embedded-icon verification method/result | |
| Uninstall result | |
| Installed files removed | |
| No B.O.B. process remains after uninstall | |
| User data retained | |
| Uninstall messaging truthful | |

## Failure handling

A failure in any acceptance step leaves #84 open. Preserve the exact installer, commit SHA, Windows build, observed behavior, and relevant non-secret logs before changing packaging code.

If the helper reports a non-Windows-11-x64 host, unsafe or reparse-point-backed repository/evidence/session path, a reparse-backed app-specific canonical application-data target below `%APPDATA%`, a reparse-backed package-derived default install target below `%LOCALAPPDATA%`, a reparse-backed installed or relaunched `bob.exe`, evidence/session overlap with B.O.B. application data or the default install target, unsupported NSIS mode/path customization, platform-specific Windows config, any pre-existing filesystem object at the exact canonical `bob.sqlite3` target, or a pre-existing default installation target, do not override it. Reconcile the effective package policy if necessary, otherwise use a clean Windows 11 x64 disposable profile and restart at `package`.

If the installed-phase recursive installation-tree scan cannot enumerate every subtree, database absence has not been proven. Do not suppress the error, manually mark the evidence row true, or advance the smoke session. Investigate the unreadable path and start again at `package` once the environment can be checked completely.

If process enumeration fails, or any discovered `bob.exe` cannot expose its executable path for exact install-target filtering, process state is unknown. Do not manually reinterpret that failure as zero matching processes. Restore a fully inspectable environment and start again at `package` before recording process-presence or process-absence acceptance evidence.

If a later phase reports a protected smoke-session, evidence-digest, default-target, installer-hash, installed/relaunched-executable-hash, process, canonical-state, install-directory, or uninstall-retention failure, do not splice rows together manually. Fix or understand the native failure, then start again at `package` for the exact package/environment under acceptance.
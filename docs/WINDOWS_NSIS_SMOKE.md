# Windows 11 x64 NSIS Native Smoke Runbook

**Authority:** `docs/VALIDATION.md`, issue #84  
**Purpose:** Collect the remaining native installer acceptance evidence without changing product or packaging behavior.

This runbook is intentionally manual and evidence-oriented. A successful hosted NSIS build proves package creation, not Windows install, launch, relaunch, uninstall, icon, or retained-user-data behavior.

## Preconditions

- Windows 11 x64 host.
- Clean checkout at the exact commit being accepted.
- Node 22 and Rust 1.88 or newer.
- No production credentials or sensitive user data in the test profile.
- Record the exact Git commit before building:

```powershell
git rev-parse HEAD
```

## 1. Build the locked targeted-clean installer

From the repository root:

```powershell
npm ci
npm run package:windows
```

Record:

- exact commit SHA;
- Windows version from `winver` or `Get-ComputerInfo`;
- produced installer filename;
- installer SHA-256:

```powershell
Get-FileHash .\src-tauri\target\release\bundle\nsis\*.exe -Algorithm SHA256
```

Do not substitute an older installer merely because its filename matches.

## 2. Install with default per-user behavior

Launch the produced NSIS installer normally from Explorer or PowerShell. Do not use `Run as administrator` unless the installer itself explicitly requires elevation.

Record:

- whether Windows requested elevation;
- selected/default install path;
- whether installation completed successfully;
- any SmartScreen or unsigned-alpha warning separately from installer failure.

An unsigned local/developer alpha may surface Windows reputation warnings. That is not equivalent to an installation failure, but the warning must not be omitted from evidence.

## 3. Launch and create canonical state

Launch the installed B.O.B. application from the installed shortcut or executable.

Confirm:

1. the application window appears;
2. Today, Inbox, Chat, and Settings are reachable;
3. a simple non-sensitive test item can be captured or changed;
4. the application can quit normally.

B.O.B. resolves canonical state through Tauri's application data directory, not the install directory. After creating test state, verify that `bob.sqlite3` exists under the current user's application-data tree and that no canonical database was created beside the installed executable.

Useful inspection commands:

```powershell
Get-ChildItem $env:APPDATA,$env:LOCALAPPDATA -Filter bob.sqlite3 -Recurse -ErrorAction SilentlyContinue |
  Select-Object FullName,Length,LastWriteTime
```

Then inspect the installation directory separately:

```powershell
Get-ChildItem '<recorded install path>' -Filter bob.sqlite3 -Recurse -ErrorAction SilentlyContinue
```

Acceptance requires canonical user state to be outside the install directory. Record the actual discovered path rather than assuming a particular Tauri directory layout.

## 4. Quit and relaunch

Quit B.O.B. completely, verify the process exits, and relaunch the installed application.

```powershell
Get-Process bob -ErrorAction SilentlyContinue
```

Confirm the previously created test state is still present after relaunch. Record launch and state-survival results separately.

## 5. Verify packaged executable icon identity

The package must embed the current B.O.B. application icon resources generated from the canonical framed application identity.

Verify the installed executable's embedded icon independently from Explorer's cached presentation. At minimum:

1. inspect the installed executable in Windows Properties or another local resource-inspection method that reads the executable resource;
2. compare it with the current canonical application icon described in `docs/assets/README.md`;
3. if Explorer shows a stale icon, clear/rebuild shell icon cache or verify the executable resource through a second method before diagnosing a packaging failure.

Record the method used and the result. A cached shell thumbnail alone is not sufficient evidence either way.

## 6. Uninstall

Use the normal Windows uninstall entry created by NSIS. Do not manually delete the installation directory as a substitute for uninstall acceptance.

Record:

- whether uninstall completed successfully;
- whether the installed application files/shortcuts were removed;
- whether the retained user-data directory still exists;
- whether any uninstall UI or message falsely claimed user data would be removed.

Current accepted behavior intentionally does not promise deletion of local user data during uninstall. Retained user data is therefore expected unless product authority later changes that contract.

## 7. Evidence record

Attach or record the following on issue #84 before closing it:

| Evidence | Result |
| --- | --- |
| Exact commit SHA | |
| Windows 11 version/build | |
| Installer filename | |
| Installer SHA-256 | |
| Elevation requested | |
| Install path | |
| Install result | |
| First launch result | |
| Canonical state path | |
| Database absent from install directory | |
| Quit/relaunch result | |
| State survived relaunch | |
| Embedded-icon verification method/result | |
| Uninstall result | |
| Installed files removed | |
| User data retained | |
| Uninstall messaging truthful | |

## Failure handling

A failure in any acceptance step leaves issue #84 open. Preserve the exact installer, commit SHA, Windows build, observed behavior, and relevant non-secret logs before changing packaging code. Do not blur a source-level fix, hosted package rebuild, and a repeated native smoke into one evidence claim. Each materially changed package head requires its own native acceptance record.

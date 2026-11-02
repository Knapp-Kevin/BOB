# Public Repository Maintenance

B.O.B. is a public, MIT-licensed repository. This document preserves the small set of public-repository hygiene checks that matter without turning the project into an administrative hobby.

## Public baseline

The repository should maintain:

- a truthful README that distinguishes implemented capabilities from planned work;
- an explicit open-source license;
- contribution, conduct, support, and security guidance;
- structured issue and pull-request intake;
- a clear private route for vulnerability reports;
- a deliberately small active tree;
- no committed secrets, private user data, generated model weights, or unexplained third-party binaries;
- minimal CI, with implementation-time validation owned by the developer or coding agent.

## Before a supported release

Before publishing the first supported release candidate:

1. verify install/build instructions on the supported platform;
2. define the supported-version statement in `.github/SECURITY.md`;
3. review active dependencies and bundled assets for license and provenance requirements;
4. verify the private vulnerability-reporting path;
5. ensure release artifacts contain no credentials, personal data, debug logs, or unintended local state;
6. verify README status, screenshots, badges, and architecture match what actually ships;
7. confirm the one-time signing setup under [Signed Windows releases](#signed-windows-releases) is still in place, so the installer is signed and verified.

## Signed Windows releases

Every Windows installer B.O.B. distributes is Authenticode-signed by **MythologIQ Labs, LLC** through Azure Artifact Signing, following `MythologIQ-Labs-LLC/Myth-Tech-Forge` `docs/WINDOWS_CODE_SIGNING_STANDARD.md`. Only release builds are signed. Development, PR and qualification builds stay unsigned and need no signing tools.

### Version scheme

Releases use SemVer core versions, tagged `vMAJOR.MINOR.PATCH` (for example `v0.1.0`), with no pre-release suffix. While B.O.B. is an Alpha Product the major version stays `0`: MINOR rises for a release that changes user-facing capability, PATCH for a fix-only release. Every `0.x` release is marked as a GitHub pre-release. The first signed release is `v0.1.0`, an alpha pre-release.

The tag must equal the version in all of `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and the `bob` entry of `src-tauri/Cargo.lock`. `scripts/check-release-version.mjs` enforces this before anyone is asked to approve signing, and again on the release runner.

### Cutting a release

1. In an ordinary PR, set the new version in every manifest and refresh the lockfiles without touching dependencies:

   ```powershell
   npm version 0.1.0 --no-git-tag-version
   # edit "version" in src-tauri/tauri.conf.json and src-tauri/Cargo.toml to 0.1.0
   cargo update --manifest-path src-tauri/Cargo.toml --workspace
   node scripts/check-release-version.mjs v0.1.0
   ```

2. Validate the release candidate under `docs/VALIDATION.md` (including installed-app qualification) and merge the PR.
3. Tag the merged commit and push only that tag:

   ```powershell
   git tag -a v0.1.0 -m "B.O.B. 0.1.0"
   git push origin v0.1.0
   ```

4. `.github/workflows/release-windows.yml` checks the tag against the manifests on Linux, then waits for the owner to approve the `release` environment. That approval is the signing gate: no Azure sign-in or signature happens before it.
5. After approval the Windows job runs `npm ci`, then the locked, targeted-clean `npm run package:windows -- --config src-tauri/tauri.release-windows.conf.json`. The release-only config sets `bundle.windows.signCommand` to `scripts/sign-windows-release.ps1`, so Tauri signs `bob.exe`, the NSIS plugin DLLs, the uninstaller and the installer, each timestamped. The script fails closed when the signing client or Azure sign-in is missing.
6. The job installs the package on the disposable runner and runs `scripts/verify-windows-signature.ps1` on the installer, the installed `bob.exe`, the installed `uninstall.exe` and the five NSIS plugin DLLs Tauri signs and embeds (`NSISdl`, `StartMenu`, `System`, `nsDialogs`, `nsis_tauri_utils`). Each must pass `signtool verify /pa`, have a Valid Authenticode status, be signed by exactly `MythologIQ Labs, LLC`, be timestamped and chain to Microsoft Identity Verification Root Certificate Authority 2020. Any failure stops the release. `target/release/bob.exe` is not checked: Tauri patches and signs it for the bundle, then restores the unsigned build, so only its hash is recorded. The output is kept as the `windows-release-evidence-<tag>` workflow artifact.
7. The job writes `<installer>.sha256` and creates a **draft** GitHub Release with both files. The owner reviews the draft, downloads and checks the installer, then publishes it. Nothing is published automatically.

Re-running a failed release counts against the shared Actions budget (Windows minutes bill at 2x): re-run at most once, then reproduce locally.

### One-time owner setup

The owner approved and completed this setup on 2026-11-02: the `bob-release-environment` federated credential, the `release` environment (owner as required reviewer, `v*` tags only) and the three environment variables all exist. The commands below record what was done and rebuild it if needed. Only the owner makes these changes; they need Azure rights on the `MythologIQ` tenant and admin rights on `Knapp-Kevin/BOB`. Run them in PowerShell with `az` and `gh` signed in.

1. **Federated credential.** Add one to the existing `github-sonic-rift-signing` app registration, which already holds the Artifact Signing Certificate Profile Signer role on `mythologiq-labs-public-release`. This repository's OIDC tokens use GitHub's immutable subject format (owner and repository IDs), so the subject is `repo:Knapp-Kevin@205245245/BOB@956289694:environment:release`, not the older `repo:Knapp-Kevin/BOB:environment:release`. Confirm the prefix first with `gh api repos/Knapp-Kevin/BOB/actions/oidc/customization/sub`.

   ```powershell
   $appId = az ad app list --display-name github-sonic-rift-signing --query "[0].appId" -o tsv
   $file = Join-Path $env:TEMP 'bob-release-federated-credential.json'
   $json = '{"name":"bob-release-environment","issuer":"https://token.actions.githubusercontent.com","subject":"repo:Knapp-Kevin@205245245/BOB@956289694:environment:release","audiences":["api://AzureADTokenExchange"],"description":"Knapp-Kevin/BOB release environment (v* tags, owner approval)"}'
   Set-Content -LiteralPath $file -Value $json -Encoding ascii
   az ad app federated-credential create --id $appId --parameters "@$file"
   az ad app federated-credential list --id $appId --query "[].{name:name,subject:subject}" -o table
   ```

   A separate app registration with its own Signer role assignment scoped to the profile is the alternative the standard allows; it needs its own role assignment and its own client ID below.

2. **`release` environment** with the owner as required reviewer, deployable only from `v*` tags:

   ```powershell
   # A file, not a pipe: Windows PowerShell 5.1 mangles JSON piped to gh's stdin.
   $file = Join-Path $env:TEMP 'bob-release-environment.json'
   Set-Content -LiteralPath $file -Encoding ascii -Value '{"prevent_self_review":false,"reviewers":[{"type":"User","id":205245245}],"deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true}}'
   gh api -X PUT repos/Knapp-Kevin/BOB/environments/release --input $file
   gh api -X POST repos/Knapp-Kevin/BOB/environments/release/deployment-branch-policies -f name='v*' -f type=tag
   ```

3. **Azure variables.** `AZURE_CLIENT_ID` is the app registration's application (client) ID, `AZURE_TENANT_ID` the MythologIQ tenant, and `AZURE_SUBSCRIPTION_ID` the subscription that holds the `MythologIQ-Labs-Signing` account. They are identifiers, not secrets. As in `Knapp-Kevin/sonic-rift`, they live on the `release` environment so only approved release jobs see them; the workflow reads them as `vars.*`, so repository-level variables would also work. Copy them from Sonic Rift without printing them:

   ```powershell
   foreach ($name in 'AZURE_CLIENT_ID', 'AZURE_TENANT_ID', 'AZURE_SUBSCRIPTION_ID') {
     $value = gh variable get $name -R Knapp-Kevin/sonic-rift -e release
     gh variable set $name -R Knapp-Kevin/BOB -e release --body $value
   }
   gh variable list -R Knapp-Kevin/BOB -e release
   ```

No client secret, certificate or password is stored anywhere. The workflow signs in with `azure/login` using GitHub OIDC, and the signing metadata allows only the Azure CLI credential.

## Repository history

The public repository contains historical pre-revival material in older Git history and the named archive branch. That includes retired prototype code and artifacts that are not part of the active product.

Historical material is not supported software and is not an architectural authority. If a secret, private document, or asset with incompatible redistribution terms is identified in history, treat that as a remediation task rather than assuming deletion from `master` was sufficient.

## GitHub settings

Keep repository settings proportionate to the project:

- Issues enabled;
- Wiki and Discussions only if there is a real use for them;
- private vulnerability reporting enabled when available;
- secret scanning or push protection enabled when available;
- Dependabot security alerts enabled once the revived dependency graph exists;
- default-branch safeguards without mandatory expensive CI;
- GitHub Actions limited to checks that provide enough value to justify their cost.

## Principle

Public-repository hygiene should help people understand, use, contribute to, and safely report problems with B.O.B. It should not become a second product that needs its own project manager.

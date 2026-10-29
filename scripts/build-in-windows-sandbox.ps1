# Runs INSIDE a disposable Windows Sandbox staged by new-windows-sandbox-build.ps1.
# Installs the MSVC Build Tools, Rust and Node from their official sources (verifying
# signatures or published checksums), builds the NSIS installer through
# npm run package:windows, and writes the installer, hashes and logs to C:\out.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$RustToolchain = '1.88.0'
$NodeMajor = 'v22'
$Out = 'C:\out'
$Log = Join-Path $Out 'build.log'
$Downloads = 'C:\dl'
New-Item -ItemType Directory -Force $Downloads | Out-Null

function Note([string] $Text) { "[$([DateTime]::UtcNow.ToString('HH:mm:ss'))] $Text" | Add-Content -LiteralPath $Log }
function Run([string] $CommandLine) {
    Note "> $CommandLine"
    cmd /c "$CommandLine >> `"$Log`" 2>&1"
    if ($LASTEXITCODE -ne 0) { throw "'$CommandLine' failed with exit $LASTEXITCODE" }
}
function Fetch([string] $Url, [string] $Name) {
    $path = Join-Path $Downloads $Name
    Note "download $Url"
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $path
    $path
}

try {
    $os = Get-CimInstance Win32_OperatingSystem
    $sac = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
    Note "build sandbox: $($os.Caption) $($os.Version) build $($os.BuildNumber); Smart App Control state=$sac"
    Note "source: $((Get-Content C:\kit\source.txt -Raw).Trim())"

    $vs = Fetch 'https://aka.ms/vs/17/release/vs_BuildTools.exe' 'vs_BuildTools.exe'
    $sig = Get-AuthenticodeSignature -LiteralPath $vs
    if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') { throw "vs_BuildTools.exe signature is $($sig.Status) by $($sig.SignerCertificate.Subject)" }
    Note "installing MSVC Build Tools (signed by $($sig.SignerCertificate.Subject))"
    $p = Start-Process -FilePath $vs -ArgumentList '--quiet', '--wait', '--norestart', '--nocache', '--add', 'Microsoft.VisualStudio.Workload.VCTools', '--includeRecommended' -Wait -PassThru
    if ($p.ExitCode -notin 0, 3010) { throw "Build Tools installer exited $($p.ExitCode)" }

    $rustup = Fetch 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' 'rustup-init.exe'
    $expected = ((Invoke-WebRequest -UseBasicParsing 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe.sha256').Content -split '\s+')[0]
    if ((Get-FileHash -LiteralPath $rustup).Hash -ne $expected.ToUpperInvariant()) { throw 'rustup-init.exe checksum mismatch' }
    Run "`"$rustup`" -y --profile minimal --default-toolchain $RustToolchain --no-modify-path"
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

    $node = (Invoke-RestMethod 'https://nodejs.org/dist/index.json' | Where-Object { $_.version -like "$NodeMajor.*" } | Select-Object -First 1).version
    $zipName = "node-$node-win-x64.zip"
    $zip = Fetch "https://nodejs.org/dist/$node/$zipName" $zipName
    $sums = (Invoke-WebRequest -UseBasicParsing "https://nodejs.org/dist/$node/SHASUMS256.txt").Content
    $expected = ($sums -split "`n" | Where-Object { $_ -match "\s$([regex]::Escape($zipName))$" }) -split '\s+' | Select-Object -First 1
    if ((Get-FileHash -LiteralPath $zip).Hash -ne $expected.ToUpperInvariant()) { throw "$zipName checksum mismatch" }
    Expand-Archive -LiteralPath $zip -DestinationPath C:\node
    $env:PATH = "C:\node\node-$node-win-x64;$env:PATH"

    robocopy C:\kit\src C:\build /E /NFL /NDL /NJH /NJS | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "copying the source failed (robocopy $LASTEXITCODE)" }
    Set-Location C:\build
    Run 'rustc -V && cargo -V && node -v && npm -v'
    Run 'npm ci'
    Run 'npm run package:windows'

    $installer = Get-ChildItem 'C:\build\src-tauri\target\release\bundle\nsis' -Filter '*-setup.exe' | Select-Object -First 1
    if (-not $installer) { throw 'no NSIS installer was produced' }
    Copy-Item -LiteralPath $installer.FullName -Destination $Out
    $installerHash = (Get-FileHash -LiteralPath $installer.FullName).Hash
    $targetHash = (Get-FileHash -LiteralPath 'C:\build\src-tauri\target\release\bob.exe').Hash

    # The payload hash lets the offline qualification Sandbox (no 7-Zip) check the
    # installed copy. This build Sandbox is disposable, so installing here is safe.
    $p = Start-Process -FilePath $installer.FullName -ArgumentList '/S' -Wait -PassThru
    $payload = Join-Path $env:LOCALAPPDATA 'B.O.B\bob.exe'
    if ($p.ExitCode -ne 0 -or -not (Test-Path $payload)) { throw "payload install exited $($p.ExitCode); bob.exe present=$(Test-Path $payload)" }
    $payloadHash = (Get-FileHash -LiteralPath $payload).Hash

    [ordered]@{
        source = (Get-Content C:\kit\source.txt -Raw).Trim()
        sandbox_os = "$($os.Caption) $($os.Version) build $($os.BuildNumber)"
        smart_app_control_state = $sac
        rust = $RustToolchain; node = $node
        installer = $installer.Name; installer_sha256 = $installerHash
        payload_bob_exe_sha256 = $payloadHash; target_bob_exe_sha256 = $targetHash
        installer_signature = (Get-AuthenticodeSignature -LiteralPath $installer.FullName).Status.ToString()
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'build-manifest.json') -Encoding utf8
    Note "PASS installer=$installerHash payload=$payloadHash"
    'PASS' | Set-Content -LiteralPath (Join-Path $Out 'DONE')
} catch {
    Note "FAIL: $($_.Exception.Message)"
    "FAIL: $($_.Exception.Message)" | Set-Content -LiteralPath (Join-Path $Out 'DONE')
}

# Runs INSIDE a disposable Windows Sandbox staged by new-windows-sandbox-build.ps1.
# Installs the Windows SDK, MSVC Build Tools, Rust and Node from official sources (verifying
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

    if (Test-Path 'C:\toolchain\msvc\bin\Hostx64\x64\link.exe') {
        # Every MSI waits a fixed ~120 s inside Windows Sandbox (Windows Installer's own
        # software restriction policy check; plain WinVerifyTrust takes 1 s), which made the
        # SDK and Build Tools installs take over an hour. When the host's MSVC and Windows SDK
        # are mapped read-only, use them in place instead of installing anything.
        $sdkRoot = 'C:\toolchain\winsdk'
        $sdkVer = (Get-ChildItem "$sdkRoot\Lib" -Directory | Where-Object { Test-Path "$($_.FullName)\um\x64\kernel32.lib" } | Sort-Object Name | Select-Object -Last 1).Name
        if (-not $sdkVer) { throw "no x64 Windows SDK libraries under $sdkRoot" }
        $msvc = 'C:\toolchain\msvc'
        $env:VCINSTALLDIR = 'C:\toolchain\'
        $env:VCToolsInstallDir = "$msvc\"
        $env:PATH = "$msvc\bin\Hostx64\x64;$sdkRoot\bin\$sdkVer\x64;$env:PATH"
        $env:LIB = "$msvc\lib\x64;$sdkRoot\Lib\$sdkVer\um\x64;$sdkRoot\Lib\$sdkVer\ucrt\x64"
        $env:INCLUDE = (@("$msvc\include") + ('ucrt', 'um', 'shared', 'winrt', 'cppwinrt' | ForEach-Object { "$sdkRoot\Include\$sdkVer\$_" })) -join ';'
        # Resource compilers locate the SDK through this key; set it in the Sandbox only.
        foreach ($key in 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots', 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows Kits\Installed Roots') {
            New-Item -Force $key | Out-Null
            New-ItemProperty $key -Name KitsRoot10 -Value "$sdkRoot\" -Force | Out-Null
        }
        Note "using host toolchain mapped read-only: $((Get-Content C:\kit\toolchain.txt -Raw).Trim()); SDK libraries $sdkVer"
        $banner = (& link.exe /? 2>&1 | Select-Object -First 1 | Out-String).Trim()
        if ($banner -notmatch 'Linker Version') { throw "link.exe from the mapped toolchain did not run: $banner" }
        Note $banner
    } else {
        # The Windows SDK installer launched by the Build Tools stalled indefinitely in Windows
        # Sandbox, so install the SDK first, standalone, with only the desktop x64 C++ feature.
        # 10.0.26100.9457 installer link from learn.microsoft.com/windows/apps/windows-sdk/downloads.
        $sdk = Fetch 'https://go.microsoft.com/fwlink/?linkid=2382321' 'winsdksetup.exe'
        $sig = Get-AuthenticodeSignature -LiteralPath $sdk
        if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') { throw "winsdksetup.exe signature is $($sig.Status) by $($sig.SignerCertificate.Subject)" }
        Note "installing Windows SDK $((Get-Item $sdk).VersionInfo.ProductVersion) (DesktopCPPx64 only)"
        $p = Start-Process -FilePath $sdk -ArgumentList '/features', 'OptionId.DesktopCPPx64', '/quiet', '/norestart', '/ceip', 'off', '/log', (Join-Path $Out 'winsdk.log') -PassThru
        # Each SDK MSI package takes a fixed ~2 minutes inside Windows Sandbox, and about 24 run.
        if (-not $p.WaitForExit(90 * 60 * 1000)) { throw 'Windows SDK install exceeded 90 minutes; see winsdk.log' }
        if ($p.ExitCode -notin 0, 3010) { throw "Windows SDK installer exited $($p.ExitCode); see winsdk.log" }
        $kernel32 = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Lib" -Recurse -Filter kernel32.lib -ErrorAction SilentlyContinue | Where-Object FullName -match '\\x64\\' | Select-Object -First 1
        if (-not $kernel32) { throw 'Windows SDK installed without the x64 desktop libraries' }
        Note "Windows SDK ready: $($kernel32.FullName)"

        $vs = Fetch 'https://aka.ms/vs/17/release/vs_BuildTools.exe' 'vs_BuildTools.exe'
        $sig = Get-AuthenticodeSignature -LiteralPath $vs
        if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') { throw "vs_BuildTools.exe signature is $($sig.Status) by $($sig.SignerCertificate.Subject)" }
        $admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        Note "installing MSVC Build Tools (signed by $($sig.SignerCertificate.Subject)); elevated=$admin"
        $p = Start-Process -FilePath $vs -ArgumentList '--quiet', '--wait', '--norestart', '--nocache', '--add', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64' -PassThru
        # Heartbeat and installer logs, so a stalled install is visible from the host.
        $vsRoot = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio'
        $deadline = (Get-Date).AddMinutes(90)
        while (-not $p.WaitForExit(60000)) {
            $mb = [int]((Get-ChildItem $vsRoot -Recurse -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum / 1MB)
            $procs = (Get-Process | Where-Object { $_.Name -match '^(vs_|setup$|vs_installer)' } | ForEach-Object Name | Sort-Object -Unique) -join ','
            Note "Build Tools: $mb MB under $vsRoot; installer processes: $procs"
            Get-ChildItem $env:TEMP -Filter 'dd_*.log' -ErrorAction SilentlyContinue | Copy-Item -Destination $Out -Force
            if ((Get-Date) -gt $deadline) { throw 'Build Tools install exceeded 90 minutes' }
        }
        Get-ChildItem $env:TEMP -Filter 'dd_*.log' -ErrorAction SilentlyContinue | Copy-Item -Destination $Out -Force
        if ($p.ExitCode -notin 0, 3010) { throw "Build Tools installer exited $($p.ExitCode)" }
    }

    $rustup = Fetch 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' 'rustup-init.exe'
    # PowerShell 5.1 returns this octet-stream body as bytes, so read it from a file.
    $sumFile = Fetch 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe.sha256' 'rustup-init.exe.sha256'
    $expected = ((Get-Content -LiteralPath $sumFile -Raw).Trim() -split '\s+')[0]
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

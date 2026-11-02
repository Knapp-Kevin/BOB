# Stages an offline Windows Sandbox kit that runs qualify-installed-windows.ps1 against a
# verified B.O.B. NSIS installer. The Sandbox is the disposable host that script needs:
# networking, clipboard, printers, audio/video input and vGPU are off, and only the kit
# input (read-only) and evidence output folders are mapped. Run it non-elevated; it
# changes no host security setting and does not launch the Sandbox.
param(
    [Parameter(Mandatory = $true)] [string] $Installer,
    [Parameter(Mandatory = $true)] [string] $ExpectedInstallerSHA256,
    # SHA-256 of the bob.exe inside the installer; the Sandbox has no 7-Zip to extract it.
    [Parameter(Mandatory = $true)] [string] $ExpectedPayloadSHA256,
    [string] $KitRoot = 'C:\qual\sandbox-kit',
    [string] $PythonHome = (Split-Path (Get-Command python -ErrorAction Stop).Source)
)
$ErrorActionPreference = 'Stop'

$actual = (Get-FileHash -LiteralPath $Installer).Hash
if ($actual -ne $ExpectedInstallerSHA256.ToUpperInvariant()) {
    throw "installer sha256 $actual does not match the expected $ExpectedInstallerSHA256; refusing to stage it"
}

$kitInput = Join-Path $KitRoot 'input'
$evidence = Join-Path $KitRoot 'evidence'
foreach ($dir in $kitInput, $evidence) {
    $full = [IO.Path]::GetFullPath($dir)
    foreach ($forbidden in $env:APPDATA, $env:LOCALAPPDATA, $env:USERPROFILE) {
        if ($full.StartsWith($forbidden, [StringComparison]::OrdinalIgnoreCase)) { throw "$full is inside $forbidden; the kit must not map a user profile" }
    }
}
Remove-Item -Recurse -Force $kitInput -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $kitInput, $evidence | Out-Null

Copy-Item -LiteralPath $Installer -Destination (Join-Path $kitInput 'B.O.B.-setup.exe')
Copy-Item (Join-Path $PSScriptRoot 'qualify-installed-windows.ps1'), (Join-Path $PSScriptRoot 'check-installed-windows-subsystem.ps1') $kitInput
# The Sandbox has no Python; the qualification script uses its sqlite3 module.
robocopy $PythonHome (Join-Path $kitInput 'python') /E /NFL /NDL /NJH /NJS /XD site-packages Scripts include Doc tcl test __pycache__ | Out-Null
if ($LASTEXITCODE -ge 8) { throw "copying Python from $PythonHome failed (robocopy $LASTEXITCODE)" }

@"
`$ErrorActionPreference = 'Stop'
`$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
`$out = "C:\evidence\run-`$stamp"
New-Item -ItemType Directory -Force `$out | Out-Null
`$env:PATH = "C:\kit\python;`$env:PATH"
`$os = Get-CimInstance Win32_OperatingSystem
`$webview = (Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue).pv
"sandbox os=`$(`$os.Caption) `$(`$os.Version) build `$(`$os.BuildNumber); webview2=`$webview; network adapters up=`$(@(Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object Status -eq 'Up').Count)" | Set-Content "`$out\environment.txt"
& C:\kit\qualify-installed-windows.ps1 -Installer C:\kit\B.O.B.-setup.exe -EvidenceDir `$out -ExpectedPayloadSHA256 '$ExpectedPayloadSHA256' *>&1 | Tee-Object "`$out\console.txt"
"exit=`$LASTEXITCODE" | Add-Content "`$out\console.txt"
"@ | Set-Content -LiteralPath (Join-Path $kitInput 'run-in-sandbox.ps1') -Encoding utf8

$wsb = Join-Path $KitRoot 'BOB-Installer-Qualification.wsb'
@"
<Configuration>
  <Networking>Disable</Networking>
  <VGpu>Disable</VGpu>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <PrinterRedirection>Disable</PrinterRedirection>
  <AudioInput>Disable</AudioInput>
  <VideoInput>Disable</VideoInput>
  <ProtectedClient>Enable</ProtectedClient>
  <MemoryInMB>4096</MemoryInMB>
  <MappedFolders>
    <MappedFolder><HostFolder>$kitInput</HostFolder><SandboxFolder>C:\kit</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$evidence</HostFolder><SandboxFolder>C:\evidence</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\kit\run-in-sandbox.ps1</Command></LogonCommand>
</Configuration>
"@ | Set-Content -LiteralPath $wsb -Encoding utf8

# Verify what was written before anyone launches it.
[xml] $config = Get-Content -LiteralPath $wsb -Raw
$maps = @($config.Configuration.MappedFolders.MappedFolder)
if ($config.Configuration.Networking -ne 'Disable' -or $config.Configuration.ClipboardRedirection -ne 'Disable' -or $maps.Count -ne 2) { throw "generated $wsb does not have the required isolation" }
foreach ($m in $maps) { if (-not (Test-Path -LiteralPath $m.HostFolder)) { throw "mapped folder $($m.HostFolder) is missing" } }
if (($maps | Where-Object SandboxFolder -eq 'C:\kit').ReadOnly -ne 'true') { throw 'the kit input must be mapped read-only' }
Write-Host "Kit ready: $wsb"
Write-Host "installer sha256=$actual; mapped: $kitInput -> C:\kit (read-only), $evidence -> C:\evidence"

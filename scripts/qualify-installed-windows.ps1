# Installed-app acceptance for a B.O.B. NSIS installer on a DISPOSABLE Windows host.
# It installs B.O.B., writes to the host user's real B.O.B. profile, uninstalls and
# reinstalls. Never run it on a machine whose B.O.B. data matters.
# It changes no security policy; a blocked installer or executable fails the run.
param(
    [Parameter(Mandatory = $true)] [string] $Installer,
    [Parameter(Mandatory = $true)] [string] $BuiltExecutable,
    [Parameter(Mandatory = $true)] [string] $EvidenceDir
)
$ErrorActionPreference = 'Stop'
$MainTitle = 'B.O.B. | Better Organized Brain'
$Identifier = 'com.betterorganizedbrain.app'
$ProfileDir = Join-Path $env:APPDATA $Identifier
$Db = Join-Path $ProfileDir 'bob.sqlite3'
New-Item -ItemType Directory -Force $EvidenceDir | Out-Null
$EvidenceDir = (Resolve-Path $EvidenceDir).Path
$Results = [System.Collections.ArrayList]::new()
$Log = Join-Path $EvidenceDir 'qualification.log'

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text;
public class BobWin {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
}
"@

function Note([string] $Text) { $Text | Tee-Object -FilePath $Log -Append | Write-Host }
function Check([string] $Name, [bool] $Passed, [string] $Detail) {
    [void]$Results.Add([pscustomobject]@{ Check = $Name; Passed = $Passed; Detail = $Detail })
    Note ("[{0}] {1}: {2}" -f $(if ($Passed) { 'PASS' } else { 'FAIL' }), $Name, $Detail)
}
function Windows-Of([int[]] $Ids) {
    $list = [System.Collections.ArrayList]::new()
    [BobWin]::EnumWindows({ param($h, $l)
        $procId = 0; [BobWin]::GetWindowThreadProcessId($h, [ref]$procId) | Out-Null
        if ($Ids -contains $procId) {
            $t = [Text.StringBuilder]::new(256); [BobWin]::GetWindowText($h, $t, 256) | Out-Null
            $c = [Text.StringBuilder]::new(256); [BobWin]::GetClassName($h, $c, 256) | Out-Null
            [void]$list.Add([pscustomobject]@{ Pid = $procId; Handle = $h; Title = $t.ToString(); Class = $c.ToString(); Visible = [BobWin]::IsWindowVisible($h) })
        }
        $true }, [IntPtr]::Zero) | Out-Null
    $list
}
function Main-Window($p) { Windows-Of @($p.Id) | Where-Object { $_.Title -eq $MainTitle } | Select-Object -First 1 }
function Wait-Main($p, [int] $Sec = 60) {
    $deadline = (Get-Date).AddSeconds($Sec)
    while ((Get-Date) -lt $deadline) {
        $w = Main-Window $p
        if ($w -and $w.Visible) { return $w }
        if ($p.HasExited) { throw "B.O.B. exited with $($p.ExitCode) before showing its window" }
        Start-Sleep -Milliseconds 200
    }
    throw "no visible B.O.B. window within $Sec s"
}
function Close-Main($p) {
    [BobWin]::PostMessage((Main-Window $p).Handle, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    if (-not $p.WaitForExit(20000)) { throw "B.O.B. did not exit after its window was closed" }
    $p.ExitCode
}
function Start-Bob([string] $Exe, [hashtable] $Environment = @{}) {
    $psi = [Diagnostics.ProcessStartInfo]::new($Exe)
    $psi.UseShellExecute = $false
    foreach ($key in $Environment.Keys) { $psi.EnvironmentVariables[$key] = $Environment[$key] }
    [Diagnostics.Process]::Start($psi)
}
function Bob-Processes { @(Get-Process | Where-Object { $_.Path -and $_.Path -ieq $script:InstalledExe }) }
function Snapshot([string] $Path) {
    $code = "import sqlite3,sys`nc=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True)`nitems=','.join(r[0] for r in c.execute('select id from work_items order by id'))`nchat=c.execute('select count(*) from chat_entries').fetchone()[0]`nprint('schema=%d items=%s chat=%d' % (c.execute('pragma user_version').fetchone()[0], items, chat))"
    $file = Join-Path $env:TEMP 'bob-snapshot.py'; Set-Content -LiteralPath $file -Value $code -Encoding ascii
    (python -I $file $Path | Out-String).Trim()
}
function Seed([string] $Path) {
    $code = "import sqlite3,sys`nc=sqlite3.connect(sys.argv[1])`nrows=[('q-one','task','Qualification task one',15,'high','Today','planned',0),('q-two','task','Qualification task two',10,'normal','Today','planned',1),('q-three','idea','Qualification idea',None,'low',None,'planned',2)]`nc.executemany('insert into work_items(id,kind,title,estimate,priority,due,status,sort_order) values(?,?,?,?,?,?,?,?)',rows)`nc.executemany('insert into chat_entries(author,text) values(?,?)',[('user','Qualification question'),('bob','Qualification answer')])`nc.commit()"
    $file = Join-Path $env:TEMP 'bob-seed.py'; Set-Content -LiteralPath $file -Value $code -Encoding ascii
    python -I $file $Path
    if ($LASTEXITCODE -ne 0) { throw "seeding failed" }
}
function Capture($p, [string] $Name) {
    $w = Main-Window $p
    $r = New-Object BobWin+RECT; [BobWin]::GetWindowRect($w.Handle, [ref]$r) | Out-Null
    $bmp = [Drawing.Bitmap]::new($r.R - $r.L, $r.B - $r.T)
    $g = [Drawing.Graphics]::FromImage($bmp); $hdc = $g.GetHdc()
    [BobWin]::PrintWindow($w.Handle, $hdc, 2) | Out-Null
    $g.ReleaseHdc($hdc); $g.Dispose(); $bmp.Save((Join-Path $EvidenceDir "$Name.png")); $bmp.Dispose()
}
function Find-Install {
    $keys = Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue |
        ForEach-Object { Get-ItemProperty $_.PSPath } | Where-Object { $_.DisplayName -eq 'B.O.B.' }
    $keys | Select-Object -First 1
}
function Install([string] $Label) {
    $t = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $Installer -ArgumentList '/S' -PassThru -Wait
    $t.Stop()
    $entry = Find-Install
    $exe = if ($entry) { Get-ChildItem -LiteralPath ($entry.InstallLocation.Trim('"')) -Filter 'bob.exe' -ErrorAction SilentlyContinue | Select-Object -First 1 } else { $null }
    Check "$Label completes" (($p.ExitCode -eq 0) -and [bool]$exe) "exit=$($p.ExitCode) in $($t.ElapsedMilliseconds) ms; uninstall entry=$([bool]$entry); exe=$($exe.FullName)"
    if (-not $exe) { throw "installer did not produce bob.exe" }
    $script:InstalledExe = $exe.FullName
    $script:InstallEntry = $entry
}

try {
    Note "B.O.B. installed-app qualification, $([DateTime]::UtcNow.ToString('o'))"
    Note "Host: $([Environment]::OSVersion.VersionString); user=$env:USERNAME"
    $sac = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
    Note "Smart App Control VerifiedAndReputablePolicyState=$sac (recorded only; never changed)"
    if (Test-Path $ProfileDir) { throw "a B.O.B. profile already exists at $ProfileDir; this script needs a disposable host" }
    $installerHash = (Get-FileHash -LiteralPath $Installer).Hash
    $builtHash = (Get-FileHash -LiteralPath $BuiltExecutable).Hash
    Note "Installer $Installer sha256=$installerHash signature=$((Get-AuthenticodeSignature -LiteralPath $Installer).Status)"
    Note "Built bob.exe sha256=$builtHash"

    Install 'install'
    Check 'installed exe matches build' ((Get-FileHash -LiteralPath $InstalledExe).Hash -eq $builtHash) $InstalledExe
    & (Join-Path $PSScriptRoot 'check-installed-windows-subsystem.ps1') -Executable $InstalledExe -Installer $Installer `
        -EvidencePath (Join-Path $EvidenceDir 'installed-pe-evidence.json') -ExpectedExecutableSHA256 $builtHash -ExpectedInstallerSHA256 $installerHash
    Check 'PE checker (GUI subsystem 2, hashes)' $true 'scripts/check-installed-windows-subsystem.ps1 passed'

    # First launch on a fresh profile.
    $p = Start-Bob $InstalledExe; $t = [Diagnostics.Stopwatch]::StartNew(); $null = Wait-Main $p; $t.Stop()
    $children = @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$($p.Id)" | Where-Object { $_.Name -notlike 'msedgewebview2*' } | ForEach-Object Name)
    $consoleWindows = @(Windows-Of @($p.Id) | Where-Object { $_.Class -in 'ConsoleWindowClass', 'CASCADIA_HOSTING_WINDOW_CLASS' })
    Check 'launches' $true "window after $($t.ElapsedMilliseconds) ms"
    Check 'no console window' (($children.Count -eq 0) -and ($consoleWindows.Count -eq 0)) "non-webview child processes: [$($children -join ', ')]; console windows: $($consoleWindows.Count)"
    Check 'state stored in app data, not install dir' ((Test-Path $Db) -and -not (Test-Path (Join-Path (Split-Path $InstalledExe) 'bob.sqlite3'))) $Db
    Capture $p 'first-launch'
    Check 'normal close' ((Close-Main $p) -eq 0) 'exit 0'

    Seed $Db
    $seeded = Snapshot $Db
    Note "seeded: $seeded"
    $p = Start-Bob $InstalledExe; $null = Wait-Main $p
    Start-Sleep 2
    Capture $p 'seeded-tasks'

    # Second launch hands off and leaves SQLite untouched.
    $before = Snapshot $Db; $mtime = (Get-Item $Db).LastWriteTimeUtc
    $t = [Diagnostics.Stopwatch]::StartNew(); $s = Start-Bob $InstalledExe; $handedOff = $s.WaitForExit(20000); $t.Stop()
    Check 'second launch hands off' ($handedOff -and $s.ExitCode -eq 0 -and (Bob-Processes).Count -eq 1 -and $mtime -eq (Get-Item $Db).LastWriteTimeUtc -and $before -eq (Snapshot $Db)) "exit=$($s.ExitCode) after $($t.ElapsedMilliseconds) ms; processes=$((Bob-Processes).Count)"

    # Minimized window is restored and focused by a second launch.
    $w = Main-Window $p; [BobWin]::ShowWindow($w.Handle, 6) | Out-Null; Start-Sleep 1
    $wasMinimized = [BobWin]::IsIconic($w.Handle)
    $s = Start-Bob $InstalledExe; $null = $s.WaitForExit(20000); Start-Sleep 2
    $restored = (-not [BobWin]::IsIconic($w.Handle)) -and [BobWin]::IsWindowVisible($w.Handle)
    $foreground = [BobWin]::GetForegroundWindow() -eq $w.Handle
    Check 'minimized window restored' ($wasMinimized -and $restored) "minimized before=$wasMinimized; restored=$restored"
    Check 'existing window receives focus' $foreground "foreground=$foreground"
    Check 'tasks intact across relaunch' ((Close-Main $p) -eq 0 -and (Snapshot $Db) -eq $seeded) (Snapshot $Db)

    # Simultaneous launches never produce two owners or a windowless owner.
    $rounds = 10; $good = 0
    for ($round = 1; $round -le $rounds; $round++) {
        $a = Start-Bob $InstalledExe; $b = Start-Bob $InstalledExe
        Start-Sleep 12
        $alive = @($a, $b | Where-Object { -not $_.HasExited })
        $shown = @($alive | Where-Object { $w = Main-Window $_; $w -and $w.Visible })
        $detail = ($a, $b | ForEach-Object { if ($_.HasExited) { "exited $($_.ExitCode)" } else { (Windows-Of @($_.Id) | Where-Object { $_.Title -like 'B.O.B.*' } | ForEach-Object { "$($_.Title)/$(if ($_.Visible) { 'shown' } else { 'hidden' })" }) -join '+' } }) -join ' | '
        Note "simultaneous round ${round}: $detail"
        if ($shown.Count -eq 1) { $good++ }
        foreach ($x in $alive) {
            Windows-Of @($x.Id) | Where-Object { $_.Class -eq '#32770' -or $_.Title -eq $MainTitle } | ForEach-Object { [BobWin]::PostMessage($_.Handle, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
            if (-not $x.WaitForExit(20000)) { Note "  pid $($x.Id) did not exit after close; terminating"; Stop-Process -Id $x.Id -Force; $x.WaitForExit() }
        }
    }
    Check 'simultaneous launches: one visible owner' ($good -eq $rounds) "$good/$rounds rounds with exactly one visible main window"
    Check 'tasks intact after simultaneous launches' ((Snapshot $Db) -eq $seeded) (Snapshot $Db)

    # Restart after forced termination.
    $p = Start-Bob $InstalledExe; $null = Wait-Main $p; Stop-Process -Id $p.Id -Force; $p.WaitForExit()
    $p = Start-Bob $InstalledExe; $null = Wait-Main $p
    Check 'restart after forced termination' ((Close-Main $p) -eq 0 -and (Snapshot $Db) -eq $seeded) (Snapshot $Db)

    # Startup failure on a separate disposable profile root: corrupt SQLite fails closed.
    $root = Join-Path $env:TEMP 'bob-corrupt-root'
    $corruptProfile = Join-Path $root "AppData\Roaming\$Identifier"
    New-Item -ItemType Directory -Force $corruptProfile, (Join-Path $root 'AppData\Local') | Out-Null
    $bytes = New-Object byte[] 65536; ([Random]::new(1234)).NextBytes($bytes)
    $corruptDb = Join-Path $corruptProfile 'bob.sqlite3'; [IO.File]::WriteAllBytes($corruptDb, $bytes)
    $corruptHash = (Get-FileHash -LiteralPath $corruptDb).Hash
    $p = Start-Bob $InstalledExe @{ USERPROFILE = $root; APPDATA = (Join-Path $root 'AppData\Roaming'); LOCALAPPDATA = (Join-Path $root 'AppData\Local') }
    $dialog = $null; $mainSeen = $false; $t = [Diagnostics.Stopwatch]::StartNew()
    while ($t.Elapsed.TotalSeconds -lt 60 -and -not $dialog -and -not $p.HasExited) {
        $ws = Windows-Of @($p.Id)
        if ($ws | Where-Object { $_.Title -eq $MainTitle -and $_.Visible }) { $mainSeen = $true }
        $dialog = $ws | Where-Object { $_.Title -eq 'B.O.B. cannot start' -and $_.Visible } | Select-Object -First 1
        Start-Sleep -Milliseconds 50
    }
    if ($dialog) { Start-Sleep 1; [BobWin]::PostMessage($dialog.Handle, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
    $exited = $p.WaitForExit(20000); if (-not $exited) { Stop-Process -Id $p.Id -Force; $p.WaitForExit() }
    Check 'corrupt SQLite fails closed' ([bool]$dialog -and -not $mainSeen -and $exited -and $p.ExitCode -eq 1 -and (Get-FileHash -LiteralPath $corruptDb).Hash -eq $corruptHash) "dialog=$([bool]$dialog) after $($t.ElapsedMilliseconds) ms; main window seen=$mainSeen; exit=$($p.ExitCode); db unchanged=$((Get-FileHash -LiteralPath $corruptDb).Hash -eq $corruptHash)"

    # Uninstall keeps user data; reinstall picks it up again.
    $dbHash = (Get-FileHash -LiteralPath $Db).Hash
    $uninstaller = ($InstallEntry.UninstallString -replace '^"([^"]+)".*$', '$1')
    Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait
    $deadline = (Get-Date).AddSeconds(90)
    while ((Test-Path $InstalledExe) -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
    Check 'uninstall removes the app' (-not (Test-Path $InstalledExe) -and -not (Find-Install)) "exe present=$(Test-Path $InstalledExe); uninstall entry present=$([bool](Find-Install))"
    Check 'uninstall keeps user data' ((Test-Path $Db) -and (Get-FileHash -LiteralPath $Db).Hash -eq $dbHash) "db present=$(Test-Path $Db)"
    Install 'reinstall'
    $p = Start-Bob $InstalledExe; $null = Wait-Main $p; Start-Sleep 2
    Capture $p 'after-reinstall'
    Check 'tasks intact after reinstall' ((Close-Main $p) -eq 0 -and (Snapshot $Db) -eq $seeded) (Snapshot $Db)
} catch {
    Check 'qualification run' $false "aborted: $($_.Exception.Message)"
} finally {
    Get-Process | Where-Object { $_.Path -and $script:InstalledExe -and $_.Path -ieq $script:InstalledExe } | Stop-Process -Force -ErrorAction SilentlyContinue
    $Results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $EvidenceDir 'results.json') -Encoding utf8
    $Results | Format-Table -AutoSize | Out-String -Width 200 | Tee-Object -FilePath $Log -Append | Write-Host
}
if ($Results | Where-Object { -not $_.Passed }) { exit 1 }
Write-Host 'PASS: all installed-app checks passed.'

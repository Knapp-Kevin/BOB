# Runs INSIDE the offline film Sandbox ("B.O.B.: Find Your Next Step") after run-film.ps1 has
# installed the verified v0.1.0 package. Starts the installed B.O.B. on a separate fictional profile
# (fixture-film-sam), sizes its client area to 1920x1080, and serves a fixed set of capture verbs
# requested by files in C:\evidence\cmd. It runs nothing else.
param(
    [string] $Exe = (Join-Path $env:LOCALAPPDATA 'B.O.B\bob.exe'),
    [string] $Root = 'C:\demo',
    [string] $Evidence = 'C:\evidence',
    [string] $Ffmpeg = 'C:\kit\ffmpeg.exe',
    [string] $Python = 'C:\kit\python\python.exe'
)
$ErrorActionPreference = 'Stop'
$CW = 1920; $CH = 1080
$DemoEnv = @{ USERPROFILE = $Root; APPDATA = "$Root\AppData\Roaming"; LOCALAPPDATA = "$Root\AppData\Local" }
$Db = "$Root\AppData\Roaming\com.betterorganizedbrain.app\bob.sqlite3"
$Cmd = Join-Path $Evidence 'cmd'; $Media = Join-Path $Evidence 'media'
New-Item -ItemType Directory -Force $Cmd, $Media, "$Root\AppData\Roaming", "$Root\AppData\Local" | Out-Null
$Status = Join-Path $Cmd 'status.txt'
function Say([string] $Text) { "[$([DateTime]::UtcNow.ToString('HH:mm:ss'))] $Text" | Add-Content -LiteralPath $Status }

Add-Type @"
using System; using System.Runtime.InteropServices;
public class Demo {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int hh, uint f);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, int data, IntPtr extra);
}
"@
Add-Type -AssemblyName System.Windows.Forms
[Demo]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null

function Start-Demo {
    $psi = [Diagnostics.ProcessStartInfo]::new($Exe); $psi.UseShellExecute = $false
    foreach ($k in $DemoEnv.Keys) { $psi.EnvironmentVariables[$k] = $DemoEnv[$k] }
    $p = [Diagnostics.Process]::Start($psi)
    $deadline = (Get-Date).AddSeconds(60)
    do { Start-Sleep -Milliseconds 300; $script:Window = [Demo]::FindWindow([NullString]::Value, 'B.O.B. | Better Organized Brain') } while ($script:Window -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline)
    if ($script:Window -eq [IntPtr]::Zero) { throw 'demo window did not appear' }
    $script:Proc = $p
    Fit
}
function Stop-Demo {
    if ($script:Proc -and -not $script:Proc.HasExited) { $script:Proc.CloseMainWindow() | Out-Null; if (-not $script:Proc.WaitForExit(15000)) { $script:Proc.Kill() } }
    if ($script:Proc) { $script:LastExit = $script:Proc.ExitCode }
}
function Fit {
    [Demo]::ShowWindow($script:Window, 9) | Out-Null
    $w = New-Object Demo+RECT; $c = New-Object Demo+RECT
    [Demo]::GetWindowRect($script:Window, [ref]$w) | Out-Null; [Demo]::GetClientRect($script:Window, [ref]$c) | Out-Null
    $dw = ($w.R - $w.L) - ($c.R - $c.L); $dh = ($w.B - $w.T) - ($c.B - $c.T)
    [Demo]::SetWindowPos($script:Window, [IntPtr]::Zero, 0, 0, $CW + $dw, $CH + $dh, 0x0040) | Out-Null
    Start-Sleep -Milliseconds 500
    [Demo]::GetClientRect($script:Window, [ref]$c) | Out-Null
    $o = New-Object Demo+POINT; [Demo]::ClientToScreen($script:Window, [ref]$o) | Out-Null
    $script:Origin = $o
    $script:Region = @('-offset_x', $o.X, '-offset_y', $o.Y, '-video_size', "$($c.R)x$($c.B)")
    Say "fit: client $($c.R)x$($c.B) at $($o.X),$($o.Y)"
}
function Seed-Fixture {
    # Sam's fictional Tuesday: three planned tasks already competing on Today, four things waiting
    # in the Inbox. No real people, accounts or data.
    $code = @"
import sqlite3,sys
c=sqlite3.connect(sys.argv[1])
c.execute('delete from chat_entries'); c.execute('update app_state set active_item_id=null, handoff_objective=null, handoff_state=null, handoff_next=null')
c.execute('delete from work_items')
rows=[('f-slides','task',"Prep slides for Thursday's team meeting",45,'normal','Thursday','planned',0),
      ('f-bill','task','Pay the electric bill',10,'normal','Friday','planned',1),
      ('f-reply','task','Reply to Priya about Saturday plans',5,'normal',None,'planned',2),
      ('f-budget','task','Send the budget summary to Dana by 3 pm',20,'high','Today','inbox',3),
      ('f-form','task','Sign the school field-trip form',5,'normal','Tomorrow','inbox',4),
      ('f-faucet','note','Landlord: the kitchen faucet is still dripping',None,'normal',None,'inbox',5),
      ('f-picnic','idea',"Idea: a picnic for Grandma's birthday",None,'low',None,'inbox',6)]
c.executemany('insert into work_items(id,kind,title,estimate,priority,due,status,sort_order) values(?,?,?,?,?,?,?,?)',rows)
c.commit(); print('fixture-film-sam seeded:', [r[0] for r in c.execute('select id from work_items order by sort_order')])
"@
    $file = "$env:TEMP\seed-fixture.py"; Set-Content -LiteralPath $file -Value $code -Encoding ascii
    Say ((& $Python -I $file $Db | Out-String).Trim())
}
function Reset {
    Stop-Demo
    if (-not (Test-Path $Db)) { Start-Demo; Stop-Demo }   # first launch creates the schema
    Seed-Fixture; Start-Demo
}

# Real input to the B.O.B. window only. Coordinates are client-relative (0..1919, 0..1079);
# the cursor glides to the target so a recording shows where each click lands.
function Glide([int] $X, [int] $Y) {
    if ($X -lt 0 -or $X -ge $CW -or $Y -lt 0 -or $Y -ge $CH) { throw "($X,$Y) is outside the B.O.B. client area" }
    [Demo]::SetForegroundWindow($script:Window) | Out-Null
    $tx = $script:Origin.X + $X; $ty = $script:Origin.Y + $Y
    $from = New-Object Demo+POINT; [Demo]::GetCursorPos([ref]$from) | Out-Null
    for ($i = 1; $i -le 36; $i++) {
        $t = $i / 36; $e = $t * $t * (3 - 2 * $t)
        [Demo]::SetCursorPos([int]($from.X + ($tx - $from.X) * $e), [int]($from.Y + ($ty - $from.Y) * $e)) | Out-Null
        Start-Sleep -Milliseconds 20
    }
}
function Click([int] $X, [int] $Y) {
    Glide $X $Y; Start-Sleep -Milliseconds 150
    [Demo]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero); Start-Sleep -Milliseconds 60; [Demo]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
}
function Type-Text([string] $Text) {
    [Demo]::SetForegroundWindow($script:Window) | Out-Null
    foreach ($ch in $Text.ToCharArray()) {
        $k = if ('+^%~(){}[]'.Contains($ch)) { "{$ch}" } else { [string]$ch }
        [System.Windows.Forms.SendKeys]::SendWait($k); Start-Sleep -Milliseconds 45
    }
}
$Keys = @{ enter = '{ENTER}'; tab = '{TAB}'; 'shift-tab' = '+{TAB}'; escape = '{ESC}'; space = ' '; down = '{DOWN}'; up = '{UP}'; 'ctrl-k' = '^k'; 'ctrl-a' = '^a'; backspace = '{BACKSPACE}' }

Say "ffmpeg: $((& $Ffmpeg -hide_banner -version 2>&1 | Select-Object -First 1) -join ' ')"
Say "screen: $([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width)x$([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height)"
try { Reset; Say 'ready' } catch {
    Say "startup error: $($_.Exception.Message)"
    Say "bob processes: $((Get-Process bob -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Id) '$($_.MainWindowTitle)'" }) -join '; ')"
    & $Ffmpeg -hide_banner -loglevel error -y -f gdigrab -i desktop -frames:v 1 "$Media\startup-failure.png"
    throw
}
$recorder = $null
while ($true) {
    # The host writes each command file under another name and renames it to *.cmd.
    foreach ($f in Get-ChildItem $Cmd -Filter '*.cmd' -ErrorAction SilentlyContinue | Sort-Object Name) {
      $lines = @(Get-Content -LiteralPath $f.FullName | Where-Object { $_.Trim() })
      Remove-Item -LiteralPath $f.FullName
      foreach ($line in $lines) {
        $verb, $arg = ($line.Trim() -split '\s+', 2)
        $isName = $verb -in 'shot', 'record-start'
        if ($isName -and $arg -notmatch '^[A-Za-z0-9._-]+$') { Say "rejected name '$arg'"; continue }
        try {
            switch ($verb) {
                'click' { $x, $y = $arg -split '\s+'; Click ([int]$x) ([int]$y); Say "click $x,$y" }
                'move' { $x, $y = $arg -split '\s+'; Glide ([int]$x) ([int]$y); Say "move $x,$y" }
                'scroll' { [Demo]::SetForegroundWindow($script:Window) | Out-Null; [Demo]::mouse_event(0x0800, 0, 0, -120 * [int]$arg, [IntPtr]::Zero); Say "scroll $arg" }
                'type' { Type-Text $arg; Say "typed $($arg.Length) chars" }
                'key' { if (-not $Keys.ContainsKey($arg)) { throw "key '$arg' is not allowed" }; [Demo]::SetForegroundWindow($script:Window) | Out-Null; [System.Windows.Forms.SendKeys]::SendWait($Keys[$arg]); Say "key $arg" }
                'wait' { Start-Sleep -Milliseconds ([Math]::Min([int]$arg, 10000)); Say "waited $arg ms" }
                'shot' { & $Ffmpeg -hide_banner -loglevel error -y -f gdigrab -draw_mouse 0 @script:Region -i desktop -frames:v 1 "$Media\$arg.png"; Say "shot $arg" }
                'record-start' {
                    $psi = [Diagnostics.ProcessStartInfo]::new($Ffmpeg); $psi.UseShellExecute = $false; $psi.RedirectStandardInput = $true
                    $psi.Arguments = "-hide_banner -loglevel error -y -f gdigrab -framerate 30 -draw_mouse 1 $($script:Region -join ' ') -i desktop -c:v libx264 -preset ultrafast -crf 12 -pix_fmt yuv444p `"$Media\$arg.mkv`""
                    $recorder = [Diagnostics.Process]::Start($psi); $script:Marks = [ordered]@{}; $script:MarkFile = "$Media\$arg.marks.json"; Say "recording $arg" }
                'sync' {
                    # One second of black over the client area: the edit finds its end (blackdetect),
                    # and every mark is timed from that moment.
                    $form = New-Object System.Windows.Forms.Form
                    $form.FormBorderStyle = 'None'; $form.BackColor = [Drawing.Color]::Black; $form.TopMost = $true; $form.ShowInTaskbar = $false
                    $form.StartPosition = 'Manual'; $form.Bounds = New-Object Drawing.Rectangle($script:Origin.X, $script:Origin.Y, $CW, $CH)
                    $form.Show(); $form.Refresh(); Start-Sleep -Milliseconds 1000; $form.Close(); $form.Dispose()
                    $script:Zero = [DateTime]::UtcNow; $script:Marks['zero'] = 0.0; Say "sync zero" }
                'mark' {
                    if ($arg -notmatch '^[A-Za-z0-9._-]+$') { throw "bad mark name '$arg'" }
                    $script:Marks[$arg] = [Math]::Round(([DateTime]::UtcNow - $script:Zero).TotalSeconds, 3)
                    $script:Marks | ConvertTo-Json | Set-Content -LiteralPath $script:MarkFile -Encoding utf8; Say "mark $arg $($script:Marks[$arg])" }
                'record-stop' { if ($recorder) { $recorder.StandardInput.Write('q'); $recorder.WaitForExit(30000) | Out-Null; Say "recording stopped (exit $($recorder.ExitCode))"; $recorder = $null } }
                'fit' { Fit }
                'reset' { Reset; Say 'reset done' }
                'relaunch' { Stop-Demo; Start-Demo; Say "relaunched (exit of previous instance $($script:LastExit))" }
                default { Say "unknown verb '$verb'" }
            }
        } catch { Say "error in ${verb}: $($_.Exception.Message)" }
      }
      Say "done $($f.BaseName)"
    }
    Start-Sleep -Milliseconds 300
}

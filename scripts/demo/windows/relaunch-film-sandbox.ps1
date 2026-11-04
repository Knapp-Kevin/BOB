# Host side: close any Windows Sandbox, keep the previous evidence folder under a new name, start
# the film Sandbox and size its window so the Sandbox desktop fits a 1920x1080 B.O.B. client area.
param([string] $Kit = 'C:\qual\film-kit')
$ErrorActionPreference = 'Stop'
Get-Process | Where-Object { $_.Name -in 'WindowsSandbox', 'WindowsSandboxClient', 'WindowsSandboxRemoteSession' } | Stop-Process -Force -Confirm:$false
$t = 0; while ((Get-Process vmmemWindowsSandbox -ErrorAction SilentlyContinue) -and $t -lt 90) { Start-Sleep 3; $t += 3 }
Start-Sleep 5
if (Test-Path "$Kit\evidence") { Rename-Item "$Kit\evidence" "evidence-$(Get-Date -Format yyyyMMddTHHmmss)" }
New-Item -ItemType Directory "$Kit\evidence\cmd" | Out-Null
Start-Process "$Kit\BOB-Film.wsb"
Add-Type @"
using System; using System.Runtime.InteropServices;
public class FilmHost { [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
[DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int hh, uint f); }
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 60 -and $h -eq [IntPtr]::Zero; $i++) { Start-Sleep 2; $h = [FilmHost]::FindWindow([NullString]::Value, 'Windows Sandbox') }
if ($h -eq [IntPtr]::Zero) { throw 'Windows Sandbox window did not appear' }
[FilmHost]::SetWindowPos($h, [IntPtr]::Zero, 40, 20, 2060, 1300, 0x0040) | Out-Null
"film Sandbox started; window $h"

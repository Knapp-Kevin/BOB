# Film session: install the verified v0.1.0 package silently (per user), record its identity, then
# start the capture helper. Nothing else runs in this Sandbox.
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force C:\evidence\cmd | Out-Null
$installer = 'C:\kit\B.O.B._0.1.0_x64-setup.exe'
$p = Start-Process -FilePath $installer -ArgumentList '/S' -Wait -PassThru
$exe = Join-Path $env:LOCALAPPDATA 'B.O.B\bob.exe'
$sig = Get-AuthenticodeSignature $exe
$os = Get-CimInstance Win32_OperatingSystem
$info = [ordered]@{
    installer_sha256 = (Get-FileHash $installer).Hash; installer_exit = $p.ExitCode
    exe = $exe; exe_sha256 = (Get-FileHash $exe).Hash
    file_version = (Get-Item $exe).VersionInfo.FileVersion; product_version = (Get-Item $exe).VersionInfo.ProductVersion
    signature = "$($sig.Status)"; signer = $sig.SignerCertificate.Subject
    os = "$($os.Caption) $($os.Version)"
    smart_app_control = (Get-ItemProperty HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
    network_adapters_up = @(Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object Status -eq 'Up').Count
}
$info | ConvertTo-Json | Set-Content C:\evidence\install-provenance.json -Encoding utf8
"[$([DateTime]::UtcNow.ToString('HH:mm:ss'))] film session: installer exit $($p.ExitCode); exe $($info.exe_sha256) version $($info.file_version) $($info.signature)" | Add-Content C:\evidence\cmd\status.txt
& C:\kit\film-session.ps1

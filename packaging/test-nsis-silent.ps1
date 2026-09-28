# Silent install / uninstall test for the direct-download NSIS installer (Store certification
# style): installs with /S, checks the result, simulates enabled autostart, uninstalls with /S and
# checks that nothing is left in the registry or on disk.
#   .\packaging\test-nsis-silent.ps1 [-Installer path\to\Peron_x_x64-setup.exe]
# Refuses to run if Peron is already installed (it would remove the real installation).
# App data (%APPDATA%/%LOCALAPPDATA%\com.peron.app) is intentionally kept by a silent uninstall
# (Tauri's "Delete application data" checkbox is off), so it is not treated as a leftover.
param([string]$Installer)
$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\.."
if (-not $Installer) {
  $Installer = Get-ChildItem "$root\src-tauri\target\release\bundle\nsis\*-setup.exe" | Sort-Object LastWriteTime | Select-Object -Last 1 -ExpandProperty FullName
}
$uninstKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Peron"
$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$installDir = "$env:LOCALAPPDATA\Peron"
if (Test-Path $uninstKey) { throw "Peron is already installed; not touching a real installation." }

$results = [ordered]@{}
function Check($name, [bool]$ok) { $script:results[$name] = if ($ok) { "PASS" } else { "FAIL" } }

# Keep a pre-existing autostart value (e.g. from a dev build) and put it back afterwards.
$prevRun = (Get-ItemProperty $runKey -Name Peron -ErrorAction SilentlyContinue).Peron

Write-Host "Installing $Installer /S"
$p = Start-Process $Installer -ArgumentList "/S" -Wait -PassThru
Check "install exit code 0" ($p.ExitCode -eq 0)
Check "peron.exe installed" (Test-Path "$installDir\peron.exe")
Check "uninstall entry" (Test-Path $uninstKey)
$entry = Get-ItemProperty $uninstKey -ErrorAction SilentlyContinue
Check "Installed Apps: DisplayName = Peron" ($entry.DisplayName -eq "Peron")
Check "Installed Apps: Publisher set" ([bool]$entry.Publisher)
Check "Installed Apps: DisplayVersion set" ([bool]$entry.DisplayVersion)
Check "installer language recorded" ([bool](Get-ItemProperty "HKCU:\Software\Peron" -Name InstallerLanguage -ErrorAction SilentlyContinue))
Check "log folder Documents\Peron\Logs" (Test-Path (Join-Path ([Environment]::GetFolderPath("MyDocuments")) "Peron\Logs"))
Check "Start menu shortcut" ([bool](Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter "Peron*.lnk" -ErrorAction SilentlyContinue))

# Simulate "Start with Windows" having been enabled in the app.
New-ItemProperty $runKey -Name Peron -Value "`"$installDir\peron.exe`" --minimized" -Force | Out-Null

$uninstaller = ($entry.UninstallString -replace '"', '')
Write-Host "Uninstalling $uninstaller /S"
$u = Start-Process $uninstaller -ArgumentList "/S" -Wait -PassThru
# The NSIS uninstaller copies itself to %TEMP% and returns early; wait for the files to go.
for ($i = 0; $i -lt 30 -and (Test-Path "$installDir\peron.exe"); $i++) { Start-Sleep -Seconds 1 }
Check "uninstall exit code 0" ($u.ExitCode -eq 0)
Check "install dir removed" (-not (Test-Path "$installDir\peron.exe"))
Check "uninstall entry removed" (-not (Test-Path $uninstKey))
Check "HKCU\Software\Peron removed" (-not (Test-Path "HKCU:\Software\Peron"))
Check "AUMID key removed" (-not (Test-Path "HKCU:\Software\Classes\AppUserModelId\com.peron.app"))
Check "autostart Run value removed" (-not (Get-ItemProperty $runKey -Name Peron -ErrorAction SilentlyContinue))
Check "Start menu shortcut removed" (-not (Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter "Peron*.lnk" -ErrorAction SilentlyContinue))

if ($prevRun) { New-ItemProperty $runKey -Name Peron -Value $prevRun -Force | Out-Null }

$results.GetEnumerator() | ForEach-Object { "{0,-4} {1}" -f $_.Value, $_.Key }
if ($results.Values -contains "FAIL") { exit 1 }

# Builds the Microsoft Store package: target/msix/Peron_<version>_x64.msix
#
#   .\packaging\msix\build-msix.ps1 [-SkipBuild] [-SelfSign]
#
# The defaults are Peron's Store identity (Partner Center → Product management → Product
# identity; must match exactly, including PublisherDisplayName). The Store signs submitted
# packages itself, so upload the unsigned package; -SelfSign is only for installing a local test
# build (trust the generated certificate first, see docs/MANUAL_TESTS.md).
param(
  [string]$IdentityName = "M.BurakAltiparmak.Peron",
  [string]$Publisher = "CN=9B72CAB6-7945-401D-A620-65C1A1B29120",
  [string]$PublisherDisplayName = "M. Burak Altiparmak",
  [switch]$SkipBuild,
  [switch]$SelfSign
)
$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\..\.."
$sdk = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $sdk) { throw "Windows SDK (makeappx.exe) not found" }
$bin = $sdk.DirectoryName

# --- Version: SemVer → MSIX Major.Minor.Build.0 ------------------------------------------------
# Store requires Revision = 0 and every submission to be higher than the last one.
# Build = Patch*1000 + beta number, stable = Patch*1000 + 999:
#   0.1.0-beta.1 → 0.1.1.0   0.1.0-beta.2 → 0.1.2.0   0.1.0 → 0.1.999.0   0.1.1-beta.1 → 0.1.1001.0
$semver = (Get-Content "$root\package.json" -Raw | ConvertFrom-Json).version
if ($semver -notmatch '^(\d+)\.(\d+)\.(\d+)(?:-[A-Za-z]+\.(\d+))?$') { throw "Unsupported version: $semver" }
$build = [int]$Matches[3] * 1000 + $(if ($Matches[4]) { [int]$Matches[4] } else { 999 })
$msixVersion = "$($Matches[1]).$($Matches[2]).$build.0"
Write-Host "Version $semver -> MSIX $msixVersion"

# --- Build peron.exe (same binary as the NSIS build; it detects the package at runtime) --------
if (-not $SkipBuild) {
  Push-Location $root
  # npx directly: `npm run … -- --no-bundle` loses the flag when invoked from PowerShell.
  npx tauri build --no-bundle
  if ($LASTEXITCODE) { throw "tauri build failed" }
  Pop-Location
}

# --- Stage -------------------------------------------------------------------------------------
$out = "$root\src-tauri\target\msix"
$stage = "$out\stage"
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$stage\Assets" | Out-Null
Copy-Item "$root\src-tauri\target\release\peron.exe" $stage

# --- Visual assets from the icon source --------------------------------------------------------
Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Image]::FromFile("$root\design\peron-icon.png")
function Save-Asset([string]$name, [int]$w, [int]$h, [double]$fill = 1.0) {
  $bmp = New-Object System.Drawing.Bitmap $w, $h
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = 'HighQualityBicubic'; $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  $s = [Math]::Min($w, $h) * $fill
  $g.DrawImage($icon, [single](($w - $s) / 2), [single](($h - $s) / 2), [single]$s, [single]$s)
  $g.Dispose(); $bmp.Save("$stage\Assets\$name", [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
}
foreach ($scale in 100, 200) {
  $k = $scale / 100
  Save-Asset "Square44x44Logo.scale-$scale.png" (44 * $k) (44 * $k)
  Save-Asset "Square150x150Logo.scale-$scale.png" (150 * $k) (150 * $k) 0.7
  Save-Asset "Wide310x150Logo.scale-$scale.png" (310 * $k) (150 * $k) 0.7
  Save-Asset "StoreLogo.scale-$scale.png" (50 * $k) (50 * $k)
}
# Taskbar / Start list icons without the tile plate.
foreach ($size in 16, 24, 32, 48, 256) {
  Save-Asset "Square44x44Logo.targetsize-$size.png" $size $size
  Save-Asset "Square44x44Logo.targetsize-${size}_altform-unplated.png" $size $size
}
$icon.Dispose()

# --- Manifest ----------------------------------------------------------------------------------
$manifest = (Get-Content "$PSScriptRoot\AppxManifest.template.xml" -Raw -Encoding utf8).
  Replace("{{IDENTITY_NAME}}", $IdentityName).
  Replace("{{PUBLISHER}}", [System.Security.SecurityElement]::Escape($Publisher)).
  Replace("{{PUBLISHER_DISPLAY_NAME}}", [System.Security.SecurityElement]::Escape($PublisherDisplayName)).
  Replace("{{VERSION}}", $msixVersion)
[System.IO.File]::WriteAllText("$stage\AppxManifest.xml", $manifest, (New-Object System.Text.UTF8Encoding $false))

# --- Resource index (maps scale-/targetsize- qualified assets) ---------------------------------
Push-Location $out
& "$bin\makepri.exe" createconfig /cf priconfig.xml /dq en-US /pv 10.0.0 /o | Out-Null
# Single package, not a bundle: keep every scale in one resources.pri (the default config splits
# scale-200 into a resource pack that a flat .msix would never load).
[xml]$pri = Get-Content priconfig.xml
$packaging = $pri.resources.SelectSingleNode("packaging")
if ($packaging) { [void]$pri.resources.RemoveChild($packaging); $pri.Save("$out\priconfig.xml") }
& "$bin\makepri.exe" new /pr $stage /cf priconfig.xml /mn "$stage\AppxManifest.xml" /of "$stage\resources.pri" /o | Out-Null
if ($LASTEXITCODE) { Pop-Location; throw "makepri failed" }
Pop-Location

# --- Pack --------------------------------------------------------------------------------------
$msix = "$out\Peron_${semver}_x64.msix"
& "$bin\makeappx.exe" pack /d $stage /p $msix /o
if ($LASTEXITCODE) { throw "makeappx failed" }

if ($SelfSign) {
  # Local testing only: a throwaway certificate whose subject equals the manifest Publisher.
  $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -eq $Publisher | Select-Object -First 1
  if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type Custom -Subject $Publisher -KeyUsage DigitalSignature `
      -FriendlyName "Peron MSIX test" -CertStoreLocation Cert:\CurrentUser\My `
      -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
  }
  & "$bin\signtool.exe" sign /fd SHA256 /sha1 $cert.Thumbprint $msix
  if ($LASTEXITCODE) { throw "signtool failed" }
  Write-Host "Self-signed with $($cert.Thumbprint). Trust it (Trusted People) before installing."
}

Get-Item $msix | Select-Object Name, @{n = 'MB'; e = { [math]::Round($_.Length / 1MB, 2) } }

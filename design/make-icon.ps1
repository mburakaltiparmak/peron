# Builds the app icon source (design/peron-icon.png, 1024x1024) from the chosen logo concept:
# the logo is centered on a white rounded tile so it stays legible on dark taskbars and trays.
# Then: npx tauri icon design/peron-icon.png
param(
  [string]$Source = "$PSScriptRoot\logo-concepts\peron-concept-08-multi-platform-p-node.png",
  [string]$Out = "$PSScriptRoot\peron-icon.png",
  [string]$UiOut = "$PSScriptRoot\..\public\logo.png"
)
Add-Type -AssemblyName System.Drawing

$src = New-Object System.Drawing.Bitmap $Source
# Content bounds (non-transparent pixels)
$minX = $src.Width; $minY = $src.Height; $maxX = 0; $maxY = 0
for ($y = 0; $y -lt $src.Height; $y += 2) {
  for ($x = 0; $x -lt $src.Width; $x += 2) {
    if ($src.GetPixel($x, $y).A -gt 20) {
      if ($x -lt $minX) { $minX = $x }; if ($y -lt $minY) { $minY = $y }
      if ($x -gt $maxX) { $maxX = $x }; if ($y -gt $maxY) { $maxY = $y }
    }
  }
}
$cw = $maxX - $minX + 1; $ch = $maxY - $minY + 1

function New-Icon([int]$size, [string]$path) {
  $bmp = New-Object System.Drawing.Bitmap $size, $size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'; $g.InterpolationMode = 'HighQualityBicubic'; $g.PixelOffsetMode = 'HighQuality'
  $g.Clear([System.Drawing.Color]::Transparent)
  # White rounded tile (radius ~22%)
  $r = [int]($size * 0.22); $d = $r * 2; $m = [int]($size * 0.02); $s = $size - 2 * $m
  $tile = New-Object System.Drawing.Drawing2D.GraphicsPath
  $tile.AddArc($m, $m, $d, $d, 180, 90); $tile.AddArc($m + $s - $d, $m, $d, $d, 270, 90)
  $tile.AddArc($m + $s - $d, $m + $s - $d, $d, $d, 0, 90); $tile.AddArc($m, $m + $s - $d, $d, $d, 90, 90)
  $tile.CloseFigure()
  $g.FillPath([System.Drawing.Brushes]::White, $tile)
  # Logo scaled to 74% of the tile height, centered
  $scale = ($size * 0.74) / [Math]::Max($cw, $ch)
  $w = $cw * $scale; $h = $ch * $scale
  $dest = New-Object System.Drawing.RectangleF ((($size - $w) / 2), (($size - $h) / 2), $w, $h)
  $srcRect = New-Object System.Drawing.RectangleF $minX, $minY, $cw, $ch
  $g.DrawImage($src, $dest, $srcRect, [System.Drawing.GraphicsUnit]::Pixel)
  $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
}

New-Icon 1024 $Out
New-Icon 96 $UiOut
$src.Dispose()
"icon: $Out"; "ui: $UiOut"

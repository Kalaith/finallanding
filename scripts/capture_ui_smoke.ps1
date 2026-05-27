param(
    [string]$OutputDir = "docs\verification",
    [int]$Frames = 8
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$workspaceRoot = Split-Path -Parent (Split-Path -Parent $repoRoot)
$exe = Join-Path $workspaceRoot ".cargo-target\debug\finallanding.exe"
$outDir = Join-Path $repoRoot $OutputDir

Set-Location $repoRoot
cargo build

if (!(Test-Path -LiteralPath $exe)) {
    throw "Missing executable: $exe"
}

New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Add-Type -AssemblyName System.Drawing

$sizes = @(
    @{ Width = 1280; Height = 720; Name = "ui_smoke_1280x720.png"; Fullscreen = "0" },
    @{ Width = 1920; Height = 1080; Name = "ui_smoke_1920x1080.png"; Fullscreen = "1" }
)

foreach ($size in $sizes) {
    $path = Join-Path $outDir $size.Name
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }

    $env:TFL_START_GAMEPLAY = "1"
    $env:TFL_CAPTURE_PATH = $path
    $env:TFL_CAPTURE_FRAMES = "$Frames"
    $env:TFL_WINDOW_WIDTH = "$($size.Width)"
    $env:TFL_WINDOW_HEIGHT = "$($size.Height)"
    $env:TFL_FULLSCREEN = "$($size.Fullscreen)"

    & $exe

    if (!(Test-Path -LiteralPath $path)) {
        throw "Capture failed: $path was not created."
    }

    $file = Get-Item -LiteralPath $path
    if ($file.Length -lt 100000) {
        throw "Capture failed: $path is unexpectedly small ($($file.Length) bytes)."
    }

    $image = [System.Drawing.Image]::FromFile($path)
    try {
        if ($image.Width -ne $size.Width -or $image.Height -ne $size.Height) {
            throw "Capture failed: $path is $($image.Width)x$($image.Height), expected $($size.Width)x$($size.Height)."
        }
    }
    finally {
        $image.Dispose()
    }

    Write-Host "Captured $($file.FullName) ($($file.Length) bytes, $($size.Width)x$($size.Height))"
}

Remove-Item Env:\TFL_START_GAMEPLAY -ErrorAction SilentlyContinue
Remove-Item Env:\TFL_CAPTURE_PATH -ErrorAction SilentlyContinue
Remove-Item Env:\TFL_CAPTURE_FRAMES -ErrorAction SilentlyContinue
Remove-Item Env:\TFL_WINDOW_WIDTH -ErrorAction SilentlyContinue
Remove-Item Env:\TFL_WINDOW_HEIGHT -ErrorAction SilentlyContinue
Remove-Item Env:\TFL_FULLSCREEN -ErrorAction SilentlyContinue

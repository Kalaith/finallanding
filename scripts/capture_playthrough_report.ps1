param(
    [string]$OutputPath = "docs\verification\playthrough_report.md"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$output = Join-Path $repoRoot $OutputPath

Set-Location $repoRoot
cargo build

if (Test-Path -LiteralPath $output) {
    Remove-Item -LiteralPath $output -Force
}

$env:TFL_PLAYTHROUGH_REPORT_PATH = $output
try {
    cargo run
}
finally {
    Remove-Item Env:\TFL_PLAYTHROUGH_REPORT_PATH -ErrorAction SilentlyContinue
}

if (!(Test-Path -LiteralPath $output)) {
    throw "Playthrough report was not created: $output"
}

$file = Get-Item -LiteralPath $output
if ($file.Length -lt 500) {
    throw "Playthrough report is unexpectedly small: $($file.Length) bytes"
}

$content = Get-Content -LiteralPath $output -Raw
foreach ($required in @("Reference", "Conservative", "Survey heavy", "No missions", "Tech", "Incidents")) {
    if ($content -notmatch [Regex]::Escape($required)) {
        throw "Playthrough report missing expected text: $required"
    }
}

Write-Host "Captured $($file.FullName) ($($file.Length) bytes)"

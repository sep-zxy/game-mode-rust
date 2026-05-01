Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $projectRoot

function Assert-Tool([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Missing tool: $Name"
    }
}

Assert-Tool "pnpm"
Assert-Tool "cargo"

Write-Host "Installing dependencies..."
pnpm install

Write-Host "Launching Tauri dev mode..."
pnpm tauri:dev

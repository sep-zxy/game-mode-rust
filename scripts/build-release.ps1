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

Write-Host "[1/3] Installing frontend dependencies..."
pnpm install

Write-Host "[2/3] Running frontend build..."
pnpm build

Write-Host "[3/3] Building Tauri bundle..."
pnpm tauri:build

Write-Host "Done"

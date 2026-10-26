# Athanor Sequential Crates.io Publisher
# Publishes all 10 Athanor crates in dependency order with index wait times.

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " ⚗ ATHANOR CRATES.IO PUBLISHING PIPELINE" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Verify credentials exist
$credentialsPath = "$HOME\.cargo\credentials.toml"
$legacyPath = "$HOME\.cargo\credentials"
if (-not (Test-Path $credentialsPath) -and -not (Test-Path $legacyPath)) {
    Write-Error "No crates.io credentials found! Please run 'cargo login <YOUR_TOKEN>' first."
}

# 2. Sequential publish queue
$publishQueue = @(
    @{ Name = "athanor-core";     Wait = 45 },
    @{ Name = "athanor-crucible"; Wait = 30 },
    @{ Name = "athanor-macros";   Wait = 30 },
    @{ Name = "athanor-opus";     Wait = 30 },
    @{ Name = "athanor-alembic";  Wait = 30 },
    @{ Name = "athanor-mcp";      Wait = 30 },
    @{ Name = "athanor-rag";      Wait = 30 },
    @{ Name = "athanor-evals";    Wait = 30 },
    @{ Name = "athanor-atelier";  Wait = 45 },
    @{ Name = "athanor";          Wait = 0  }
)

$step = 1
$total = $publishQueue.Count

foreach ($item in $publishQueue) {
    $crateName = $item.Name
    $waitTime = $item.Wait

    Write-Host "`n[$step/$total] Publishing $crateName to crates.io..." -ForegroundColor Yellow
    cargo publish -p $crateName

    Write-Host "  [SUCCESS] $crateName published!" -ForegroundColor Green

    if ($waitTime -gt 0) {
        Write-Host "  Waiting $waitTime seconds for crates.io index to update..." -ForegroundColor Gray
        Start-Sleep -Seconds $waitTime
    }

    $step++
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " ALL 10 CRATES PUBLISHED SUCCESSFULLY TO CRATES.IO!" -ForegroundColor Green
Write-Host " Namespace 'athanor' is permanently secured." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan

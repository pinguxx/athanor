<#
.SYNOPSIS
  Inspects a SQLite crucible database file
#>
param(
    [string]$DbPath = "crucible.db"
)

if (-not (Test-Path $DbPath)) {
    Write-Host "Database file '$DbPath' not found." -ForegroundColor Yellow
    exit 0
}

Write-Host "Inspecting Crucible DB: $DbPath..." -ForegroundColor Cyan
# Uses sqlite3 CLI if available
if (Get-Command sqlite3 -ErrorAction SilentlyContinue) {
    Write-Host "`n=== Active Runs ===" -ForegroundColor Yellow
    sqlite3 $DbPath "SELECT run_id, opus_name, status, updated_at FROM opus_runs LIMIT 10;"
    Write-Host "`n=== Suspended Tokens ===" -ForegroundColor Yellow
    sqlite3 $DbPath "SELECT token, run_id, vessel_id FROM suspended_tokens LIMIT 10;"
} else {
    Write-Host "sqlite3 CLI not found in PATH. Install sqlite3 to inspect database via terminal." -ForegroundColor Gray
}

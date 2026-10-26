<#
.SYNOPSIS
  Profiles the Working Set (RSS) of a Athanor process against the 40MB limit
#>
param(
    [string]$ProcessName = "athanor"
)

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "ATHANOR MEMORY PROFILE (SLA: < 40 MB RSS)" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$proc = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
if (-not $proc) {
    Write-Host "No process matching '$ProcessName' currently running." -ForegroundColor Yellow
    exit 0
}

$rssMb = [math]::Round($proc.WorkingSet64 / 1MB, 2)
Write-Host "Process: $($proc.ProcessName) (PID: $($proc.Id))" -ForegroundColor White
Write-Host "Resident Memory (RSS): ${rssMb} MB" -ForegroundColor $(if ($rssMb -le 40.0) { "Green" } else { "Red" })

if ($rssMb -gt 40.0) {
    Write-Host "[FAIL] Process exceeds sub-40MB memory SLA by $([math]::Round($rssMb - 40.0, 2)) MB!" -ForegroundColor Red
    exit 1
} else {
    Write-Host "[PASS] Process complies with Tenet 5 sub-40MB memory constraint." -ForegroundColor Green
    exit 0
}

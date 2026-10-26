<#
.SYNOPSIS
  Master Post-Task Audit Runner for Athanor
.DESCRIPTION
  Executes the core engineering and security audit pipeline:
  1. Critical Code Reviewer: inspects git diff and assesses blast radius
  2. Security Auditor: scans for secrets, unwrap/expect panic vectors, unsafe blocks, clippy, and unit tests
#>

param(
    [switch]$SkipTests
)

$RepoRoot = (Get-Item -Path "$PSScriptRoot\..\..").FullName

Write-Host "======================================================================" -ForegroundColor Magenta
Write-Host " ⚗  ATHANOR MASTER POST-TASK AUDIT PIPELINE" -ForegroundColor Magenta
Write-Host "======================================================================" -ForegroundColor Magenta
Write-Host "Repository: $RepoRoot" -ForegroundColor Gray
Write-Host ""

$failedAudits = @()

# ------------------------------------------------------------
# 1. CRITICAL CODE REVIEWER (Diff & Blast Radius Inspection)
# ------------------------------------------------------------
Write-Host "[STEP 1/2] Running Critical Code Reviewer Reconnaissance..." -ForegroundColor Cyan
try {
    & "$RepoRoot\.agents\skills\critical-code-reviewer\scripts\inspect_diff.ps1"
} catch {
    Write-Host "[-] Critical Code Reviewer inspection encountered an error: $_" -ForegroundColor Red
    $failedAudits += "critical-code-reviewer"
}

# ------------------------------------------------------------
# 2. SECURITY AUDITOR (Secrets, Invariants, Clippy, Tests)
# ------------------------------------------------------------
Write-Host "`n[STEP 2/2] Running Security & Invariant Auditor..." -ForegroundColor Cyan
try {
    & "$RepoRoot\.agents\skills\security-auditor\scripts\audit_security.ps1" -RepoRoot $RepoRoot -SkipLiveServer
    if ($LASTEXITCODE -ne 0) {
        $failedAudits += "security-auditor"
    }
} catch {
    Write-Host "[-] Security Auditor encountered an error: $_" -ForegroundColor Red
    $failedAudits += "security-auditor"
}

# ------------------------------------------------------------
# FINAL SUMMARY
# ------------------------------------------------------------
Write-Host "`n======================================================================" -ForegroundColor Magenta
if ($failedAudits.Count -eq 0) {
    Write-Host " ✨ ALL POST-TASK AUDITS PASSED WITH ZERO BLOCKERS!" -ForegroundColor Green
    Write-Host " Codebase adheres to ironclad quality, safety, and security standards." -ForegroundColor Green
} else {
    Write-Host " ❌ POST-TASK AUDIT FAILED IN THE FOLLOWING SUBSYSTEMS:" -ForegroundColor Red
    foreach ($fa in $failedAudits) {
        Write-Host "    - $fa" -ForegroundColor Red
    }
}
Write-Host "======================================================================" -ForegroundColor Magenta

if ($failedAudits.Count -gt 0) {
    exit 1
} else {
    exit 0
}

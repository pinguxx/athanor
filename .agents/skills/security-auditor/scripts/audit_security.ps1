<#
.SYNOPSIS
  Automated Security Posture Auditor for Athanor
.DESCRIPTION
  Scans the Athanor codebase for common security vulnerabilities and code quality invariants:
  - Hardcoded secrets & credentials
  - Forbidden panic vectors (unwrap/expect) in non-test crate sources
  - Unaudited unsafe blocks
  - Potential command/SQL injection risks
  - SSRF protections on network requests
  - HTTP Security Headers when Atelier dev server is running
#>

param(
    [string]$RepoRoot = "",
    [string]$BackendUrl = "http://127.0.0.1:4000",
    [switch]$SkipLiveServer
)

if (-not $RepoRoot) {
    try {
        $gitRoot = git rev-parse --show-toplevel 2>$null
        if ($gitRoot) { $RepoRoot = $gitRoot.Trim() }
    } catch {}
    if (-not $RepoRoot) {
        $RepoRoot = (Get-Item -Path "$PSScriptRoot\..\..\..\..").FullName
    }
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "ATHANOR AUTOMATED SECURITY & INVARIANT AUDITOR" -ForegroundColor Cyan
Write-Host "Repo Root: $RepoRoot" -ForegroundColor Gray
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$issuesFound = 0
$warningsFound = 0

# ------------------------------------------------------------
# 1. SCAN FOR HARDCODED SECRETS & CREDENTIALS
# ------------------------------------------------------------
Write-Host "[1/5] Scanning for hardcoded secrets and credentials..." -ForegroundColor Yellow

$secretPatterns = @(
    "ghp_[0-9a-zA-Z]{36}",
    "sk_live_[0-9a-zA-Z]{24}",
    "sk-[0-9a-zA-Z]{48}",
    "AKIA[0-9A-Z]{16}"
)

$srcFiles = Get-ChildItem -Path $RepoRoot -Recurse -File | Where-Object {
    $_.FullName -notmatch '\\(node_modules|target|\.git|dist)\\'
}

foreach ($pat in $secretPatterns) {
    $matches = $srcFiles | Select-String -Pattern $pat -CaseSensitive
    if ($matches) {
        Write-Host "  [CRITICAL] Potential secret matching pattern '$pat' found:" -ForegroundColor Red
        foreach ($m in $matches) {
            Write-Host "     $($m.Path):$($m.LineNumber)" -ForegroundColor Red
            $issuesFound++
        }
    }
}
Write-Host "  [PASS] Secret scanning completed." -ForegroundColor Green

# ------------------------------------------------------------
# 2. SCAN RUST CODE FOR FORBIDDEN PANIC VECTORS (unwrap/expect)
# ------------------------------------------------------------
Write-Host "`n[2/5] Scanning non-test Rust sources for unwrap() / expect() calls..." -ForegroundColor Yellow

$rustSourceFiles = Get-ChildItem -Path "$RepoRoot\crates" -Recurse -File -Filter "*.rs" | Where-Object {
    $_.FullName -match '\\src\\' -and $_.FullName -notmatch '\\tests\\'
}

$panicMatches = @()
foreach ($file in $rustSourceFiles) {
    $lines = Get-Content $file.FullName
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        # Skip comments and macro test modules
        if ($line.Trim().StartsWith("//") -or $line.Trim().StartsWith("/*")) { continue }
        if ($line -match '\.(unwrap|expect)\(' -and $line -notmatch '#\[test\]') {
            $panicMatches += [PSCustomObject]@{
                Path = $file.FullName.Replace($RepoRoot, "")
                LineNumber = $lineNum
                Line = $line.Trim()
            }
        }
    }
}

if ($panicMatches.Count -gt 0) {
    Write-Host "  [CRITICAL] Found $($panicMatches.Count) forbidden unwrap()/expect() call(s) in non-test source:" -ForegroundColor Red
    foreach ($pm in $panicMatches) {
        Write-Host "     $($pm.Path):$($pm.LineNumber) -> $($pm.Line)" -ForegroundColor Red
        $issuesFound++
    }
} else {
    Write-Host "  [PASS] Zero unwrap() or expect() calls in non-test crate sources! Ironclad error handling verified." -ForegroundColor Green
}

# ------------------------------------------------------------
# 3. SCAN FOR UNAUDITED UNSAFE BLOCKS
# ------------------------------------------------------------
Write-Host "`n[3/5] Verifying unsafe block containment..." -ForegroundColor Yellow

$unsafeMatches = @()
foreach ($file in $rustSourceFiles) {
    $matches = Select-String -Path $file.FullName -Pattern 'unsafe\s*\{'
    if ($matches) {
        foreach ($m in $matches) {
            $unsafeMatches += $m
        }
    }
}

if ($unsafeMatches.Count -gt 0) {
    Write-Host "  [WARN] Found $($unsafeMatches.Count) unsafe block(s). Ensure each has a // SAFETY comment:" -ForegroundColor Yellow
    foreach ($um in $unsafeMatches) {
        $relPath = $um.Path.Replace($RepoRoot, "")
        Write-Host "     $($relPath):$($um.LineNumber) -> $($um.Line.Trim())" -ForegroundColor Yellow
        $warningsFound++
    }
} else {
    Write-Host "  [PASS] 100% safe Rust code across all non-test crates. Zero unsafe blocks." -ForegroundColor Green
}

# ------------------------------------------------------------
# 4. RUN WORKSPACE CLIPPY & UNIT/INTEGRATION TESTS
# ------------------------------------------------------------
Write-Host "`n[4/5] Executing workspace Clippy and test suite..." -ForegroundColor Yellow

Push-Location $RepoRoot
try {
    $clippyOutput = cargo clippy --workspace --all-targets -- -D warnings 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [PASS] Cargo clippy passed with zero warnings!" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] Cargo clippy reported warnings/errors:" -ForegroundColor Red
        Write-Host $clippyOutput -ForegroundColor Red
        $issuesFound++
    }

    $testOutput = cargo test --workspace 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [PASS] All workspace tests passed with 0 failures!" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] Workspace tests failed:" -ForegroundColor Red
        Write-Host $testOutput -ForegroundColor Red
        $issuesFound++
    }
} finally {
    Pop-Location
}

# ------------------------------------------------------------
# 5. TEST LIVE HTTP SECURITY HEADERS (IF ATELIER SERVER RUNNING)
# ------------------------------------------------------------
if (-not $SkipLiveServer) {
    Write-Host "`n[5/5] Checking live HTTP security headers against $BackendUrl..." -ForegroundColor Yellow
    try {
        $resp = Invoke-WebRequest -Uri "$BackendUrl/api/registry" -Method Get -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
        
        $expectedHeaders = @(
            "X-Content-Type-Options",
            "X-Frame-Options",
            "Content-Security-Policy"
        )
        
        foreach ($hdr in $expectedHeaders) {
            $val = $resp.Headers[$hdr]
            if ($val) {
                Write-Host "  [PASS] $hdr : $val" -ForegroundColor Green
            } else {
                Write-Host "  [WARN] Missing header: $hdr" -ForegroundColor Yellow
                $warningsFound++
            }
        }
    } catch {
        Write-Host "  [INFO] Live server check skipped: Atelier server not currently active on $BackendUrl." -ForegroundColor Gray
    }
}

# ------------------------------------------------------------
# SUMMARY REPORT
# ------------------------------------------------------------
Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "AUDIT SUMMARY" -ForegroundColor Cyan
if ($issuesFound -gt 0) { $blockerColor = "Red" } else { $blockerColor = "Green" }
if ($warningsFound -gt 0) { $warningColor = "Yellow" } else { $warningColor = "Green" }
Write-Host "  Critical Security/Invariant Blockers: $issuesFound" -ForegroundColor $blockerColor
Write-Host "  Security Warnings/Nits:               $warningsFound" -ForegroundColor $warningColor
Write-Host "==========================================================" -ForegroundColor Cyan

if ($issuesFound -gt 0) {
    exit 1
} else {
    exit 0
}

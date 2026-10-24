param(
    [string]$TargetRef = "HEAD"
)

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " ADVERSARIAL DIFF RECONNAISSANCE TOOL" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# Check git status
$changes = git status --short
if (-not $changes) {
    Write-Host "Working directory is clean compared to $TargetRef." -ForegroundColor Green
    Write-Host "Checking last commit:" -ForegroundColor Yellow
    git log -1 --stat
    exit 0
}

Write-Host "`n[1] Modified and Untracked Files:" -ForegroundColor Yellow
$changes

# Categorize modified files
$files = git diff --name-only $TargetRef
$rustFiles = @($files | Where-Object { $_ -like "*.rs" })
$tsFiles = @($files | Where-Object { $_ -like "*.ts" -or $_ -like "*.tsx" })
$configFiles = @($files | Where-Object { $_ -like "*.json" -or $_ -like "*.toml" -or $_ -like "*.yml" -or $_ -like "*.yaml" })
$docFiles = @($files | Where-Object { $_ -like "*.md" })

Write-Host "`n[2] Blast Radius by Technology Domain:" -ForegroundColor Yellow
Write-Host ("  - Rust Backend Files:       " + $rustFiles.Count) -ForegroundColor $(if ($rustFiles.Count -gt 0) { "Red" } else { "Gray" })
Write-Host ("  - TypeScript/React Files:   " + $tsFiles.Count) -ForegroundColor $(if ($tsFiles.Count -gt 0) { "Yellow" } else { "Gray" })
Write-Host ("  - Configuration Files:      " + $configFiles.Count) -ForegroundColor $(if ($configFiles.Count -gt 0) { "Magenta" } else { "Gray" })
Write-Host ("  - Documentation Files:      " + $docFiles.Count) -ForegroundColor "Gray"

# Highlight high-risk security files
$securityKeywords = @("auth", "security", "token", "password", "session", "zk", "merkle", "admin", "secret")
$highRiskFiles = @($files | Where-Object {
    $f = $_.ToLower()
    foreach ($k in $securityKeywords) {
        if ($f.Contains($k)) { return $true }
    }
    return $false
})

if ($highRiskFiles.Count -gt 0) {
    Write-Host "`n[!] HIGH-RISK SECURITY SURFACE DETECTED:" -ForegroundColor Red
    foreach ($hr in $highRiskFiles) {
        Write-Host "    [!] $hr" -ForegroundColor Red
    }
}

Write-Host "`n[3] Diff Statistics:" -ForegroundColor Yellow
git diff --stat $TargetRef

Write-Host "`n[+] Reconnaissance complete. Proceed with deep adversarial inspection." -ForegroundColor Cyan

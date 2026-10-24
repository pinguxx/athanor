<#
.SYNOPSIS
  Automated UI/UX Consistency & Usability Auditor
.DESCRIPTION
  Scans the frontend React/Tailwind codebase for:
  - Color palette consistency (avoiding mixed gray/zinc/slate tokens)
  - Inline style bypasses (ensuring Tailwind utility class usage)
  - Accessibility affordances (missing aria-labels on icon buttons)
  - Clickable affordances (missing cursor-pointer on interactive elements)
  - Responsive padding and spacing scale compliance
#>

param(
    [string]$SrcDir = "c:\dev\social\frontend\src"
)

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "TWOSIDES UI/UX USABILITY & CONSISTENCY AUDITOR" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$issuesCount = 0
$warningsCount = 0

$componentFiles = Get-ChildItem -Path $SrcDir -Recurse -File -Include "*.tsx", "*.jsx" | Where-Object {
    $_.FullName -notmatch '\\(node_modules|dist|\.git)\\'
}

Write-Host "Auditing $($componentFiles.Count) frontend component files..." -ForegroundColor Gray

# ------------------------------------------------------------
# 1. PALETTE CONSISTENCY (Slate vs Gray/Zinc/Neutral)
# ------------------------------------------------------------
Write-Host "`n[1/5] Checking color palette consistency (Slate design system)..." -ForegroundColor Yellow

$inconsistentPalette = @()
foreach ($file in $componentFiles) {
    $matches = Select-String -Path $file.FullName -Pattern "(text|bg|border)-(gray|zinc|neutral)-[0-9]{2,3}"
    if ($matches) {
        $inconsistentPalette += $matches
    }
}

if ($inconsistentPalette.Count -gt 0) {
    Write-Host "  [WARN] Found $($inconsistentPalette.Count) occurrences of non-slate palette tokens (gray/zinc/neutral):" -ForegroundColor Yellow
    foreach ($m in $inconsistentPalette | Select-Object -First 5) {
        $relPath = $m.Path.Replace($SrcDir, "")
        Write-Host "     $($relPath):$($m.LineNumber) -> $($m.Line.Trim())" -ForegroundColor Yellow
    }
    if ($inconsistentPalette.Count -gt 5) {
        Write-Host "     ... and $($inconsistentPalette.Count - 5) more." -ForegroundColor Gray
    }
    $warningsCount += $inconsistentPalette.Count
} else {
    Write-Host "  [PASS] 100% unified slate palette. Zero conflicting gray/zinc/neutral tokens!" -ForegroundColor Green
}

# ------------------------------------------------------------
# 2. INLINE STYLE BYPASSES
# ------------------------------------------------------------
Write-Host "`n[2/5] Checking for inline style bypasses..." -ForegroundColor Yellow

$inlineStyles = @()
foreach ($file in $componentFiles) {
    $matches = Select-String -Path $file.FullName -Pattern "style=\{\{" | Where-Object {
        $_.Line -notmatch "width:" -and $_.Line -notmatch "transform:" -and $_.Line -notmatch "animation"
    }
    if ($matches) {
        $inlineStyles += $matches
    }
}

if ($inlineStyles.Count -gt 0) {
    Write-Host "  [WARN] Found $($inlineStyles.Count) inline styles that should be Tailwind classes:" -ForegroundColor Yellow
    foreach ($m in $inlineStyles | Select-Object -First 3) {
        $relPath = $m.Path.Replace($SrcDir, "")
        Write-Host "     $($relPath):$($m.LineNumber) -> $($m.Line.Trim())" -ForegroundColor Yellow
    }
    $warningsCount += $inlineStyles.Count
} else {
    Write-Host "  [PASS] No unwarranted inline styles bypassing the Tailwind design system." -ForegroundColor Green
}

# ------------------------------------------------------------
# 3. ACCESSIBILITY AFFORDANCES (ARIA-LABELS ON ICON BUTTONS)
# ------------------------------------------------------------
Write-Host "`n[3/5] Checking icon buttons for accessible labels (aria-label / title)..." -ForegroundColor Yellow

$missingLabels = 0
foreach ($file in $componentFiles) {
    $content = Get-Content -Path $file.FullName -Raw
    $buttonBlocks = [regex]::Matches($content, '<button[^>]*>[\s\S]*?<\/button>')
    
    foreach ($btn in $buttonBlocks) {
        $text = $btn.Value
        # Check if button contains an icon (e.g. <X, <Plus, <Share, <Flag) and no visible inner text
        if ($text -match '<[A-Z][a-zA-Z0-9]*\s+className=[^>]*\/>\s*<\/button>' -or $text -match '<[A-Z][a-zA-Z0-9]*\s*\/>\s*<\/button>') {
            if ($text -notmatch 'aria-label=' -and $text -notmatch 'title=') {
                $missingLabels++
            }
        }
    }
}

if ($missingLabels -gt 0) {
    Write-Host "  [WARN] Found $missingLabels icon-only buttons without explicit aria-label or title." -ForegroundColor Yellow
    $warningsCount += $missingLabels
} else {
    Write-Host "  [PASS] All icon-only buttons have accessible aria-labels or tooltips." -ForegroundColor Green
}

# ------------------------------------------------------------
# 4. CLICKABLE AFFORDANCES (cursor-pointer on interactive items)
# ------------------------------------------------------------
Write-Host "`n[4/5] Checking interactive elements for cursor-pointer affordance..." -ForegroundColor Yellow

$missingCursor = 0
foreach ($file in $componentFiles) {
    $lines = Get-Content -Path $file.FullName
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $l = $lines[$i]
        if ($l -match 'onClick=' -and $l -notmatch 'cursor-pointer' -and $l -notmatch '<button' -and $l -notmatch '<input') {
            # Non-button elements with onClick should declare cursor-pointer
            if ($l -match '<div|<span|<article') {
                $missingCursor++
            }
        }
    }
}

if ($missingCursor -gt 0) {
    Write-Host "  [WARN] Found $missingCursor interactive non-button elements (div/span) without 'cursor-pointer'." -ForegroundColor Yellow
    $warningsCount += $missingCursor
} else {
    Write-Host "  [PASS] All interactive elements exhibit explicit cursor-pointer affordances." -ForegroundColor Green
}

# ------------------------------------------------------------
# 5. RESPONSIVE BREATHING ROOM & PADDING
# ------------------------------------------------------------
Write-Host "`n[5/5] Checking responsive padding & anti-crowding spacing..." -ForegroundColor Yellow

$crampedLayouts = 0
foreach ($file in $componentFiles) {
    $matches = Select-String -Path $file.FullName -Pattern 'className="[^"]*p-0\.5[^"]*"'
    if ($matches) {
        $crampedLayouts += $matches.Count
    }
}

if ($crampedLayouts -gt 0) {
    Write-Host "  [INFO] $crampedLayouts micro-padding containers detected (ensure not used for major cards)." -ForegroundColor Gray
} else {
    Write-Host "  [PASS] Generous padding and spacing scales used across cards." -ForegroundColor Green
}

# ------------------------------------------------------------
# SUMMARY REPORT
# ------------------------------------------------------------
Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "UI/UX AUDIT SUMMARY" -ForegroundColor Cyan
if ($issuesCount -gt 0) { $blockerColor = "Red" } else { $blockerColor = "Green" }
if ($warningsCount -gt 0) { $warningColor = "Yellow" } else { $warningColor = "Green" }
Write-Host "  Layout / Visual Blockers: $issuesCount" -ForegroundColor $blockerColor
Write-Host "  Refinement / Nits Found:  $warningsCount" -ForegroundColor $warningColor
Write-Host "==========================================================" -ForegroundColor Cyan

if ($issuesCount -gt 0) {
    exit 1
} else {
    exit 0
}

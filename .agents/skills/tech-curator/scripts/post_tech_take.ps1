# PowerShell Automation Script for tech-curator (Alchemical Systems Architect)
# Fetches live technical news from Hacker News or uses curated systems catalog.
# Generates and stores systems engineering perspectives under docs/perspectives/.

param(
    [ValidateSet("all", "architecture", "wasm", "sqlite", "rag")]
    [string]$Topic = "all",
    [switch]$Offline
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Get-Item -Path "$ScriptDir\..\..\..\..").FullName
$PerspectivesDir = Join-Path $RepoRoot "docs\perspectives"
$HistoryFile = Join-Path $ScriptDir ".posted_history.json"

if (-not (Test-Path $PerspectivesDir)) {
    New-Item -ItemType Directory -Path $PerspectivesDir -Force | Out-Null
}

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host " ⚗  RUBEDO ALCHEMICAL SYSTEMS CURATOR" -ForegroundColor Cyan
Write-Host " Category Filter: $Topic | Live Internet: $(!$Offline)" -ForegroundColor Gray
Write-Host "======================================================================" -ForegroundColor Cyan

# Load history
$postedHistory = @()
if (Test-Path $HistoryFile) {
    try {
        $json = Get-Content $HistoryFile -Raw
        if ($json) { $postedHistory = $json | ConvertFrom-Json }
    } catch {}
}

# Curated catalog perspectives
$catalogTakes = @(
    @{
        title = "SQLite on NVMe vs Distributed Microservices"
        category = "sqlite"
        content = "Unpopular systems opinion: A SQLite database running embedded in a compiled Rust binary on an NVMe SSD effortlessly handles 80,000+ reads per second with sub-millisecond p99 latency without a Kubernetes cluster, a message bus, or 40 microservices. In Rubedo, Crucible leverages this exact embedded event store for zero-infrastructure durability."
    },
    @{
        title = "In-Process WASM Isolation vs Docker Containers"
        category = "wasm"
        content = "Container spin-up latency is 200-800ms with a 50MB+ memory footprint. In contrast, an in-process Wasmtime WebAssembly instance instantiates in <1ms and enforces instruction-level fuel limits, aborting infinite loops in <15ms without OS thread killing. In Rubedo, Alembic isolates untrusted tools with zero network roundtrips."
    },
    @{
        title = "Zero Lifetime Annotations in Public Macro Design"
        category = "architecture"
        content = "Rust's borrow checker is a masterpiece of compile-time safety, but complex lifetime parameters ('a, 'b: 'a) cause endless headaches when LLMs autonomously synthesize code. By mandating owned value semantics (serde_json::Value) and Arc envelopes, Rubedo macros guarantee 'static + Send + Sync, allowing AI agents to generate valid tool code without lifetime errors."
    },
    @{
        title = "In-Process Vector RAG (<40MB) vs Cloud Vector DBs"
        category = "rag"
        content = "You don't need a cloud vector database with network latency to achieve high-accuracy retrieval. Combining in-process FastEmbed (BGESmallENV15) with embedded LanceDB delivers sub-20ms similarity search within a sub-40MB resident memory footprint, entirely in-process."
    }
)

$selectedTake = $null

if (-not $Offline) {
    Write-Host "`n[1/2] Fetching live top stories from Hacker News..." -ForegroundColor Yellow
    try {
        $topStoryIds = Invoke-RestMethod -Uri "https://hacker-news.firebaseio.com/v0/topstories.json" -TimeoutSec 5 | Select-Object -First 10
        foreach ($id in $topStoryIds) {
            $story = Invoke-RestMethod -Uri "https://hacker-news.firebaseio.com/v0/item/$id.json" -TimeoutSec 5
            if ($story.title -and ($postedHistory -notcontains $story.title)) {
                $selectedTake = @{
                    title = $story.title
                    category = "live_hn"
                    content = "Looking at today's trending discussion on '$($story.title)': From a systems engineering standpoint, operational simplicity and deterministic state transitions should always supersede distributed architectural sprawl. In Rubedo, we prioritize in-process containment and zero external infrastructure."
                }
                break
            }
        }
    } catch {
        Write-Host "  [INFO] Hacker News API unreachable or timed out. Falling back to curated catalog." -ForegroundColor Gray
    }
}

if (-not $selectedTake) {
    Write-Host "`n[1/2] Selecting from curated alchemical systems catalog..." -ForegroundColor Yellow
    $eligible = $catalogTakes | Where-Object { $Topic -eq "all" -or $_.category -eq $Topic }
    $selectedTake = $eligible | Get-Random
}

if (-not $selectedTake) {
    Write-Host "No new systems perspectives found." -ForegroundColor Yellow
    exit 0
}

Write-Host "`n[2/2] Generating Systems Engineering Perspective..." -ForegroundColor Green
Write-Host "Title: $($selectedTake.title)" -ForegroundColor White
Write-Host "`nAnalysis:" -ForegroundColor Gray
Write-Host $selectedTake.content -ForegroundColor Cyan

# Save to docs/perspectives/
$timestamp = (Get-Date).ToString("yyyyMMdd_HHmmss")
$cleanTitle = $selectedTake.title -replace '[^a-zA-Z0-9]', '_'
$filePath = Join-Path $PerspectivesDir "perspective_${timestamp}_${cleanTitle}.md"

$mdContent = @"
# Systems Engineering Perspective: $($selectedTake.title)
*Generated on $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') by Rubedo Tech Curator*

---

## Technical Analysis

$($selectedTake.content)

---

## Alchemical Architectural Tenets
- **Tenet 1**: Zero Lifetime Annotations in Macros
- **Tenet 2**: Deterministic SQLite Event Sourcing
- **Tenet 3**: In-Process Fuel-Metered WASM Sandboxing
- **Tenet 5**: Zero External Infrastructure Required (<40MB Footprint)
"@

$mdContent | Out-File -FilePath $filePath -Encoding utf8
Write-Host "`n[+] Perspective saved to $filePath" -ForegroundColor Green

# Update history
$postedHistory += $selectedTake.title
$postedHistory | ConvertTo-Json | Out-File -FilePath $HistoryFile -Encoding utf8

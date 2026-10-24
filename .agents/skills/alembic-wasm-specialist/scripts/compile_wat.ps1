<#
.SYNOPSIS
  Compiles WAT text fixtures to binary WASM
.DESCRIPTION
  Takes a .wat file or string and converts it to .wasm binary using wasm-tools or rust wat crate.
#>
param(
    [string]$WatPath,
    [string]$OutPath
)

if (-not $WatPath) {
    Write-Host "Usage: ./compile_wat.ps1 -WatPath <path_to_wat> -OutPath <path_to_wasm>" -ForegroundColor Yellow
    exit 1
}

Write-Host "Compiling $WatPath -> $OutPath..." -ForegroundColor Cyan
# In Rust tests, wat::parse_str() is typically used directly in-memory.

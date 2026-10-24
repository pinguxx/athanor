<#
.SYNOPSIS
  Validates a JSON string against MCP JSON-RPC 2.0 structure
#>
param(
    [string]$JsonString
)

if (-not $JsonString) {
    Write-Host "Usage: ./verify_mcp_message.ps1 -JsonString '{\"jsonrpc\":\"2.0\",...}'" -ForegroundColor Yellow
    exit 1
}

try {
    $obj = $JsonString | ConvertFrom-Json
    if ($obj.jsonrpc -ne "2.0") {
        Write-Host "[-] Invalid jsonrpc version: expected '2.0', got '$($obj.jsonrpc)'" -ForegroundColor Red
        exit 1
    }
    Write-Host "[+] Valid JSON-RPC 2.0 message envelope." -ForegroundColor Green
} catch {
    Write-Host "[-] Malformed JSON: $_" -ForegroundColor Red
    exit 1
}

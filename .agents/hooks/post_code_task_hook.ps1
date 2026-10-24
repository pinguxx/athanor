# Antigravity PostToolUse Hook for write_file and edit_file
param()

try {
    # Read stdin context provided by Antigravity
    $stdin = ""
    if ([Console]::IsInputRedirected) {
        $stdin = [Console]::In.ReadToEnd()
    }

    # Log timestamp of modification
    $logDir = "$PSScriptRoot\..\.logs"
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }

    $entry = [PSCustomObject]@{
        timestamp = (Get-Date).ToString("o")
        event = "file_modified"
    }
    $entry | ConvertTo-Json -Compress | Out-File -FilePath "$logDir\recent_edits.log" -Append -Encoding utf8
} catch {}

# Output empty JSON object as mandated by Antigravity PostToolUse contract
[Console]::Out.WriteLine("{}")

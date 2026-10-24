# Antigravity Stop Hook executing when an agent turn finishes
param()

try {
    $stdin = ""
    if ([Console]::IsInputRedirected) {
        $stdin = [Console]::In.ReadToEnd()
    }
} catch {}

# Output empty JSON object as mandated by Antigravity Stop contract
[Console]::Out.WriteLine("{}")

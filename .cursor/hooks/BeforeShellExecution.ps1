. (Join-Path $PSScriptRoot "Common.ps1")

$payload = Read-HookPayload
$command = $null

foreach ($name in @("command", "command_line", "commandLine", "raw_command")) {
    if ($payload.PSObject.Properties.Name -contains $name) {
        $value = $payload.$name
        if (-not [string]::IsNullOrWhiteSpace($value)) {
            $command = [string]$value
            break
        }
    }
}

if (-not $command -and ($payload.PSObject.Properties.Name -contains "args")) {
    $command = (@($payload.args) -join " ")
}

if (-not $command -and ($payload.PSObject.Properties.Name -contains "raw_input")) {
    $command = [string]$payload.raw_input
}

$blockedPatterns = @(
    "\bgit\s+reset\s+--hard\b",
    "\bgit\s+checkout\s+--\b",
    "\bgit\s+clean\s+-f(?:[dx]+)?\b",
    "\bRemove-Item\b.*\b-Recurse\b",
    "(?:^|\s)del(?:\.exe)?\s+.*(?:^| )/s(?:\s|$)",
    "(?:^|\s)rd(?:\.exe)?\s+.*(?:^| )/s(?:\s|$)",
    "(?:^|\s)rmdir(?:\.exe)?\s+.*(?:^| )/s(?:\s|$)"
)

$isBlocked = $false
foreach ($pattern in $blockedPatterns) {
    if ($command -match $pattern) {
        $isBlocked = $true
        break
    }
}

if ($isBlocked) {
    Write-HookResponse -Response ([ordered]@{
        continue = $true
        permission = "deny"
        user_message = "TrendLab repo-local hook blocked a destructive shell command."
        agent_message = "Blocked a shell command that matches the repo's destructive-command denylist."
    })
    exit 0
}

Write-HookResponse -Response ([ordered]@{
    continue = $true
    permission = "allow"
})

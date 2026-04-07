. (Join-Path $PSScriptRoot "Common.ps1")

$payload = Read-HookPayload
$sessionId = Get-SessionId -Payload $payload
$state = Read-State -SessionId $sessionId

$allTouched = @()
$allTouched += @($state.touched_core_or_data)
$allTouched += @($state.touched_repo_guidance)
$allTouched = $allTouched | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique

if (@($allTouched).Count -eq 0) {
    Remove-State -SessionId $sessionId
    Write-HookResponse -Response ([ordered]@{})
    exit 0
}

$preview = ($allTouched | Select-Object -First 5) -join ", "
$message = "TrendLab reminder: this session touched protected files. Review docs/Status.md and any impacted contracts before considering the task complete. Files: $preview"

Remove-State -SessionId $sessionId

Write-HookResponse -Response ([ordered]@{
    user_message = $message
    agent_message = $message
})

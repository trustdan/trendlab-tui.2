. (Join-Path $PSScriptRoot "Common.ps1")

$payload = Read-HookPayload
$relativePath = Get-RelativeRepoPath -Payload $payload
$sessionId = Get-SessionId -Payload $payload

$protectedContracts = @(
    "docs/Artifacts.md",
    "docs/BarSemantics.md",
    "docs/Invariants.md",
    "docs/MathContract.md",
    "docs/Plan.md",
    "docs/Roadmap.md",
    "docs/Workspace.md"
)

$isCoreOrData = $false
$isRepoGuidance = $false

if ($relativePath) {
    if ($relativePath -match "^crates/(trendlab-core|trendlab-data)/") {
        $isCoreOrData = $true
    }

    if (
        $relativePath -eq "AGENTS.md" -or
        $relativePath -match "^\.cursor/rules/" -or
        $protectedContracts -contains $relativePath
    ) {
        $isRepoGuidance = $true
    }
}

if ($isCoreOrData -or $isRepoGuidance) {
    $state = Read-State -SessionId $sessionId

    if ($isCoreOrData) {
        $state.touched_core_or_data = Add-UniqueStatePath -Items @($state.touched_core_or_data) -Value $relativePath
    }

    if ($isRepoGuidance) {
        $state.touched_repo_guidance = Add-UniqueStatePath -Items @($state.touched_repo_guidance) -Value $relativePath
    }

    Write-State -SessionId $sessionId -State $state
}

$messages = @()
if ($isCoreOrData) {
    $messages += "TrendLab reminder: edits under trendlab-core/trendlab-data usually require contract-doc and golden-test review."
}
if ($isRepoGuidance) {
    $messages += "TrendLab reminder: repo guidance files changed; keep AGENTS, rules, and core contracts aligned."
}

if ($messages.Count -eq 0) {
    Write-HookResponse -Response ([ordered]@{})
    exit 0
}

$messageText = ($messages -join " ")
Write-HookResponse -Response ([ordered]@{
    user_message = $messageText
    agent_message = $messageText
})

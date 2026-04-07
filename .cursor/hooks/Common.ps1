Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Read-HookPayload {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) {
        return [pscustomobject]@{}
    }

    try {
        return $raw | ConvertFrom-Json
    } catch {
        return [pscustomobject]@{
            raw_input = $raw
            parse_error = $_.Exception.Message
        }
    }
}

function Write-HookResponse {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Response
    )

    $Response | ConvertTo-Json -Compress -Depth 20 | Write-Output
}

function Normalize-HookPath {
    param(
        [AllowNull()]
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }

    $normalized = $Path -replace "\\", "/"
    if ($normalized -match "^/[A-Za-z]:/") {
        $normalized = $normalized.Substring(1)
    }

    return $normalized -replace "/", "\"
}

function Get-WorkspaceRoot {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Payload
    )

    $candidates = @()
    if ($Payload.PSObject.Properties.Name -contains "workspace_roots") {
        $candidates += @($Payload.workspace_roots)
    }
    if ($Payload.PSObject.Properties.Name -contains "workspaceRoots") {
        $candidates += @($Payload.workspaceRoots)
    }

    foreach ($candidate in $candidates) {
        $normalized = Normalize-HookPath -Path $candidate
        if (-not [string]::IsNullOrWhiteSpace($normalized)) {
            return $normalized.TrimEnd("\")
        }
    }

    return $null
}

function Get-FilePath {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Payload
    )

    $propertyNames = @("file_path", "filePath", "path")
    foreach ($name in $propertyNames) {
        if ($Payload.PSObject.Properties.Name -contains $name) {
            $value = $Payload.$name
            $normalized = Normalize-HookPath -Path $value
            if (-not [string]::IsNullOrWhiteSpace($normalized)) {
                return $normalized
            }
        }
    }

    return $null
}

function Get-RelativeRepoPath {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Payload
    )

    $filePath = Get-FilePath -Payload $Payload
    if (-not $filePath) {
        return $null
    }

    $workspaceRoot = Get-WorkspaceRoot -Payload $Payload
    if (-not $workspaceRoot) {
        return $filePath
    }

    if ($filePath.StartsWith($workspaceRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        $relative = $filePath.Substring($workspaceRoot.Length).TrimStart("\")
        if (-not [string]::IsNullOrWhiteSpace($relative)) {
            return $relative -replace "\\", "/"
        }
    }

    return $filePath -replace "\\", "/"
}

function Get-SessionId {
    param(
        [Parameter(Mandatory = $true)]
        [object]$Payload
    )

    foreach ($name in @("conversation_id", "conversationId", "generation_id", "generationId")) {
        if ($Payload.PSObject.Properties.Name -contains $name) {
            $value = $Payload.$name
            if (-not [string]::IsNullOrWhiteSpace($value)) {
                return $value
            }
        }
    }

    return "default"
}

function Get-StateDirectory {
    $baseDir = Join-Path $env:TEMP "trendlab-cursor-hooks"
    if (-not (Test-Path -LiteralPath $baseDir)) {
        New-Item -ItemType Directory -Path $baseDir -Force | Out-Null
    }
    return $baseDir
}

function Get-StateFilePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SessionId
    )

    $safeSessionId = ($SessionId -replace '[^A-Za-z0-9._-]', "_")
    return Join-Path (Get-StateDirectory) "$safeSessionId.json"
}

function Read-State {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SessionId
    )

    $path = Get-StateFilePath -SessionId $SessionId
    if (-not (Test-Path -LiteralPath $path)) {
        return [pscustomobject]@{
            touched_core_or_data = @()
            touched_repo_guidance = @()
        }
    }

    try {
        return Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    } catch {
        return [pscustomobject]@{
            touched_core_or_data = @()
            touched_repo_guidance = @()
        }
    }
}

function Write-State {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SessionId,
        [Parameter(Mandatory = $true)]
        [object]$State
    )

    $path = Get-StateFilePath -SessionId $SessionId
    $State | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $path -Encoding utf8
}

function Remove-State {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SessionId
    )

    $path = Get-StateFilePath -SessionId $SessionId
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }
}

function Add-UniqueStatePath {
    param(
        [object[]]$Items = @(),
        [Parameter(Mandatory = $true)]
        [string]$Value
    )

    $existing = @($Items)
    if ($existing -contains $Value) {
        return $existing
    }

    return @($existing + $Value)
}

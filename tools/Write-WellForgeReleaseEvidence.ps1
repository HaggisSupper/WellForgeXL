[CmdletBinding()]
param(
    [string]$OutputDirectory,
    [string]$EvidencePath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repositoryRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
if ([string]::IsNullOrWhiteSpace($EvidencePath)) { $EvidencePath = Join-Path $repositoryRoot 'outputs\release-evidence.json' }

function Get-FileEvidence {
    param([Parameter(Mandatory = $true)][string]$Path)
    $exists = Test-Path -LiteralPath $Path -PathType Leaf
    [ordered]@{
        path = $Path
        exists = $exists
        sha256 = if ($exists) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
        size_bytes = if ($exists) { (Get-Item -LiteralPath $Path).Length } else { $null }
    }
}

function Get-ExecutableEvidence {
    param([Parameter(Mandatory = $true)][string]$Path)
    $file = Get-FileEvidence -Path $Path
    $manifestPath = $Path + '.sha256'
    $manifestExists = Test-Path -LiteralPath $manifestPath -PathType Leaf
    $expectedHash = if ($manifestExists) { (Get-Content -LiteralPath $manifestPath -Raw).Trim().ToLowerInvariant() } else { $null }
    [ordered]@{
        file = $file
        hash_manifest = Get-FileEvidence -Path $manifestPath
        expected_sha256 = $expectedHash
        hash_match = [bool]($file.exists -and $manifestExists -and $expectedHash -eq $file.sha256)
    }
}

$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsm',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsm',
    'Torque_Drag_and_Buckling_SI.xlsm',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsm',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsm'
)

$workbooks = @($workbookNames | ForEach-Object { Get-FileEvidence -Path (Join-Path $OutputDirectory $_) })
$bhaExecutable = Get-ExecutableEvidence -Path (Join-Path $OutputDirectory 'wellforge-bha.exe')
$trajectoryExecutable = Get-ExecutableEvidence -Path (Join-Path $OutputDirectory 'wellforge-trajectory.exe')
$logDirectory = Join-Path $repositoryRoot 'logs'
$logs = if (Test-Path -LiteralPath $logDirectory -PathType Container) {
    @(Get-ChildItem -LiteralPath $logDirectory -Filter '*.jsonl' -File | Sort-Object Name | ForEach-Object { Get-FileEvidence -Path $_.FullName })
} else { @() }
$allWorkbooksExist = @($workbooks | Where-Object { -not $_.exists }).Count -eq 0
$passed = $allWorkbooksExist -and $bhaExecutable.hash_match -and $trajectoryExecutable.hash_match -and $logs.Count -gt 0

$evidence = [ordered]@{
    schema_version = '1.0.0'
    generated_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    git_sha = $env:GITHUB_SHA
    rust_toolchain = [ordered]@{
        rustc = (@(& rustc --version) -join "`n")
        cargo = (@(& cargo --version) -join "`n")
    }
    bha_executable = $bhaExecutable
    trajectory_executable = $trajectoryExecutable
    workbooks = $workbooks
    logs = $logs
    overall_status = if ($passed) { 'passed' } else { 'failed' }
}

$evidenceDirectory = Split-Path -Parent $EvidencePath
New-Item -ItemType Directory -Path $evidenceDirectory -Force | Out-Null
$evidence | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $EvidencePath -Encoding UTF8
Write-Host ("Release evidence written: {0} ({1})" -f $EvidencePath, $evidence.overall_status)
if (-not $passed) { exit 1 }

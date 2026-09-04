[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidateSet('wellforge-bha-cli', 'wellforge-trajectory-cli', 'wellforge-torque-drag-cli', 'wellforge-hydraulics-cli')][string]$EnginePackage,
    [Parameter(Mandatory = $true)][ValidateSet('wellforge-bha.exe', 'wellforge-trajectory.exe', 'wellforge-torque-drag.exe', 'wellforge-hydraulics.exe')][string]$ExecutableName,
    [string]$OutputDirectory,
    [string]$LogDirectory,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$engineRoot = Join-Path $repositoryRoot 'engine'
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
if (-not [System.IO.Path]::IsPathRooted($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot $OutputDirectory }
$OutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
if ([string]::IsNullOrWhiteSpace($LogDirectory)) { $LogDirectory = Join-Path $repositoryRoot 'logs' }
New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
$logPath = Join-Path $LogDirectory ('{0}-build-{1}.jsonl' -f $ExecutableName.Replace('.exe', ''), (Get-Date -Format 'yyyyMMdd-HHmmss'))
$succeeded = $false
$locationPushed = $false

function Get-FileHash {
    param([Parameter(Mandatory = $true)][string]$LiteralPath)
    $stream = $null
    $hasher = $null
    try {
        $stream = [System.IO.File]::OpenRead($LiteralPath)
        $hasher = [System.Security.Cryptography.SHA256]::Create()
        return [pscustomobject]@{ Hash = [System.BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '') }
    }
    finally {
        if ($null -ne $hasher) { $hasher.Dispose() }
        if ($null -ne $stream) { $stream.Dispose() }
    }
}

function Write-EngineEvent {
    param([string]$Level, [string]$Message)
    [ordered]@{ timestamp = (Get-Date).ToUniversalTime().ToString('o'); level = $Level; message = $Message } |
        ConvertTo-Json -Compress | Add-Content -LiteralPath $logPath -Encoding UTF8
    Write-Host ('[{0}] {1}' -f $Level, $Message)
}

try {
    Write-EngineEvent INFO ("Building {0} as {1} with pinned Rust 1.98.0." -f $EnginePackage, $ExecutableName)
    $rustIdentity = @(& rustup run 1.98.0 rustc --version --verbose)
    if ($LASTEXITCODE -ne 0 -or $rustIdentity.Count -eq 0 -or $rustIdentity[0] -notmatch '^rustc 1\.98\.0(?:\s|$)') {
        throw 'Expected Rust 1.98.0 toolchain identity.'
    }
    $rustIdentity | ForEach-Object { Write-EngineEvent INFO $_ }
    Push-Location $engineRoot
    $locationPushed = $true
    & cargo +1.98.0 fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt failed' }
    & cargo +1.98.0 clippy --workspace --all-targets --locked --offline -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'cargo clippy failed' }
    & cargo +1.98.0 test --workspace --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'cargo test failed' }
    & cargo +1.98.0 build --release --locked --offline -p $EnginePackage
    if ($LASTEXITCODE -ne 0) { throw 'cargo release build failed' }

    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $builtExecutable = Join-Path $engineRoot ('target\release\{0}' -f $ExecutableName)
    if (-not (Test-Path -LiteralPath $builtExecutable -PathType Leaf)) { throw "Release executable was not found: $builtExecutable" }
    $targetExecutable = Join-Path $OutputDirectory $ExecutableName
    Copy-Item -LiteralPath $builtExecutable -Destination $targetExecutable -Force
    $engineHash = (Get-FileHash -LiteralPath $targetExecutable).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText(($targetExecutable + '.sha256'), $engineHash, [System.Text.UTF8Encoding]::new($false))
    Write-EngineEvent INFO ("Engine SHA-256: {0}" -f $engineHash)
    $succeeded = $true
    Write-EngineEvent SUCCESS ("{0} build completed: {1}" -f $ExecutableName, $targetExecutable)
}
catch {
    Write-EngineEvent ERROR $_.Exception.Message
    Write-Host ($_ | Format-List * -Force | Out-String) -ForegroundColor Red
}
finally {
    if ($locationPushed) { Pop-Location }
    Write-Host ('Full JSONL log: {0}' -f $logPath)
    if (-not $NoPause) { [void](Read-Host 'Press Enter to close this window') }
}

if (-not $succeeded) { exit 1 }

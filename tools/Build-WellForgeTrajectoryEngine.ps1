[CmdletBinding()]
param(
    [string]$OutputDirectory,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$engineRoot = Join-Path $repositoryRoot 'engine'
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
$logDirectory = Join-Path $repositoryRoot 'logs'
New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
$logPath = Join-Path $logDirectory ('trajectory-engine-build-{0}.jsonl' -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$succeeded = $false

function Write-EngineEvent {
    param([string]$Level, [string]$Message)
    [ordered]@{ timestamp = (Get-Date).ToUniversalTime().ToString('o'); level = $Level; message = $Message } |
        ConvertTo-Json -Compress | Add-Content -LiteralPath $logPath -Encoding UTF8
    Write-Host ('[{0}] {1}' -f $Level, $Message)
}

try {
    Write-EngineEvent INFO 'Verifying pinned Rust 1.98.0 toolchain identity.'
    $rustIdentity = @(& rustup run 1.98.0 rustc --version --verbose)
    if ($LASTEXITCODE -ne 0) { throw 'rustc identity check failed' }
    $rustIdentity | ForEach-Object { Write-EngineEvent INFO $_ }
    if ($rustIdentity.Count -eq 0 -or $rustIdentity[0] -notmatch '^rustc 1\.98\.0(?:\s|$)') {
        throw ('Expected rustc 1.98.0, received: {0}' -f ($rustIdentity -join '; '))
    }
    & cargo +1.98.0 --version --verbose | ForEach-Object { Write-EngineEvent INFO $_ }
    if ($LASTEXITCODE -ne 0) { throw 'cargo identity check failed' }
    Set-Location $engineRoot
    & cargo +1.98.0 fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt failed' }
    & cargo +1.98.0 clippy --workspace --all-targets --locked --offline -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'trajectory workspace clippy failed' }
    & cargo +1.98.0 test --workspace --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'trajectory workspace tests failed' }
    & cargo +1.98.0 build --release --locked --offline -p wellforge-trajectory-cli
    if ($LASTEXITCODE -ne 0) { throw 'trajectory release build failed' }

    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $builtExecutable = Join-Path $engineRoot 'target\release\wellforge-trajectory.exe'
    if (-not (Test-Path -LiteralPath $builtExecutable -PathType Leaf)) { throw "Release executable was not found: $builtExecutable" }
    $targetExecutable = Join-Path $OutputDirectory 'wellforge-trajectory.exe'
    Copy-Item -LiteralPath $builtExecutable -Destination $targetExecutable -Force
    $engineHash = (Get-FileHash -LiteralPath $targetExecutable -Algorithm SHA256).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText(($targetExecutable + '.sha256'), $engineHash, [System.Text.UTF8Encoding]::new($false))
    Write-EngineEvent INFO ("Engine SHA-256: {0}" -f $engineHash)
    $succeeded = $true
    Write-EngineEvent SUCCESS ("WellForge trajectory Rust engine build completed: {0}" -f $targetExecutable)
}
catch {
    Write-EngineEvent ERROR $_.Exception.Message
    Write-Host ($_ | Format-List * -Force | Out-String) -ForegroundColor Red
}
finally {
    Write-Host ('Full JSONL log: {0}' -f $logPath)
    if (-not $NoPause) { [void](Read-Host 'Press Enter to close this window') }
}

if (-not $succeeded) { exit 1 }

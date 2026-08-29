[CmdletBinding()]
param(
    [string]$OutputDirectory,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$engineRoot = Join-Path $repositoryRoot 'engine'
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
$logDirectory = Join-Path $repositoryRoot 'logs'
New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
$logPath = Join-Path $logDirectory ('bha-engine-build-{0}.jsonl' -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$succeeded = $false

function Write-EngineEvent {
    param([string]$Level, [string]$Message)
    [ordered]@{ timestamp = (Get-Date).ToUniversalTime().ToString('o'); level = $Level; message = $Message } |
        ConvertTo-Json -Compress | Add-Content -LiteralPath $logPath -Encoding UTF8
    Write-Host ('[{0}] {1}' -f $Level, $Message)
}

try {
    Write-EngineEvent INFO 'Verifying Rust toolchain identity.'
    & rustc --version --verbose | ForEach-Object { Write-EngineEvent INFO $_ }
    & cargo --version --verbose | ForEach-Object { Write-EngineEvent INFO $_ }
    Set-Location $engineRoot
    & cargo fmt --check
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt failed' }
    & cargo clippy --workspace --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'cargo clippy failed' }
    & cargo test --workspace --all-features --locked
    if ($LASTEXITCODE -ne 0) { throw 'cargo test failed' }
    if ($null -eq (Get-Command cargo-deny -ErrorAction SilentlyContinue)) {
        Write-EngineEvent INFO 'Installing the pinned cargo-deny release for the dependency-policy gate.'
        & cargo install cargo-deny --version 0.20.2 --locked
        if ($LASTEXITCODE -ne 0) { throw 'cargo-deny installation failed' }
    }
    & cargo deny check licenses bans sources
    if ($LASTEXITCODE -ne 0) { throw 'cargo-deny policy failed' }
    & cargo build --release --locked -p wellforge-bha-cli
    if ($LASTEXITCODE -ne 0) { throw 'release build failed' }
    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $builtExecutable = Join-Path $engineRoot 'target\release\wellforge-bha.exe'
    if (-not (Test-Path -LiteralPath $builtExecutable -PathType Leaf)) { throw "Release executable was not found: $builtExecutable" }
    $targetExecutable = Join-Path $OutputDirectory 'wellforge-bha.exe'
    Copy-Item -LiteralPath $builtExecutable -Destination $targetExecutable -Force
    $engineHash = (Get-FileHash -LiteralPath $targetExecutable -Algorithm SHA256).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText(($targetExecutable + '.sha256'), $engineHash, [System.Text.UTF8Encoding]::new($false))
    Write-EngineEvent INFO ("Engine SHA-256: {0}" -f $engineHash)
    $succeeded = $true
    Write-EngineEvent SUCCESS ("WellForge BHA Rust engine build completed: {0}" -f $targetExecutable)
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

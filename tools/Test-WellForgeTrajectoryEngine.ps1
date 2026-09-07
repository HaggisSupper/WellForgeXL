[CmdletBinding()]
param(
    [string]$EngineDirectory,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'WellForgeBuildGuards.ps1')
$repositoryRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($EngineDirectory)) { $EngineDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
$executable = Join-Path $EngineDirectory 'wellforge-trajectory.exe'
$hashManifest = $executable + '.sha256'
$fixture = Join-Path $repositoryRoot 'engine\fixtures\requests\trajectory-release-one-minimal.json'
$temporaryParent = [System.IO.Path]::GetFullPath((Join-Path ([System.IO.Path]::GetTempPath()) 'WellForgeTrajectory'))
$runName = 'release-test-' + [guid]::NewGuid().ToString('N')
$runRoot = Join-Path $temporaryParent $runName
$runDirectoryCreated = $false
$result = Join-Path $runRoot 'result.json'
$diagnostics = Join-Path $runRoot 'diagnostics.jsonl'
$bridge = Join-Path $runRoot 'result.wfbridge'
$succeeded = $false

try {
    foreach ($required in @($executable, $hashManifest, $fixture)) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Required file was not found: $required" }
    }
    $expectedHash = ([System.IO.File]::ReadAllText($hashManifest)).Trim().ToLowerInvariant()
    if ($expectedHash -notmatch '^[0-9a-f]{64}$') { throw 'Trajectory executable hash manifest is invalid.' }
    $actualHash = (Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) { throw 'Trajectory executable hash mismatch.' }

    Assert-WellForgeCleanupTarget -Path $runRoot -RunDirectory $temporaryParent -ExpectedChildName $runName
    New-Item -ItemType Directory -Path $temporaryParent -Force | Out-Null
    Assert-WellForgeCleanupTarget -Path $runRoot -RunDirectory $temporaryParent -ExpectedChildName $runName
    New-WellForgeExclusiveDirectory -LiteralPath $runRoot
    $runDirectoryCreated = $true
    & $executable validate --input $fixture
    if ($LASTEXITCODE -ne 0) { throw 'validate failed' }
    & $executable run --input $fixture --output $result --diagnostics $diagnostics --no-backup
    if ($LASTEXITCODE -ne 0) { throw 'run failed' }
    if (-not (Test-Path -LiteralPath $diagnostics -PathType Leaf)) { throw 'diagnostics were not written' }
    $diagnostic = Get-Content -LiteralPath $diagnostics -Encoding UTF8 | Select-Object -First 1 | ConvertFrom-Json
    $requestHash = [string]$diagnostic.request_hash
    if ($requestHash -notmatch '^[0-9a-f]{64}$') { throw 'diagnostics did not contain a canonical request hash' }
    & $executable verify-result --input $result --request-hash $requestHash
    if ($LASTEXITCODE -ne 0) { throw 'verify-result failed' }
    & $executable bridge --input $result --output $bridge --request-hash $requestHash
    if ($LASTEXITCODE -ne 0) { throw 'bridge failed' }
    if (-not (Test-Path -LiteralPath $bridge -PathType Leaf)) { throw 'bridge was not written' }
    $bridgeHeader = Get-Content -LiteralPath $bridge -Encoding UTF8 | Select-Object -First 1
    if (-not $bridgeHeader.StartsWith("H`t1.0.0`t")) { throw 'bridge header is invalid' }
    $succeeded = $true
}
catch {
    Write-Host ($_ | Format-List * -Force | Out-String) -ForegroundColor Red
}
finally {
    if ($runDirectoryCreated) {
        try {
            Assert-WellForgeCleanupTarget -Path $runRoot -RunDirectory $temporaryParent -ExpectedChildName $runName
            if (Test-Path -LiteralPath $runRoot) {
                if (-not (Test-Path -LiteralPath $runRoot -PathType Container)) { throw 'The owned run directory is no longer a directory.' }
                Assert-WellForgeCleanupTarget -Path $runRoot -RunDirectory $temporaryParent -ExpectedChildName $runName
                Remove-Item -LiteralPath $runRoot -Recurse -Force
            }
        }
        catch {
            $succeeded = $false
            Write-Host "Blocked/manual cleanup: temporary material retained at '$runRoot'. $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    elseif (Test-Path -LiteralPath $runRoot) {
        $succeeded = $false
        Write-Host "Blocked/manual cleanup: this invocation did not create '$runRoot'; existing material was retained." -ForegroundColor Red
    }
    if ($succeeded) { Write-Host 'WellForge trajectory engine release test passed.' -ForegroundColor Green }
    if (-not $NoPause) { [void](Read-Host 'Press Enter to close this window') }
}

if (-not $succeeded) { exit 1 }

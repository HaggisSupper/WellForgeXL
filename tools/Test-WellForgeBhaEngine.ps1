[CmdletBinding()]
param(
    [string]$EnginePath,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($EnginePath)) { $EnginePath = Join-Path $repositoryRoot 'outputs\vba-engine\wellforge-bha.exe' }
$requestPath = Join-Path $repositoryRoot 'engine\fixtures\requests\release-one-minimal.json'
$resultPath = Join-Path $env:TEMP ('wellforge-bha-release-test-{0}.json' -f [guid]::NewGuid().ToString('N'))
$succeeded = $false
try {
    if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) { throw "Engine not found: $EnginePath" }
    $requestHash = (& $EnginePath validate --input $requestPath).Trim()
    if ($LASTEXITCODE -ne 0 -or $requestHash.Length -ne 64) { throw 'Request validation/hash test failed.' }
    & $EnginePath run --input $requestPath --output $resultPath
    if ($LASTEXITCODE -ne 0) { throw 'Static/modal fixture run failed.' }
    & $EnginePath verify-result --input $resultPath --request-hash $requestHash
    if ($LASTEXITCODE -ne 0) { throw 'Result hash verification failed.' }
    $result = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
    if (-not $result.evidence.converged) { throw 'Fixture did not report convergence.' }
    if ($result.static_nodes.Count -lt 2 -or $result.modes.Count -lt 1 -or $result.frequency_response.Count -lt 10) { throw 'Result arrays are incomplete.' }
    $succeeded = $true
    Write-Host 'WellForge BHA Rust engine release test passed.' -ForegroundColor Green
}
catch {
    Write-Host ($_ | Format-List * -Force | Out-String) -ForegroundColor Red
}
finally {
    if (Test-Path -LiteralPath $resultPath) { Remove-Item -LiteralPath $resultPath -Force }
    if (-not $NoPause) { [void](Read-Host 'Press Enter to close this window') }
}
if (-not $succeeded) { exit 1 }

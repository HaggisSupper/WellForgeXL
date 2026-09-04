[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$GateResultsPath,
    [Parameter(Mandatory = $true)][string]$ArchivePath,
    [Parameter(Mandatory = $true)][string]$SupportRoot,
    [Parameter(Mandatory = $true)][string]$EvidencePath,
    [Parameter(Mandatory = $true)][string]$ExpectedGitSha,
    [Parameter(Mandatory = $true)][string]$RunId
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$releaseTool = Join-Path $repositoryRoot 'tools\release-package.mjs'
$evidenceDirectory = Split-Path -Parent $EvidencePath
New-Item -ItemType Directory -Path $evidenceDirectory -Force | Out-Null

& node $releaseTool evidence `
    --gate-results $GateResultsPath `
    --archive $ArchivePath `
    --support-root $SupportRoot `
    --output $EvidencePath `
    --git-sha $ExpectedGitSha `
    --run-id $RunId

if ($LASTEXITCODE -ne 0) {
    Write-Host "Release evidence is fail-closed: $EvidencePath" -ForegroundColor Red
    exit 1
}
Write-Host "Release evidence passed: $EvidencePath" -ForegroundColor Green

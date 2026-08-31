[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RunRoot,
    [Parameter(Mandatory = $true)][string]$ExpectedGitSha,
    [Parameter(Mandatory = $true)][string]$RunId,
    [int]$TimeoutMinutes = 75
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$releaseScript = Join-Path $PSScriptRoot 'Invoke-WellForgeWindowsRelease.ps1'
$logDirectory = Join-Path $RunRoot 'logs'
$gateResultsPath = Join-Path $RunRoot 'gate-results.json'
New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
$stdoutPath = Join-Path $logDirectory 'windows-acceptance.stdout.log'
$stderrPath = Join-Path $logDirectory 'windows-acceptance.stderr.log'
$baselineExcelProcessIds = @()
foreach ($excelProcess in @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue)) {
    $baselineExcelProcessIds += $excelProcess.Id
}

function Stop-NewExcelProcesses {
    foreach ($excelProcess in @(Get-Process -Name EXCEL -ErrorAction SilentlyContinue)) {
        if ($baselineExcelProcessIds -notcontains $excelProcess.Id) {
            & taskkill.exe /PID $excelProcess.Id /T /F | Out-Null
        }
    }
}

$arguments = @(
    '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ('"{0}"' -f $releaseScript),
    '-RunRoot', ('"{0}"' -f $RunRoot), '-ExpectedGitSha', $ExpectedGitSha, '-RunId', ('"{0}"' -f $RunId)
)
$process = Start-Process -FilePath 'powershell.exe' -ArgumentList $arguments -PassThru `
    -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
$finished = $process.WaitForExit($TimeoutMinutes * 60 * 1000)

if (-not $finished) {
    & taskkill.exe /PID $process.Id /T /F | Out-Null
    Stop-NewExcelProcesses
    if (-not (Test-Path -LiteralPath $gateResultsPath -PathType Leaf)) {
        $gates = [ordered]@{}
        foreach ($name in @('native_binaries', 'vba_compilation_excel_com', 'unit_switching', 'chart_rendering', 'rollback_runtime', 'package_acceptance')) {
            $gates[$name] = [ordered]@{ status = 'pending' }
        }
        $gates['native_binaries'] = [ordered]@{ status = 'failed'; error = "Windows acceptance exceeded $TimeoutMinutes minutes." }
        $gateDocument = [ordered]@{
            schema_version = '1.0.0'
            run_id = $RunId
            git_sha = $ExpectedGitSha.ToLowerInvariant()
            gates = $gates
            failure = "Windows acceptance exceeded $TimeoutMinutes minutes and the process tree was terminated."
        }
        $json = ($gateDocument | ConvertTo-Json -Depth 8) + [Environment]::NewLine
        [System.IO.File]::WriteAllText($gateResultsPath, $json, [System.Text.UTF8Encoding]::new($false))
    }
    Write-Host "Windows acceptance timed out after $TimeoutMinutes minutes." -ForegroundColor Red
    exit 124
}

$process.WaitForExit()
Get-Content -LiteralPath $stdoutPath -ErrorAction SilentlyContinue | Write-Host
Get-Content -LiteralPath $stderrPath -ErrorAction SilentlyContinue | Write-Host
if ($process.ExitCode -ne 0) {
    Stop-NewExcelProcesses
    exit $process.ExitCode
}

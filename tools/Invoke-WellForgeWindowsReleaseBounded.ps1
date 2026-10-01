[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RunRoot,
    [Parameter(Mandatory = $true)][string]$ExpectedGitSha,
    [Parameter(Mandatory = $true)][string]$RunId,
    [ValidateRange(1, 1440)][int]$TimeoutMinutes = 75
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$releaseScript = Join-Path $PSScriptRoot 'Invoke-WellForgeWindowsRelease.ps1'
$logDirectory = Join-Path $RunRoot 'logs'
$gateResultsPath = Join-Path $RunRoot 'gate-results.json'
New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
$stdoutPath = Join-Path $logDirectory 'windows-acceptance.stdout.log'
$stderrPath = Join-Path $logDirectory 'windows-acceptance.stderr.log'
function Save-BlockedCleanup {
    param([string]$Reason)
    $message = "$Reason Blocked/manual cleanup: inspect remaining build helpers and Excel workbooks; no Excel process or unproven descendant was terminated."
    $gateDocument = $null
    if (Test-Path -LiteralPath $gateResultsPath -PathType Leaf) {
        try { $gateDocument = Get-Content -LiteralPath $gateResultsPath -Raw | ConvertFrom-Json } catch { }
    }
    if ($null -eq $gateDocument) {
        $gates = [ordered]@{}
        foreach ($name in @('native_binaries', 'vba_compilation_excel_com', 'unit_switching', 'chart_rendering', 'rollback_runtime', 'package_acceptance')) {
            $gates[$name] = [ordered]@{ status = 'pending' }
        }
        $gateDocument = [pscustomobject]@{ schema_version = '1.0.0'; run_id = $RunId; git_sha = $ExpectedGitSha.ToLowerInvariant(); gates = [pscustomobject]$gates }
    }
    $gateDocument | Add-Member -NotePropertyName failure -NotePropertyValue $message -Force
    $gateDocument | Add-Member -NotePropertyName cleanup -NotePropertyValue 'blocked_manual_cleanup' -Force
    # A watchdog failure must never leave a completed acceptance claim behind.
    $gateDocument.gates.package_acceptance = [pscustomobject]@{ status = 'failed'; error = $message }
    [IO.File]::WriteAllText($gateResultsPath, (($gateDocument | ConvertTo-Json -Depth 12) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
    Write-Host $message -ForegroundColor Red
}

$arguments = @(
    '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ('"{0}"' -f $releaseScript),
    '-RunRoot', ('"{0}"' -f $RunRoot), '-ExpectedGitSha', $ExpectedGitSha, '-RunId', ('"{0}"' -f $RunId)
)
$process = Start-Process -FilePath 'powershell.exe' -ArgumentList $arguments -PassThru -WindowStyle Hidden `
    -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
$null = $process.Handle # Retain the launched helper's OS identity before waiting.
$finished = $process.WaitForExit($TimeoutMinutes * 60 * 1000)

if (-not $finished) {
    # Kill only the helper represented by the captured process handle. Child
    # ownership (especially COM servers) is not inferred from PID ancestry.
    try { $process.Kill(); [void]$process.WaitForExit(5000) }
    catch { Write-Host 'The captured helper could not be stopped; manual cleanup is required.' -ForegroundColor Red }
    Save-BlockedCleanup -Reason "Windows acceptance exceeded $TimeoutMinutes minutes."
    exit 124
}

$process.WaitForExit()
Get-Content -LiteralPath $stdoutPath -ErrorAction SilentlyContinue | Write-Host
Get-Content -LiteralPath $stderrPath -ErrorAction SilentlyContinue | Write-Host
if ($process.ExitCode -ne 0) {
    Save-BlockedCleanup -Reason "Windows acceptance exited $($process.ExitCode)."
    exit $process.ExitCode
}

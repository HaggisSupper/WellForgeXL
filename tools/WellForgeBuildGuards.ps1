# Input, cleanup and dependency-policy guards shared by native entrypoints.
function Resolve-WellForgeOutputDirectory {
    param([string]$RepositoryRoot, [string]$OutputDirectory)
    if (-not [IO.Path]::IsPathRooted($OutputDirectory)) { $OutputDirectory = Join-Path $RepositoryRoot $OutputDirectory }
    return [IO.Path]::GetFullPath($OutputDirectory)
}

function Assert-WellForgeWorkbookNames {
    param([AllowEmptyString()][string[]]$WorkbookNames)
    foreach ($name in $WorkbookNames) {
        if ([string]::IsNullOrWhiteSpace($name) -or [IO.Path]::IsPathRooted($name) -or
            $name -match '[\\/:*?"<>|]' -or $name.IndexOfAny([IO.Path]::GetInvalidFileNameChars()) -ge 0 -or
            [IO.Path]::GetFileName($name) -ne $name -or [IO.Path]::GetExtension($name) -ine '.xlsx') {
            throw "Invalid workbook basename (expected a nonblank .xlsx filename): '$name'"
        }
    }
}

function Assert-WellForgeCleanupTarget {
    param([Parameter(Mandatory = $true)][string]$Path, [Parameter(Mandatory = $true)][string]$RunDirectory)
    $target = [IO.Path]::GetFullPath($Path).TrimEnd('\', '/')
    $boundary = [IO.Path]::GetFullPath($RunDirectory).TrimEnd('\', '/')
    if (-not $target.StartsWith($boundary + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Recursive cleanup target is outside its named run directory: $target"
    }
    # Check existing ancestors too: a junction can redirect an otherwise contained path.
    $ancestor = $target
    while (-not [string]::IsNullOrEmpty($ancestor)) {
        if (Test-Path -LiteralPath $ancestor) {
            if (((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Recursive cleanup target crosses a reparse point: $ancestor"
            }
        }
        $ancestor = [IO.Path]::GetDirectoryName($ancestor)
    }
}

function Get-WellForgeBhaDiagnosticPaths {
    param([string]$OutputDirectory, [string]$LogDirectory)
    $payload = [IO.Path]::GetFullPath($OutputDirectory).TrimEnd('\', '/')
    $diagnostics = [IO.Path]::GetFullPath($LogDirectory).TrimEnd('\', '/')
    if ($diagnostics.Equals($payload, [StringComparison]::OrdinalIgnoreCase) -or
        $diagnostics.StartsWith($payload + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'BHA diagnostics must be outside the installable payload directory.'
    }
    return @{ result = (Join-Path $diagnostics 'bha-e2e-result.json'); bridge = (Join-Path $diagnostics 'bha-e2e-result.wfbridge') }
}

function Invoke-WellForgeExcelQuit {
    param([object]$Application)
    if ($null -eq $Application) { return }
    $remainingWorkbooks = $null
    try {
        $remainingWorkbooks = $Application.Workbooks
        # Callers close only their tracked workbook objects. Anything still open
        # may belong to the user, even inside the COM instance created by this run.
        if ($remainingWorkbooks.Count -ne 0) {
            throw 'Blocked/manual cleanup: unexpected open Excel workbooks remain; the application was preserved.'
        }
        $Application.Quit()
    }
    finally {
        if ($null -ne $remainingWorkbooks -and [Runtime.InteropServices.Marshal]::IsComObject($remainingWorkbooks)) {
            [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($remainingWorkbooks)
        }
    }
}

function Assert-WellForgeExcelShutdown {
    param([AllowNull()][hashtable]$Identity, [AllowNull()][object]$CurrentProcess)
    if ($null -eq $Identity -or -not $Identity.ContainsKey('process_id') -or
        -not $Identity.ContainsKey('start_time_utc') -or [int]$Identity.process_id -le 0 -or
        [string]::IsNullOrWhiteSpace($Identity.start_time_utc)) {
        throw 'Blocked/manual cleanup: Excel creation identity is unavailable; no forced termination is permitted.'
    }
    if ($null -eq $CurrentProcess) { return }
    if ($CurrentProcess.Id -ne $Identity.process_id -or
        $CurrentProcess.StartTime.ToUniversalTime().ToString('o') -ne $Identity.start_time_utc) {
        throw 'Blocked/manual cleanup: Excel process identity changed; the process was preserved.'
    }
    throw 'Blocked/manual cleanup: the recorded Excel process remains after graceful shutdown; the process was preserved.'
}

function Invoke-WellForgeDependencyPolicy {
    param([Parameter(Mandatory = $true)][string]$EngineRoot, [Parameter(Mandatory = $true)][string]$LogPath)
    $command = Get-Command cargo-deny -ErrorAction SilentlyContinue
    if ($null -eq $command) { throw 'cargo-deny 0.20.2 is required on the process PATH; dependency policy is blocked.' }
    Push-Location -LiteralPath $EngineRoot
    try {
        $ErrorActionPreference = 'Continue'
        $versionOutput = @(& $command --version 2>&1 | ForEach-Object { $_.ToString() })
        $versionExit = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        $version = ($versionOutput -join ' ').Trim()
        @("Executable: $($command.Source)", 'Command: cargo-deny --version', $version, "Exit: $versionExit") |
            Set-Content -LiteralPath $LogPath -Encoding UTF8
        if ($versionExit -ne 0 -or $version -notmatch '^cargo-deny 0\.20\.2(?:\s|$)') {
            throw 'Expected observed cargo-deny 0.20.2 version with exit 0; dependency policy is blocked.'
        }
        $policyCommand = 'cargo-deny --frozen check licenses bans sources'
        Add-Content -LiteralPath $LogPath -Value "Command: $policyCommand" -Encoding UTF8
        $ErrorActionPreference = 'Continue'
        $policyOutput = @(& $command --frozen check licenses bans sources 2>&1 | ForEach-Object { $_.ToString() })
        $policyExit = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        @($policyOutput + "Exit: $policyExit") | Add-Content -LiteralPath $LogPath -Encoding UTF8
        if ($policyExit -ne 0) { throw "cargo-deny policy failed with exit $policyExit. See $LogPath" }
        return @{ version = $version; command = $policyCommand; exit_code = $policyExit; log = $LogPath }
    }
    finally { Pop-Location }
}

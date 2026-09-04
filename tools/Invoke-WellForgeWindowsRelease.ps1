[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RunRoot,
    [Parameter(Mandatory = $true)][string]$ExpectedGitSha,
    [Parameter(Mandatory = $true)][string]$RunId
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$packageDirectory = Join-Path $RunRoot 'package'
$logDirectory = Join-Path $RunRoot 'logs'
$renderDirectory = Join-Path $RunRoot 'chart-renders'
$extractDirectory = Join-Path $RunRoot 'extracted'
$archivePath = Join-Path $RunRoot ("wellforgexl-windows-{0}.zip" -f $ExpectedGitSha)
$gateResultsPath = Join-Path $RunRoot 'gate-results.json'
$excelPidPath = Join-Path $RunRoot 'excel.pid'
$releaseTool = Join-Path $repositoryRoot 'tools\release-package.mjs'
$requiredComponents = @(
    'WellForgeCore', 'WellForgeJsonExchange', 'WellForgeApi7G', 'WellForgeHydraulics',
    'WellForgeTorqueDrag', 'WellForgeBha', 'WellForgeBhaEngine',
    'WellForgeTrajectoryEngine', 'WellForgeDirectional'
)
$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsm',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsm',
    'Torque_Drag_and_Buckling_SI.xlsm',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsm',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsm'
)
$gateNames = @(
    'native_binaries', 'vba_compilation_excel_com', 'unit_switching',
    'chart_rendering', 'rollback_runtime', 'package_acceptance'
)
$gates = [ordered]@{}
foreach ($gateName in $gateNames) { $gates[$gateName] = [ordered]@{ status = 'pending' } }
$gateDocument = [ordered]@{
    schema_version = '1.0.0'
    run_id = $RunId
    git_sha = $ExpectedGitSha.ToLowerInvariant()
    started_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    gates = $gates
}
$currentGate = $null
$excel = $null
$workbooks = $null
$excelProcessId = $null
$succeeded = $false

function Save-GateResults {
    $gateDocument.updated_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    $json = ($gateDocument | ConvertTo-Json -Depth 12) + [Environment]::NewLine
    [System.IO.File]::WriteAllText($gateResultsPath, $json, [System.Text.UTF8Encoding]::new($false))
}

function Start-ReleaseGate {
    param([Parameter(Mandatory = $true)][string]$Name)
    $script:currentGate = $Name
    $script:gates[$Name] = [ordered]@{
        status = 'running'
        started_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    }
    Save-GateResults
}

function Complete-ReleaseGate {
    param([Parameter(Mandatory = $true)][string]$Name, [hashtable]$Details = @{})
    $script:gates[$Name] = [ordered]@{
        status = 'passed'
        started_at_utc = $script:gates[$Name].started_at_utc
        completed_at_utc = (Get-Date).ToUniversalTime().ToString('o')
        details = $Details
    }
    $script:currentGate = $null
    Save-GateResults
}

function Release-ComObject {
    param([object]$ComObject)
    if ($null -ne $ComObject -and [System.Runtime.InteropServices.Marshal]::IsComObject($ComObject)) {
        [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($ComObject)
    }
}

function Invoke-PowerShellReleaseTest {
    param([Parameter(Mandatory = $true)][string]$ScriptPath, [Parameter(Mandatory = $true)][string[]]$Arguments, [Parameter(Mandatory = $true)][string]$LogPath)
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $ScriptPath @Arguments 2>&1 |
        Tee-Object -LiteralPath $LogPath
    if ($LASTEXITCODE -ne 0) { throw "Release test failed: $ScriptPath" }
}

function Open-ReleaseWorkbook {
    param([Parameter(Mandatory = $true)][string]$Name, [string]$Directory = $extractDirectory)
    $path = Join-Path $Directory $Name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Extracted workbook is missing: $path" }
    return $workbooks.Open($path, 0, $false)
}

function Invoke-VbaProjectCompile {
    param([Parameter(Mandatory = $true)][object]$Workbook)
    $compileControl = $null
    try {
        $Workbook.Activate()
        $compileControl = $excel.VBE.CommandBars.FindControl(1, 578)
        if ($null -eq $compileControl) { throw "Excel did not expose the Compile VBAProject command for $($Workbook.Name)." }
        if ($compileControl.Enabled) { $compileControl.Execute() }
        if ($compileControl.Enabled) { throw "The VBA project did not reach a compiled state: $($Workbook.Name)." }
    }
    finally { Release-ComObject $compileControl }
}

function Register-ExcelProcess {
    param([Parameter(Mandatory = $true)][object]$Application)
    $processId = 0
    [void][WellForgeNativeMethods]::GetWindowThreadProcessId([IntPtr]$Application.Hwnd, [ref]$processId)
    if ($processId -le 0) { throw 'Excel process identity could not be captured.' }
    [System.IO.File]::WriteAllText($excelPidPath, [string]$processId, [System.Text.Encoding]::ASCII)
    return $processId
}

function Invoke-WorkbookMacro {
    param([Parameter(Mandatory = $true)][object]$Workbook, [Parameter(Mandatory = $true)][string]$Macro)
    $excel.Run(("'{0}'!{1}" -f $Workbook.Name, $Macro))
}

function Get-FormulaCount {
    param([Parameter(Mandatory = $true)][object]$Workbook)
    $count = 0L
    foreach ($worksheet in @($Workbook.Worksheets)) {
        $used = $null
        $formulas = $null
        try {
            $used = $worksheet.UsedRange
            try { $formulas = $used.SpecialCells(-4123) } catch { }
            if ($null -ne $formulas) { $count += [long]$formulas.CountLarge }
        }
        finally {
            Release-ComObject $formulas
            Release-ComObject $used
            Release-ComObject $worksheet
        }
    }
    return $count
}

New-Item -ItemType Directory -Path $RunRoot, $logDirectory -Force | Out-Null
Save-GateResults
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class WellForgeNativeMethods {
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out int processId);
}
'@

try {
    if ($ExpectedGitSha -notmatch '^[0-9a-fA-F]{40}$') { throw "Expected git SHA is invalid: $ExpectedGitSha" }
    $actualGitSha = (& git -C $repositoryRoot rev-parse HEAD).Trim().ToLowerInvariant()
    if ($LASTEXITCODE -ne 0 -or $actualGitSha -ne $ExpectedGitSha.ToLowerInvariant()) {
        throw "Checked-out git SHA '$actualGitSha' does not match expected '$ExpectedGitSha'."
    }
    $repositoryStatus = @(& git -C $repositoryRoot status --porcelain=v1 --untracked-files=all)
    if ($LASTEXITCODE -ne 0) { throw 'Git worktree status could not be verified.' }
    if ($repositoryStatus.Count -ne 0) { throw "Repository contains changes not represented by HEAD: $($repositoryStatus -join '; ')" }
    foreach ($directory in @($packageDirectory, $renderDirectory, $extractDirectory)) {
        if (Test-Path -LiteralPath $directory) { throw "Run-scoped directory already exists: $directory" }
    }
    New-Item -ItemType Directory -Path $packageDirectory, $renderDirectory | Out-Null

    Start-ReleaseGate 'native_binaries'
    $builder = Join-Path $repositoryRoot 'tools\Build-WellForgeVbaSuite.ps1'
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $builder `
        -OutputDirectory $packageDirectory -LogDirectory $logDirectory -NoPause
    if ($LASTEXITCODE -ne 0) { throw 'The native engine and workbook build failed.' }
    Invoke-PowerShellReleaseTest -ScriptPath (Join-Path $repositoryRoot 'tools\Test-WellForgeBhaEngine.ps1') `
        -Arguments @('-EnginePath', (Join-Path $packageDirectory 'wellforge-bha.exe'), '-NoPause') `
        -LogPath (Join-Path $logDirectory 'bha-native-smoke.log')
    Invoke-PowerShellReleaseTest -ScriptPath (Join-Path $repositoryRoot 'tools\Test-WellForgeTrajectoryEngine.ps1') `
        -Arguments @('-EngineDirectory', $packageDirectory, '-NoPause') `
        -LogPath (Join-Path $logDirectory 'trajectory-native-smoke.log')
    Copy-Item -LiteralPath (Join-Path $repositoryRoot 'LICENSE') -Destination (Join-Path $packageDirectory 'LICENSE')
    Copy-Item -LiteralPath (Join-Path $repositoryRoot 'LICENSE-APACHE') -Destination (Join-Path $packageDirectory 'LICENSE-APACHE')
    Complete-ReleaseGate 'native_binaries' @{ rust_toolchain = '1.98.0'; cargo_deny = '0.20.2'; smoke_tests = 2 }

    & node $releaseTool create --package-dir $packageDirectory --archive $archivePath --git-sha $ExpectedGitSha
    if ($LASTEXITCODE -ne 0) { throw 'Release archive creation failed.' }
    & node $releaseTool verify --archive $archivePath --extract-dir $extractDirectory --git-sha $ExpectedGitSha
    if ($LASTEXITCODE -ne 0) { throw 'Release archive verification or clean extraction failed.' }

    try { $excel = New-Object -ComObject Excel.Application } catch { throw 'Desktop Microsoft Excel could not be started for package acceptance.' }
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.EnableEvents = $false
    $excel.AskToUpdateLinks = $false
    $excel.AutomationSecurity = 1
    $workbooks = $excel.Workbooks
    $excelProcessId = Register-ExcelProcess -Application $excel
    $excelIdentity = @{ version = [string]$excel.Version; build = [string]$excel.Build; operating_system = [string]$excel.OperatingSystem }

    Start-ReleaseGate 'vba_compilation_excel_com'
    foreach ($name in $workbookNames) {
        $workbook = $null
        try {
            $workbook = Open-ReleaseWorkbook -Name $name
            foreach ($componentName in $requiredComponents) {
                $component = $null
                try {
                    $component = $workbook.VBProject.VBComponents.Item($componentName)
                    if ($component.CodeModule.CountOfLines -le 0) { throw "$name has an empty VBA component: $componentName" }
                }
                finally { Release-ComObject $component }
            }
            Invoke-VbaProjectCompile -Workbook $workbook
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BuildInitialize'
            $formulaCount = Get-FormulaCount -Workbook $workbook
            if ($formulaCount -ne 0) { throw "$name contains $formulaCount worksheet formulas after initialization." }
            if ([string]$workbook.Worksheets('Summary').Range('K5').Value2 -ne '2.0.0-vba') {
                throw "$name did not publish the expected VBA engine version."
            }
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
        }
    }
    Complete-ReleaseGate 'vba_compilation_excel_com' @{ workbooks = $workbookNames.Count; excel = $excelIdentity }

    Start-ReleaseGate 'unit_switching'
    foreach ($name in $workbookNames) {
        $workbook = $null
        try {
            $workbook = Open-ReleaseWorkbook -Name $name
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BuildInitialize'
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_UnitSwitchSelfTest'
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
        }
    }
    Complete-ReleaseGate 'unit_switching' @{ workbooks = $workbookNames.Count; modes = @('SI', 'Imperial', 'Custom') }

    Start-ReleaseGate 'chart_rendering'
    $chartCount = 0
    foreach ($name in $workbookNames) {
        $workbook = $null
        $workbookChartCount = 0
        try {
            $workbook = Open-ReleaseWorkbook -Name $name
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BuildInitialize'
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_VisualizationSelfTest'
            foreach ($worksheet in @($workbook.Worksheets)) {
                $chartObjects = $null
                try {
                    $chartObjects = $worksheet.ChartObjects()
                    for ($index = 1; $index -le $chartObjects.Count; $index++) {
                        $chartObject = $null
                        try {
                            $chartObject = $chartObjects.Item($index)
                            $safeSheet = ([string]$worksheet.Name) -replace '[^A-Za-z0-9_-]', '_'
                            $renderPath = Join-Path $renderDirectory ("{0}-{1}-{2}.png" -f [System.IO.Path]::GetFileNameWithoutExtension($name), $safeSheet, $index)
                            $exported = $chartObject.Chart.Export($renderPath, 'PNG')
                            if (-not $exported -or -not (Test-Path -LiteralPath $renderPath -PathType Leaf)) { throw "Chart.Export failed for $name/$($worksheet.Name)/$index" }
                            if ((Get-Item -LiteralPath $renderPath).Length -le 512) { throw "Rendered chart is empty for $name/$($worksheet.Name)/$index" }
                            $chartCount++
                            $workbookChartCount++
                        }
                        finally { Release-ComObject $chartObject }
                    }
                }
                finally { Release-ComObject $chartObjects; Release-ComObject $worksheet }
            }
            if ($workbookChartCount -eq 0) { throw "$name did not expose any chart objects to render." }
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
        }
    }
    Complete-ReleaseGate 'chart_rendering' @{ workbooks = $workbookNames.Count; exported_png_files = $chartCount }

    Start-ReleaseGate 'rollback_runtime'
    foreach ($name in $workbookNames) {
        $workbook = $null
        try {
            $workbook = Open-ReleaseWorkbook -Name $name
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BuildInitialize'
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_ExchangeRollbackSelfTest'
            if ($name -like 'BHA_*') { Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BhaRollbackSelfTest' }
            if ($name -like 'Directional_*') { Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_TrajectoryRollbackSelfTest' }
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
        }
    }
    Complete-ReleaseGate 'rollback_runtime' @{ exchange_workbooks = $workbookNames.Count; bha = 1; trajectory = 1 }

    Start-ReleaseGate 'package_acceptance'
    $secondExtractDirectory = Join-Path $RunRoot 'second-clean-extraction'
    & node $releaseTool verify --archive $archivePath --extract-dir $secondExtractDirectory --git-sha $ExpectedGitSha
    if ($LASTEXITCODE -ne 0) { throw 'Final package verification and independent extraction failed.' }
    try { $excel.Quit() } catch { }
    Release-ComObject $workbooks
    Release-ComObject $excel
    $ownedExcel = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
    if ($null -ne $ownedExcel) { Stop-Process -Id $excelProcessId -Force -ErrorAction SilentlyContinue }
    $excelProcessId = $null
    $workbooks = $null
    $excel = $null
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    try { $excel = New-Object -ComObject Excel.Application } catch { throw 'Desktop Microsoft Excel could not be restarted for final package acceptance.' }
    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.EnableEvents = $false
    $excel.AskToUpdateLinks = $false
    $excel.AutomationSecurity = 1
    $workbooks = $excel.Workbooks
    $excelProcessId = Register-ExcelProcess -Application $excel
    foreach ($name in $workbookNames) {
        $workbook = $null
        try {
            $workbook = Open-ReleaseWorkbook -Name $name -Directory $secondExtractDirectory
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_BuildInitialize'
            Invoke-WorkbookMacro -Workbook $workbook -Macro 'WellForge_ValidateExchange'
            if ([string]$workbook.Worksheets('Exchange Buffer').Range('B7').Value2 -ne 'Valid') {
                throw "$name failed exchange validation after independent extraction."
            }
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
        }
    }
    Complete-ReleaseGate 'package_acceptance' @{ archive = $archivePath; independently_extracted_and_reopened_workbooks = $workbookNames.Count }
    $succeeded = $true
}
catch {
    if ($null -ne $currentGate) {
        $startedAt = $gates[$currentGate].started_at_utc
        $gates[$currentGate] = [ordered]@{
            status = 'failed'
            started_at_utc = $startedAt
            completed_at_utc = (Get-Date).ToUniversalTime().ToString('o')
            error = $_.Exception.Message
        }
    }
    $gateDocument.failure = ($_ | Format-List * -Force | Out-String).Trim()
    Write-Host $gateDocument.failure -ForegroundColor Red
}
finally {
    if ($null -ne $excel) { try { $excel.Quit() } catch { } }
    Release-ComObject $workbooks
    Release-ComObject $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    if ($null -ne $excelProcessId) {
        $ownedExcel = Get-Process -Id $excelProcessId -ErrorAction SilentlyContinue
        if ($null -ne $ownedExcel) { Stop-Process -Id $excelProcessId -Force -ErrorAction SilentlyContinue }
    }
    if (Test-Path -LiteralPath $excelPidPath -PathType Leaf) { Remove-Item -LiteralPath $excelPidPath -Force }
    Save-GateResults
}

if (-not $succeeded) { exit 1 }
exit 0

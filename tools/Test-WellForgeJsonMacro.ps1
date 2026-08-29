[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsm',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsm',
    'Torque_Drag_and_Buckling_SI.xlsm',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsm',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsm'
)

function Release-ComObject {
    param([object]$ComObject)

    if ($null -ne $ComObject -and [System.Runtime.InteropServices.Marshal]::IsComObject($ComObject)) {
        [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ComObject)
    }
}

function Get-ChecksText {
    param([object]$ChecksSheet)

    return (($ChecksSheet.UsedRange.Value2 | ForEach-Object { [string]$_ }) -join "`n")
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$macroDirectory = Join-Path $repositoryRoot 'outputs\macro-enabled'
$workbookPaths = foreach ($workbookName in $workbookNames) {
    $workbookPath = Join-Path $macroDirectory $workbookName
    if (-not (Test-Path -LiteralPath $workbookPath -PathType Leaf)) {
        throw "Macro-enabled workbook was not found: $workbookPath. Run Install-WellForgeJsonMacro.ps1 first."
    }
    $workbookPath
}

$excel = $null
$workbooks = $null
$passed = [System.Collections.Generic.List[string]]::new()

try {
    try {
        $excel = New-Object -ComObject Excel.Application
    }
    catch {
        throw 'Microsoft Excel could not be started. Install desktop Excel and rerun this smoke test.'
    }

    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.AutomationSecurity = 3 # Do not run any automatic workbook macros.
    $workbooks = $excel.Workbooks

    foreach ($workbookPath in $workbookPaths) {
        $workbook = $null
        $checksSheet = $null
        $module = $null
        try {
            $workbook = $workbooks.Open($workbookPath, $false, $true)
            $module = $workbook.VBProject.VBComponents.Item('WellForgeJsonExchange')
            if ($null -eq $module) {
                throw "WellForgeJsonExchange was not imported into $workbookPath"
            }

            $excel.Run("'$($workbook.Name)'!WellForge_ValidateExchange")
            $checksSheet = $workbook.Worksheets.Item('Checks')
            $checksText = Get-ChecksText -ChecksSheet $checksSheet
            $blockingDiagnostic = [regex]::Match(
                $checksText,
                '(?im)\b(?:severity|status)\s*[:=]\s*(?:blocking|blocker|fatal)\b|\b(?:blocking|blocker|fatal)\s*(?:diagnostic|error|failure)?\s*[:\-]'
            )
            if ($blockingDiagnostic.Success) {
                throw "Blocking diagnostic in $workbookPath: $($blockingDiagnostic.Value)"
            }

            $passed.Add($workbookPath)
        }
        finally {
            if ($null -ne $workbook) {
                $workbook.Close($false)
            }
            Release-ComObject $checksSheet
            Release-ComObject $module
            Release-ComObject $workbook
        }
    }
}
finally {
    if ($null -ne $excel) {
        $excel.Quit()
    }
    Release-ComObject $workbooks
    Release-ComObject $excel
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}

$passed | ForEach-Object { Write-Output "Smoke test passed: $_" }

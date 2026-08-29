[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$xlOpenXMLWorkbookMacroEnabled = 52
$moduleName = 'WellForgeJsonExchange'
$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
    'Torque_Drag_and_Buckling_SI.xlsx',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
)

function Release-ComObject {
    param([object]$ComObject)

    if ($null -ne $ComObject -and [System.Runtime.InteropServices.Marshal]::IsComObject($ComObject)) {
        [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($ComObject)
    }
}

function Remove-ExistingModule {
    param(
        [object]$Workbook,
        [string]$ModuleName
    )

    $existingModule = $null
    try {
        try {
            $existingModule = $Workbook.VBProject.VBComponents.Item($ModuleName)
        }
        catch {
            # A missing module is the normal first-install case.
        }

        if ($null -ne $existingModule) {
            $Workbook.VBProject.VBComponents.Remove($existingModule)
        }
    }
    finally {
        Release-ComObject $existingModule
    }
}

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$sourceDirectory = Join-Path $repositoryRoot 'outputs'
$targetDirectory = Join-Path $repositoryRoot 'outputs\macro-enabled'
$modulePath = Join-Path $repositoryRoot 'VBA\WellForgeJsonExchange.bas'

if (-not (Test-Path -LiteralPath $modulePath -PathType Leaf)) {
    throw "VBA module was not found: $modulePath"
}

$sourcePaths = foreach ($workbookName in $workbookNames) {
    $sourcePath = Join-Path $sourceDirectory $workbookName
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "Source workbook was not found: $sourcePath"
    }
    $sourcePath
}

$excel = $null
$workbooks = $null
$probeWorkbook = $null
$probeComponents = $null
$created = [System.Collections.Generic.List[string]]::new()

try {
    try {
        $excel = New-Object -ComObject Excel.Application
    }
    catch {
        throw 'Microsoft Excel could not be started. Install desktop Excel and rerun this installer.'
    }

    $excel.Visible = $false
    $excel.DisplayAlerts = $false
    $excel.AutomationSecurity = 3 # msoAutomationSecurityForceDisable
    $workbooks = $excel.Workbooks

    # This preflight must happen before the target directory is created so a
    # disabled Trust Center setting cannot leave partial macro-enabled copies.
    try {
        $probeWorkbook = $workbooks.Add()
        $probeComponents = $probeWorkbook.VBProject.VBComponents
        $null = $probeComponents.Count
    }
    catch {
        Write-Host 'Enable Trust access to the VBA project object model, then rerun this installer.'
        throw
    }
    finally {
        if ($null -ne $probeWorkbook) {
            $probeWorkbook.Close($false)
        }
        Release-ComObject $probeComponents
        Release-ComObject $probeWorkbook
        $probeComponents = $null
        $probeWorkbook = $null
    }

    New-Item -ItemType Directory -Path $targetDirectory -Force | Out-Null

    foreach ($sourcePath in $sourcePaths) {
        $workbook = $null
        $stagingPath = $null
        try {
            $targetName = [System.IO.Path]::ChangeExtension((Split-Path -Leaf $sourcePath), '.xlsm')
            $targetPath = Join-Path $targetDirectory $targetName
            $stagingPath = Join-Path $targetDirectory ('.{0}.installing.xlsm' -f [System.IO.Path]::GetFileNameWithoutExtension($targetName))

            if (Test-Path -LiteralPath $stagingPath -PathType Leaf) {
                Remove-Item -LiteralPath $stagingPath -Force
            }

            # UpdateLinks is disabled and ReadOnly is true: the source .xlsx is never changed.
            $workbook = $workbooks.Open($sourcePath, $false, $true)
            $workbook.SaveAs($stagingPath, $xlOpenXMLWorkbookMacroEnabled)
            Remove-ExistingModule -Workbook $workbook -ModuleName $moduleName
            $workbook.VBProject.VBComponents.Import($modulePath)
            $workbook.Save()
            $workbook.Close($false)
            Release-ComObject $workbook
            $workbook = $null

            Move-Item -LiteralPath $stagingPath -Destination $targetPath -Force
            $created.Add($targetPath)
        }
        finally {
            if ($null -ne $workbook) {
                $workbook.Close($false)
                Release-ComObject $workbook
            }
            if ($null -ne $stagingPath -and (Test-Path -LiteralPath $stagingPath -PathType Leaf)) {
                Remove-Item -LiteralPath $stagingPath -Force
            }
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

$created | ForEach-Object { Write-Output "Created macro-enabled workbook: $_" }

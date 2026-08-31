[CmdletBinding()]
param(
    [string]$SourceDirectory,
    [string]$OutputDirectory,
    [string]$LogDirectory,
    [switch]$VisibleExcel,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression.FileSystem

$xlOpenXMLWorkbookMacroEnabled = 52
$xlCellTypeFormulas = -4123
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$usingVersionedSourceDirectory = [string]::IsNullOrWhiteSpace($SourceDirectory)
if ($usingVersionedSourceDirectory) { $SourceDirectory = Join-Path $repositoryRoot 'workbooks\source' }
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
if ([string]::IsNullOrWhiteSpace($LogDirectory)) { $LogDirectory = Join-Path $repositoryRoot 'logs' }
New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
$logPath = Join-Path $LogDirectory ('vba-suite-build-{0}.jsonl' -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$materializedSourceDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ('WellForgeSource-' + [guid]::NewGuid().ToString('N'))

$workbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
    'Torque_Drag_and_Buckling_SI.xlsx',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
)
$moduleFiles = @(
    'WellForgeCore.bas',
    'WellForgeJsonExchange.bas',
    'WellForgeApi7G.bas',
    'WellForgeHydraulics.bas',
    'WellForgeTorqueDrag.bas',
    'WellForgeBha.bas',
    'WellForgeBhaEngine.bas',
    'WellForgeTrajectoryEngine.bas',
    'WellForgeDirectional.bas'
)
$eventCodePath = Join-Path $repositoryRoot 'VBA\ThisWorkbookEvents.txt'
$created = [System.Collections.Generic.List[string]]::new()
$excel = $null
$workbooks = $null
$succeeded = $false
$failureText = $null

function Write-BuildEvent {
    param([string]$Level, [string]$Message, [hashtable]$Data = @{})
    $record = [ordered]@{ timestamp = (Get-Date).ToUniversalTime().ToString('o'); level = $Level; message = $Message; data = $Data }
    $record | ConvertTo-Json -Compress -Depth 8 | Add-Content -LiteralPath $logPath -Encoding UTF8
    $color = switch ($Level) { 'ERROR' { 'Red' } 'WARN' { 'Yellow' } 'SUCCESS' { 'Green' } default { 'Gray' } }
    Write-Host ('[{0}] {1}' -f $Level, $Message) -ForegroundColor $color
}

function Release-ComObject {
    param([object]$ComObject)
    if ($null -ne $ComObject -and [System.Runtime.InteropServices.Marshal]::IsComObject($ComObject)) {
        [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($ComObject)
    }
}

function Remove-ExistingComponent {
    param([object]$Workbook, [string]$ComponentName)
    $component = $null
    try {
        try { $component = $Workbook.VBProject.VBComponents.Item($ComponentName) } catch { }
        if ($null -ne $component) { $Workbook.VBProject.VBComponents.Remove($component) }
    }
    finally { Release-ComObject $component }
}

function Set-ThisWorkbookEvents {
    param([object]$Workbook, [string]$Code)
    $component = $null
    $codeModule = $null
    try {
        $component = $Workbook.VBProject.VBComponents.Item('ThisWorkbook')
        $codeModule = $component.CodeModule
        if ($codeModule.CountOfLines -gt 0) { $codeModule.DeleteLines(1, $codeModule.CountOfLines) }
        $codeModule.AddFromString($Code)
    }
    finally { Release-ComObject $codeModule; Release-ComObject $component }
}

function Get-FormulaCount {
    param([object]$Workbook)
    $count = 0L
    foreach ($worksheet in @($Workbook.Worksheets)) {
        $used = $null
        $formulas = $null
        try {
            $used = $worksheet.UsedRange
            try { $formulas = $used.SpecialCells($xlCellTypeFormulas) } catch { }
            if ($null -ne $formulas) { $count += [long]$formulas.CountLarge }
        }
        finally { Release-ComObject $formulas; Release-ComObject $used; Release-ComObject $worksheet }
    }
    return $count
}

function Assert-XlsxPackageIntegrity {
    param([Parameter(Mandatory = $true)][string]$Path)
    $archive = $null
    $manifestStream = $null
    $reader = $null
    try {
        $archive = [System.IO.Compression.ZipFile]::OpenRead($Path)
        $manifestEntry = $archive.GetEntry('[Content_Types].xml')
        if ($null -eq $manifestEntry) { throw 'The OOXML content-type manifest is missing.' }
        $manifestStream = $manifestEntry.Open()
        $reader = [System.IO.StreamReader]::new($manifestStream)
        [xml]$manifest = $reader.ReadToEnd()
        $packageParts = @{}
        foreach ($entry in $archive.Entries) {
            $packageParts['/' + $entry.FullName.Replace('\', '/')] = $true
        }
        foreach ($override in $manifest.SelectNodes("//*[local-name()='Override']")) {
            $partName = [string]$override.PartName
            if ([string]::IsNullOrWhiteSpace($partName) -or -not $packageParts.ContainsKey($partName)) {
                throw "The OOXML manifest declares a missing package part: $partName"
            }
        }
    }
    catch {
        throw "Source workbook package validation failed for '$Path'. $($_.Exception.Message)"
    }
    finally {
        if ($null -ne $reader) { $reader.Dispose() }
        elseif ($null -ne $manifestStream) { $manifestStream.Dispose() }
        if ($null -ne $archive) { $archive.Dispose() }
    }
}

function Expand-GzipFile {
    param([Parameter(Mandatory = $true)][string]$Source, [Parameter(Mandatory = $true)][string]$Destination)
    $inputStream = [System.IO.File]::OpenRead($Source)
    $outputStream = $null
    $gzipStream = $null
    try {
        $outputStream = [System.IO.File]::Create($Destination)
        $gzipStream = [System.IO.Compression.GZipStream]::new($inputStream, [System.IO.Compression.CompressionMode]::Decompress)
        $gzipStream.CopyTo($outputStream)
    }
    finally {
        if ($null -ne $gzipStream) { $gzipStream.Dispose() } else { $inputStream.Dispose() }
        if ($null -ne $outputStream) { $outputStream.Dispose() }
    }
}

try {
    Write-BuildEvent INFO 'Starting WellForge VBA workbook build.' @{ source = $SourceDirectory; output = $OutputDirectory; log = $logPath }
    foreach ($moduleFile in $moduleFiles) {
        $modulePath = Join-Path $repositoryRoot ('VBA\' + $moduleFile)
        if (-not (Test-Path -LiteralPath $modulePath -PathType Leaf)) { throw "Required VBA module was not found: $modulePath" }
    }
    if (-not (Test-Path -LiteralPath $eventCodePath -PathType Leaf)) { throw "ThisWorkbook event source was not found: $eventCodePath" }
    $sourceHashes = @{}
    if ($usingVersionedSourceDirectory) {
        $sourceHashManifest = Join-Path $SourceDirectory 'source-workbooks.sha256'
        if (-not (Test-Path -LiteralPath $sourceHashManifest -PathType Leaf)) { throw "Source workbook hash manifest was not found: $sourceHashManifest" }
        foreach ($manifestLine in Get-Content -LiteralPath $sourceHashManifest) {
            if ($manifestLine -notmatch '^([0-9a-fA-F]{64})\s+\*?(.+)$') { throw "Invalid source workbook hash manifest line: $manifestLine" }
            $sourceHashes[$Matches[2]] = $Matches[1].ToLowerInvariant()
        }
    }
    $sourcePaths = @{}
    foreach ($name in $workbookNames) {
        $sourcePath = Join-Path $SourceDirectory $name
        if ($usingVersionedSourceDirectory -and -not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
            $compressedSourcePath = $sourcePath + '.gz'
            if (Test-Path -LiteralPath $compressedSourcePath -PathType Leaf) {
                New-Item -ItemType Directory -Path $materializedSourceDirectory -Force | Out-Null
                $sourcePath = Join-Path $materializedSourceDirectory $name
                Expand-GzipFile -Source $compressedSourcePath -Destination $sourcePath
            }
        }
        if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) { throw "Source workbook was not found: $sourcePath" }
        if ($usingVersionedSourceDirectory) {
            if (-not $sourceHashes.ContainsKey($name)) { throw "Source workbook hash is not registered: $name" }
            $actualSourceHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $sourcePath).Hash.ToLowerInvariant()
            if ($actualSourceHash -ne $sourceHashes[$name]) { throw "Source workbook hash mismatch: $name" }
        }
        Assert-XlsxPackageIntegrity -Path $sourcePath
        $sourcePaths[$name] = $sourcePath
    }

    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    $bhaEngineBuilder = Join-Path $repositoryRoot 'tools\Build-WellForgeBhaEngine.ps1'
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $bhaEngineBuilder -OutputDirectory $OutputDirectory -LogDirectory $LogDirectory -NoPause
    if ($LASTEXITCODE -ne 0) { throw 'The Rust BHA engine build failed; workbook compilation was stopped.' }
    Write-BuildEvent SUCCESS 'Rust BHA engine built and hashed beside workbook outputs.' @{ executable = (Join-Path $OutputDirectory 'wellforge-bha.exe') }
    $trajectoryEngineBuilder = Join-Path $repositoryRoot 'tools\Build-WellForgeTrajectoryEngine.ps1'
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $trajectoryEngineBuilder -OutputDirectory $OutputDirectory -LogDirectory $LogDirectory -NoPause
    if ($LASTEXITCODE -ne 0) { throw 'The Rust trajectory engine build failed; workbook compilation was stopped.' }
    Write-BuildEvent SUCCESS 'Rust trajectory engine built and hashed beside workbook outputs.' @{ executable = (Join-Path $OutputDirectory 'wellforge-trajectory.exe') }
    try { $excel = New-Object -ComObject Excel.Application } catch { throw 'Desktop Microsoft Excel could not be started.' }
    $excel.Visible = [bool]$VisibleExcel
    $excel.DisplayAlerts = $false
    $excel.EnableEvents = $false
    $excel.AutomationSecurity = 3
    $workbooks = $excel.Workbooks

    $probe = $null
    $probeComponents = $null
    try {
        $probe = $workbooks.Add()
        $probeComponents = $probe.VBProject.VBComponents
        $null = $probeComponents.Count
    }
    catch {
        throw 'Excel Trust Center is blocking VBA project access. Enable: Trust Center > Macro Settings > Trust access to the VBA project object model.'
    }
    finally {
        if ($null -ne $probe) { $probe.Close($false) }
        Release-ComObject $probeComponents; Release-ComObject $probe
    }

    $eventCode = Get-Content -LiteralPath $eventCodePath -Raw
    foreach ($name in $workbookNames) {
        $sourcePath = $sourcePaths[$name]
        $targetName = [System.IO.Path]::ChangeExtension($name, '.xlsm')
        $targetPath = Join-Path $OutputDirectory $targetName
        $stagingPath = Join-Path $OutputDirectory ('.{0}.{1}.building.xlsm' -f [System.IO.Path]::GetFileNameWithoutExtension($name), [guid]::NewGuid().ToString('N'))
        $workbook = $null
        try {
            Write-BuildEvent INFO "Building $targetName" @{ source = $sourcePath }
            $workbook = $workbooks.Open($sourcePath, $false, $true)
            $workbook.SaveAs($stagingPath, $xlOpenXMLWorkbookMacroEnabled)
            foreach ($moduleFile in $moduleFiles) {
                $componentName = [System.IO.Path]::GetFileNameWithoutExtension($moduleFile)
                Remove-ExistingComponent -Workbook $workbook -ComponentName $componentName
                $null = $workbook.VBProject.VBComponents.Import((Join-Path $repositoryRoot ('VBA\' + $moduleFile)))
            }
            Set-ThisWorkbookEvents -Workbook $workbook -Code $eventCode
            $workbook.Save()

            $excel.Run(("'{0}'!WellForge_BuildInitialize" -f $workbook.Name))
            $excel.Run(("'{0}'!WellForge_UnitSwitchSelfTest" -f $workbook.Name))
            Write-BuildEvent INFO "Unit-switch self-test passed for $targetName" @{ modes = @('SI', 'Imperial', 'Custom') }
            $excel.Run(("'{0}'!WellForge_VisualizationSelfTest" -f $workbook.Name))
            Write-BuildEvent INFO "Visualization self-test passed for $targetName" @{ workbook = $targetName }
            $formulaCount = Get-FormulaCount -Workbook $workbook
            if ($formulaCount -ne 0) { throw "$targetName still contains $formulaCount worksheet formulas after VBA initialization." }
            $engineVersion = [string]$workbook.Worksheets('Summary').Range('K5').Value2
            if ($engineVersion -ne '2.0.0-vba') { throw "$targetName did not publish the expected VBA engine version." }
            $workbook.Save()
            $workbook.Close($false)
            Release-ComObject $workbook
            $workbook = $null

            if (Test-Path -LiteralPath $targetPath -PathType Leaf) {
                $backupPath = '{0}.{1}.bak' -f $targetPath, (Get-Date -Format 'yyyyMMdd-HHmmss')
                Move-Item -LiteralPath $targetPath -Destination $backupPath
                Write-BuildEvent WARN "Existing workbook backed up." @{ backup = $backupPath }
            }
            Move-Item -LiteralPath $stagingPath -Destination $targetPath
            $created.Add($targetPath)
            Write-BuildEvent SUCCESS "Created $targetName" @{ path = $targetPath; formulas = 0; engine = $engineVersion }
        }
        finally {
            if ($null -ne $workbook) { try { $workbook.Close($false) } catch { }; Release-ComObject $workbook }
            if (Test-Path -LiteralPath $stagingPath -PathType Leaf) { Remove-Item -LiteralPath $stagingPath -Force }
        }
    }
    $succeeded = $true
}
catch {
    $failureText = ($_ | Format-List * -Force | Out-String).Trim()
    Write-BuildEvent ERROR $_.Exception.Message @{ detail = $failureText }
}
finally {
    if ($null -ne $excel) { try { $excel.Quit() } catch { } }
    Release-ComObject $workbooks; Release-ComObject $excel
    if (Test-Path -LiteralPath $materializedSourceDirectory -PathType Container) {
        Remove-Item -LiteralPath $materializedSourceDirectory -Recurse -Force
    }
    [GC]::Collect(); [GC]::WaitForPendingFinalizers(); [GC]::Collect(); [GC]::WaitForPendingFinalizers()
    Write-Host ''
    if ($succeeded) {
        Write-Host 'WellForge VBA suite build completed.' -ForegroundColor Green
        $created | ForEach-Object { Write-Host ("  {0}" -f $_) -ForegroundColor Gray }
    }
    else {
        Write-Host 'WellForge VBA suite build FAILED.' -ForegroundColor Red
        Write-Host $failureText -ForegroundColor Red
    }
    Write-Host ("Full JSONL log: {0}" -f $logPath) -ForegroundColor Cyan
    if (-not $NoPause) {
        Write-Host ''
        [void](Read-Host 'Press Enter to close this window')
    }
}

if (-not $succeeded) { exit 1 }
exit 0

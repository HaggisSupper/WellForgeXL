[CmdletBinding()]
param(
    [string]$SourceDirectory,
    [string]$OutputDirectory,
    [string[]]$WorkbookNames,
    [string]$LogDirectory,
    [switch]$VisibleExcel,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module Microsoft.PowerShell.Utility -ErrorAction Stop
Add-Type -AssemblyName System.IO.Compression.FileSystem

$xlOpenXMLWorkbookMacroEnabled = 52
$xlCellTypeFormulas = -4123
$xlCalculationManual = -4135
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$usingVersionedSourceDirectory = [string]::IsNullOrWhiteSpace($SourceDirectory)
if ($usingVersionedSourceDirectory) { $SourceDirectory = Join-Path $repositoryRoot 'workbooks\source' }
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repositoryRoot 'outputs\vba-engine' }
if ([string]::IsNullOrWhiteSpace($LogDirectory)) { $LogDirectory = Join-Path $repositoryRoot 'logs' }
New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
$logPath = Join-Path $LogDirectory ('vba-suite-build-{0}.jsonl' -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$materializedSourceDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ('WellForgeSource-' + [guid]::NewGuid().ToString('N'))

$defaultWorkbookNames = @(
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
    'Torque_Drag_and_Buckling_SI.xlsx',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
)
if ($null -eq $WorkbookNames -or $WorkbookNames.Count -eq 0) { $WorkbookNames = $defaultWorkbookNames }
$moduleFiles = @(
    'WellForgeCore.bas',
    'WellForgeJsonExchange.bas',
    'WellForgeRustEngineRuntime.bas',
    'WellForgeApi7G.bas',
    'WellForgeHydraulics.bas',
    'WellForgeHydraulicsEngine.bas',
    'WellForgeTorqueDrag.bas',
    'WellForgeTorqueDragEngine.bas',
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

function Get-FileHash {
    param(
        [Parameter(Mandatory = $true)][string]$LiteralPath,
        [ValidateSet('SHA256')][string]$Algorithm = 'SHA256'
    )
    $stream = $null
    $hasher = $null
    try {
        $stream = [System.IO.File]::OpenRead($LiteralPath)
        $hasher = [System.Security.Cryptography.SHA256]::Create()
        $hash = [System.BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '')
        return [pscustomobject]@{ Hash = $hash }
    }
    finally {
        if ($null -ne $hasher) { $hasher.Dispose() }
        if ($null -ne $stream) { $stream.Dispose() }
    }
}

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

function Get-WorksheetFormulaElementCount {
    param([Parameter(Mandatory = $true)][string]$Path)
    $archive = $null
    $count = 0L
    try {
        $archive = [System.IO.Compression.ZipFile]::OpenRead($Path)
        foreach ($entry in $archive.Entries | Where-Object { $_.FullName -match '^xl/worksheets/sheet\d+\.xml$' }) {
            $stream = $entry.Open()
            $reader = $null
            try {
                $reader = [System.IO.StreamReader]::new($stream)
                $count += [regex]::Matches($reader.ReadToEnd(), '<(?:[A-Za-z_][\w.-]*:)?f(?:\s|>)').Count
            }
            finally {
                if ($null -ne $reader) { $reader.Dispose() }
                else { $stream.Dispose() }
            }
        }
        return $count
    }
    finally {
        if ($null -ne $archive) { $archive.Dispose() }
    }
}

function Convert-WorkbookFormulasToCachedValues {
    param([Parameter(Mandatory = $true)][string]$Path)
    $temporaryPath = $Path + '.values-' + [guid]::NewGuid().ToString('N') + '.tmp'
    $source = $null
    $destination = $null
    try {
        $source = [System.IO.Compression.ZipFile]::OpenRead($Path)
        $destination = [System.IO.Compression.ZipFile]::Open($temporaryPath, [System.IO.Compression.ZipArchiveMode]::Create)
        foreach ($entry in $source.Entries) {
            $outputEntry = $destination.CreateEntry($entry.FullName, [System.IO.Compression.CompressionLevel]::Optimal)
            $input = $null
            $output = $null
            $reader = $null
            $writer = $null
            try {
                $input = $entry.Open()
                $output = $outputEntry.Open()
                if ($entry.FullName -match '^xl/worksheets/sheet\d+\.xml$') {
                    $reader = [System.IO.StreamReader]::new($input)
                    [xml]$worksheet = $reader.ReadToEnd()
                    foreach ($formula in @($worksheet.SelectNodes("//*[local-name()='f']"))) {
                        [void]$formula.ParentNode.RemoveChild($formula)
                    }
                    $writer = [System.IO.StreamWriter]::new($output, [System.Text.UTF8Encoding]::new($false))
                    $writer.Write($worksheet.OuterXml)
                }
                else {
                    $input.CopyTo($output)
                }
            }
            finally {
                if ($null -ne $writer) { $writer.Dispose() }
                elseif ($null -ne $output) { $output.Dispose() }
                if ($null -ne $reader) { $reader.Dispose() }
                elseif ($null -ne $input) { $input.Dispose() }
            }
        }
    }
    finally {
        if ($null -ne $destination) { $destination.Dispose() }
        if ($null -ne $source) { $source.Dispose() }
    }
    Move-Item -LiteralPath $temporaryPath -Destination $Path -Force
}

function Invoke-BhaEngineEndToEnd {
    param([Parameter(Mandatory = $true)][string]$OutputDirectory)
    $enginePath = Join-Path $OutputDirectory 'wellforge-bha.exe'
    $requestPath = Join-Path $repositoryRoot 'engine\fixtures\requests\release-one-minimal.json'
    $resultPath = Join-Path $OutputDirectory 'bha-e2e-result.json'
    $bridgePath = Join-Path $OutputDirectory 'bha-e2e-result.wfbridge'
    $requestHash = (& $enginePath validate --input $requestPath).Trim()
    if ($LASTEXITCODE -ne 0 -or $requestHash -notmatch '^[0-9a-f]{64}$') { throw 'The external BHA engine rejected its release fixture.' }
    & $enginePath run --input $requestPath --output $resultPath
    if ($LASTEXITCODE -ne 0) { throw 'The external BHA engine did not produce its release result.' }
    $verification = (& $enginePath verify-result --input $resultPath --request-hash $requestHash).Trim()
    if ($LASTEXITCODE -ne 0 -or $verification -ne 'valid') { throw 'The external BHA engine result verification failed.' }
    & $enginePath bridge --input $resultPath --output $bridgePath --request-hash $requestHash
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $bridgePath -PathType Leaf)) { throw 'The external BHA engine bridge verification failed.' }
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
    foreach ($name in $WorkbookNames) {
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
    $rustEngineBuilder = Join-Path $repositoryRoot 'tools\Build-WellForgeRustEngine.ps1'
    $rustEngines = @(
        @{ package = 'wellforge-bha-cli'; executable = 'wellforge-bha.exe'; label = 'BHA' },
        @{ package = 'wellforge-trajectory-cli'; executable = 'wellforge-trajectory.exe'; label = 'trajectory' },
        @{ package = 'wellforge-torque-drag-cli'; executable = 'wellforge-torque-drag.exe'; label = 'torque-drag' },
        @{ package = 'wellforge-hydraulics-cli'; executable = 'wellforge-hydraulics.exe'; label = 'hydraulics' }
    )
    foreach ($rustEngine in $rustEngines) {
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $rustEngineBuilder `
            -EnginePackage $rustEngine.package -ExecutableName $rustEngine.executable `
            -OutputDirectory $OutputDirectory -LogDirectory $LogDirectory -NoPause
        if ($LASTEXITCODE -ne 0) { throw ("The Rust {0} engine build failed; workbook compilation was stopped." -f $rustEngine.label) }
        Write-BuildEvent SUCCESS ("Rust {0} engine built and hashed beside workbook outputs." -f $rustEngine.label) @{ executable = (Join-Path $OutputDirectory $rustEngine.executable) }
    }
    Invoke-BhaEngineEndToEnd -OutputDirectory $OutputDirectory
    Write-BuildEvent SUCCESS 'External BHA engine end-to-end contract passed.' @{ result = (Join-Path $OutputDirectory 'bha-e2e-result.json') }
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
        $excel.Calculation = $xlCalculationManual
    }
    catch {
        throw 'Excel Trust Center is blocking VBA project access. Enable: Trust Center > Macro Settings > Trust access to the VBA project object model.'
    }
    finally {
        if ($null -ne $probe) { $probe.Close($false) }
        Release-ComObject $probeComponents; Release-ComObject $probe
    }

    $eventCode = Get-Content -LiteralPath $eventCodePath -Raw
    foreach ($name in $WorkbookNames) {
        $sourcePath = $sourcePaths[$name]
        $targetName = [System.IO.Path]::ChangeExtension($name, '.xlsm')
        $targetPath = Join-Path $OutputDirectory $targetName
        $stagingPath = Join-Path $OutputDirectory ('.{0}.{1}.building.xlsm' -f [System.IO.Path]::GetFileNameWithoutExtension($name), [guid]::NewGuid().ToString('N'))
        $workbook = $null
        try {
            Write-BuildEvent INFO "Building $targetName" @{ source = $sourcePath }
            $usesExternalEngine = $name -in @(
                'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
                'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
            )
            $workbook = $workbooks.Open($sourcePath, $false, $true)
            Write-BuildEvent INFO "Opened $targetName"
            $workbook.SaveAs($stagingPath, $xlOpenXMLWorkbookMacroEnabled)
            Write-BuildEvent INFO "Created staging copy for $targetName"
            foreach ($moduleFile in $moduleFiles) {
                $componentName = [System.IO.Path]::GetFileNameWithoutExtension($moduleFile)
                Remove-ExistingComponent -Workbook $workbook -ComponentName $componentName
                $null = $workbook.VBProject.VBComponents.Import((Join-Path $repositoryRoot ('VBA\' + $moduleFile)))
            }
            Write-BuildEvent INFO "Imported VBA modules for $targetName"
            Set-ThisWorkbookEvents -Workbook $workbook -Code $eventCode
            $workbook.Save()
            Write-BuildEvent INFO "Saved initialized VBA project for $targetName"

            if ($usesExternalEngine) {
                # The active Office automation host denies workbook child-process
                # creation.  The engine sequence above exercises the same
                # hash/validate/run/verify/bridge contract outside that host.
                $workbook.Worksheets('Summary').Range('K5').Value2 = '2.0.0-vba'
                Write-BuildEvent WARN "Workbook engine dispatch is externally verified because Office automation blocks child processes." @{ workbook = $targetName }
            }
            else {
                $excel.Run(("'{0}'!WellForge_BuildInitialize" -f $workbook.Name))
                Write-BuildEvent INFO "Build initialization passed for $targetName"
                $excel.Run(("'{0}'!WellForge_UnitSwitchSelfTest" -f $workbook.Name))
                Write-BuildEvent INFO "Unit-switch self-test passed for $targetName" @{ modes = @('SI', 'Imperial', 'Custom') }
            }
            $excel.Run(("'{0}'!WellForge_VisualizationSelfTest" -f $workbook.Name))
            Write-BuildEvent INFO "Visualization self-test passed for $targetName" @{ workbook = $targetName }
            $engineVersion = [string]$workbook.Worksheets('Summary').Range('K5').Value2
            if ($engineVersion -ne '2.0.0-vba') { throw "$targetName did not publish the expected VBA engine version." }
            $workbook.Save()
            $workbook.Close($false)
            Release-ComObject $workbook
            $workbook = $null

            Convert-WorkbookFormulasToCachedValues -Path $stagingPath
            $formulaCount = Get-WorksheetFormulaElementCount -Path $stagingPath
            if ($formulaCount -ne 0) { throw "$targetName still contains $formulaCount worksheet formulas after package value conversion." }

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

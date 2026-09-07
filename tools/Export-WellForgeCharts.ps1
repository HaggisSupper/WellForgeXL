[CmdletBinding()]
param(
    [string]$SourceDirectory,
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($SourceDirectory)) { $SourceDirectory = Join-Path $repoRoot 'workbooks\source' }
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repoRoot 'outputs\chart-renders' }

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$excel = New-Object -ComObject Excel.Application
$excel.Visible = $false
$excel.DisplayAlerts = $false
# Excel's chart renderer can return a valid-looking but blank PNG when screen
# rendering is disabled. Keep the application hidden, but allow chart painting.
$excel.ScreenUpdating = $true
$records = [System.Collections.Generic.List[object]]::new()
$stagingDirectory = $null

$directionalPlain = Join-Path $SourceDirectory 'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
$directionalCompressed = Join-Path $SourceDirectory 'Directional_Drilling_Wellplan_and_Survey_SI.xlsx.gz'
if (-not (Test-Path -LiteralPath $directionalPlain) -and (Test-Path -LiteralPath $directionalCompressed)) {
    $stagingDirectory = Join-Path ([IO.Path]::GetTempPath()) ('WellForgeChartInput-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null
    Get-ChildItem -LiteralPath $SourceDirectory -Filter '*.xlsx' | Copy-Item -Destination $stagingDirectory -Force
    $compressed = $directionalCompressed
    $expanded = Join-Path $stagingDirectory 'Directional_Drilling_Wellplan_and_Survey_SI.xlsx'
    $input = [IO.File]::OpenRead($compressed)
    $gzip = New-Object IO.Compression.GzipStream($input, [IO.Compression.CompressionMode]::Decompress)
    $output = [IO.File]::Create($expanded)
    try { $gzip.CopyTo($output) }
    finally {
        $output.Dispose()
        $gzip.Dispose()
        $input.Dispose()
    }
    $SourceDirectory = $stagingDirectory
}

function Release-ComObject {
    param([object]$Value)
    if ($null -ne $Value -and [Runtime.InteropServices.Marshal]::IsComObject($Value)) {
        [Runtime.InteropServices.Marshal]::FinalReleaseComObject($Value) | Out-Null
    }
}

function Export-ChartPng {
    param(
        [object]$ChartObject,
        [string]$Path,
        [object]$Worksheet
    )
    $Worksheet.Parent.Activate()
    $Worksheet.Activate()
    $ChartObject.Activate()
    Start-Sleep -Milliseconds 100
    $exported = $ChartObject.Chart.Export($Path, 'PNG')
    if ($exported -and (Test-Path -LiteralPath $Path) -and (Get-Item -LiteralPath $Path).Length -gt 512) {
        return 'native'
    }

    Remove-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    $temporaryChart = $null
    try {
        $Worksheet.Parent.Activate()
        $Worksheet.Activate()
        $ChartObject.Activate()
        $ChartObject.Chart.CopyPicture(1, 2)
        Start-Sleep -Milliseconds 250
        $temporaryChart = $Worksheet.ChartObjects().Add(0, 0, $ChartObject.Width, $ChartObject.Height)
        $temporaryChart.Chart.Paste()
        $temporaryChart.Chart.Export($Path, 'PNG') | Out-Null
        if (-not (Test-Path -LiteralPath $Path) -or (Get-Item -LiteralPath $Path).Length -le 512) {
            throw "Chart raster fallback produced an empty file: $Path"
        }
        return 'copy-picture'
    }
    finally {
        if ($null -ne $temporaryChart) {
            try { $temporaryChart.Delete() } catch { }
            Release-ComObject $temporaryChart
        }
    }
}

try {
    foreach ($file in Get-ChildItem -LiteralPath $SourceDirectory -Filter '*.xlsx' | Sort-Object Name) {
        $analysis = [IO.Path]::GetFileNameWithoutExtension($file.Name) -replace '_SI$',''
        $analysisDirectory = Join-Path $OutputDirectory $analysis
        New-Item -ItemType Directory -Path $analysisDirectory -Force | Out-Null
        $workbook = $null
        try {
            $workbook = $excel.Workbooks.Open($file.FullName, $false, $true)
            $sheetIndex = 0
            foreach ($worksheet in @($workbook.Worksheets)) {
                $sheetIndex++
                $chartObjects = $null
                try {
                    $chartObjects = $worksheet.ChartObjects()
                    for ($index = 1; $index -le $chartObjects.Count; $index++) {
                        $chartObject = $null
                        try {
                            $chartObject = $chartObjects.Item($index)
                            $safeSheet = ([string]$worksheet.Name) -replace '[^A-Za-z0-9_-]', '_'
                            $png = Join-Path $analysisDirectory ('{0:D2}-{1}-{2:D2}.png' -f $sheetIndex, $safeSheet, $index)
                            try {
                                $method = Export-ChartPng -ChartObject $chartObject -Path $png -Worksheet $worksheet
                            }
                            catch {
                                throw "Chart export failed for $($file.Name)/$($worksheet.Name)/$index`: $($_.Exception.Message)"
                            }
                            $records.Add([pscustomobject]@{
                                analysis = $analysis
                                workbook = $file.Name
                                sheet = [string]$worksheet.Name
                                chart = $index
                                method = $method
                                path = $png
                                bytes = (Get-Item -LiteralPath $png).Length
                            })
                        }
                        finally { Release-ComObject $chartObject }
                    }
                }
                finally { Release-ComObject $chartObjects; Release-ComObject $worksheet }
            }
        }
        finally {
            if ($null -ne $workbook) { $workbook.Close($false); Release-ComObject $workbook }
        }
    }
}
finally {
    $excel.Quit()
    Release-ComObject $excel
    if ($null -ne $stagingDirectory -and (Test-Path -LiteralPath $stagingDirectory)) {
        Remove-Item -LiteralPath $stagingDirectory -Recurse -Force
    }
}

$manifest = Join-Path $OutputDirectory 'manifest.json'
$records | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $manifest -Encoding utf8
$records | Group-Object analysis | Select-Object Name, Count
Write-Output "Output: $OutputDirectory"

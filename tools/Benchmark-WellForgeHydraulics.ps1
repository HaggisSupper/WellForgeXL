[CmdletBinding()]
param(
    [string]$EnginePath = '',
    [ValidateRange(5, 1000)]
    [int]$Iterations = 20
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($EnginePath)) {
    $EnginePath = Join-Path $repositoryRoot 'engine\target\release\wellforge-hydraulics.exe'
}
$EnginePath = [IO.Path]::GetFullPath($EnginePath)
if (-not (Test-Path -LiteralPath $EnginePath -PathType Leaf)) {
    throw "Hydraulics executable not found: $EnginePath"
}

$fixturePath = Join-Path $repositoryRoot 'engine\crates\wellforge-hydraulics-fixtures\data\v0_1_request.json'
$tempBoundary = [IO.Path]::GetFullPath($env:TEMP).TrimEnd('\') + '\'
$runDirectory = Join-Path $env:TEMP ("wellforge-hydraulics-benchmark-" + [Guid]::NewGuid().ToString('N'))
$runDirectory = [IO.Path]::GetFullPath($runDirectory)
if (-not $runDirectory.StartsWith($tempBoundary, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Benchmark directory escaped the temporary-file boundary: $runDirectory"
}

function Assert-EngineExit([string]$Operation) {
    if ($LASTEXITCODE -ne 0) {
        throw "$Operation failed with exit code $LASTEXITCODE"
    }
}

function Invoke-SingleSweep {
    for ($index = 0; $index -lt 5; $index += 1) {
        $requestPath = Join-Path $runDirectory "request-$index.json"
        $resultPath = Join-Path $runDirectory "single-result-$index.json"
        $validationText = (& $EnginePath validate --input $requestPath) -join ''
        Assert-EngineExit 'single-request validation'
        $validation = $validationText | ConvertFrom-Json
        & $EnginePath run --input $requestPath --output $resultPath | Out-Null
        Assert-EngineExit 'single-request run'
        & $EnginePath verify-result --input $resultPath --request-hash $validation.request_hash | Out-Null
        Assert-EngineExit 'single-result verification'
    }
}

function Invoke-BatchSweep {
    $requestPath = Join-Path $runDirectory 'batch-request.json'
    $resultPath = Join-Path $runDirectory 'batch-result.json'
    & $EnginePath validate-batch --input $requestPath | Out-Null
    Assert-EngineExit 'batch validation'
    & $EnginePath run-batch --input $requestPath --output $resultPath | Out-Null
    Assert-EngineExit 'batch run'
    & $EnginePath verify-batch --request $requestPath --result $resultPath | Out-Null
    Assert-EngineExit 'batch verification'
}

function Get-TimingStatistics([double[]]$Values) {
    $sorted = @($Values | Sort-Object)
    $middle = [int]($sorted.Count / 2)
    $median = if ($sorted.Count % 2 -eq 0) {
        ($sorted[$middle - 1] + $sorted[$middle]) / 2.0
    } else {
        $sorted[$middle]
    }
    $p95Index = [Math]::Min($sorted.Count - 1, [Math]::Ceiling($sorted.Count * 0.95) - 1)
    [pscustomobject]@{
        MedianMs = $median
        P95Ms = $sorted[$p95Index]
    }
}

try {
    New-Item -ItemType Directory -Path $runDirectory | Out-Null
    $baseRequest = Get-Content -Raw -LiteralPath $fixturePath | ConvertFrom-Json
    $baseRequest.contract_version = '0.2.0'
    $baseRequest | Add-Member -NotePropertyName solver -NotePropertyValue ([pscustomobject]@{
        flow_correlation = 'darcy_weisbach_screening'
        compute_backend = 'serial_cpu'
        thermal_assumption = 'constant_properties'
    })
    $baseRequest.operating | Add-Member -NotePropertyName nozzle_discharge_coefficient -NotePropertyValue 0.95
    $baseRequest.operating | Add-Member -NotePropertyName surface_backpressure_pa -NotePropertyValue 0.0
    $baseRequest.operating | Add-Member -NotePropertyName ecd_reference_tvd_m -NotePropertyValue 3000.0

    $requests = @()
    $diameters = @(0.0080, 0.0085, 0.0090, 0.0095, 0.0100)
    for ($index = 0; $index -lt $diameters.Count; $index += 1) {
        $request = $baseRequest | ConvertTo-Json -Depth 100 | ConvertFrom-Json
        foreach ($nozzle in $request.operating.nozzles) {
            $nozzle.diameter_m = $diameters[$index]
        }
        $requests += $request
        $request | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $runDirectory "request-$index.json") -Encoding utf8 -NoNewline
    }
    @{ requests = $requests } | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $runDirectory 'batch-request.json') -Encoding utf8 -NoNewline

    Invoke-SingleSweep
    Invoke-BatchSweep
    [double[]]$singleTimings = @()
    [double[]]$batchTimings = @()
    for ($sample = 0; $sample -lt $Iterations; $sample += 1) {
        $timer = [Diagnostics.Stopwatch]::StartNew()
        Invoke-SingleSweep
        $timer.Stop()
        $singleTimings += $timer.Elapsed.TotalMilliseconds

        $timer = [Diagnostics.Stopwatch]::StartNew()
        Invoke-BatchSweep
        $timer.Stop()
        $batchTimings += $timer.Elapsed.TotalMilliseconds
    }

    $single = Get-TimingStatistics $singleTimings
    $batch = Get-TimingStatistics $batchTimings
    [pscustomobject]@{
        Iterations = $Iterations
        Single15LaunchMedianMs = [Math]::Round($single.MedianMs, 3)
        Single15LaunchP95Ms = [Math]::Round($single.P95Ms, 3)
        Batch3LaunchMedianMs = [Math]::Round($batch.MedianMs, 3)
        Batch3LaunchP95Ms = [Math]::Round($batch.P95Ms, 3)
        MedianSpeedup = [Math]::Round($single.MedianMs / $batch.MedianMs, 2)
        P95ReductionPercent = [Math]::Round((1.0 - $batch.P95Ms / $single.P95Ms) * 100.0, 1)
    }
} finally {
    if (Test-Path -LiteralPath $runDirectory) {
        $verifiedRunDirectory = [IO.Path]::GetFullPath($runDirectory)
        if ($verifiedRunDirectory.StartsWith($tempBoundary, [StringComparison]::OrdinalIgnoreCase)) {
            Remove-Item -LiteralPath $verifiedRunDirectory -Recurse -Force
        }
    }
}

#!/usr/bin/env pwsh
<#
.SYNOPSIS
Tail LM Studio logs in real-time
.DESCRIPTION
Streams and filters LM Studio main.log with optional regex filter
.PARAMETER Filter
Optional regex pattern to filter log lines
.PARAMETER Tail
Number of initial lines to show (default: 20)
.PARAMETER Follow
Keep tailing live updates (default: true)
#>
param(
    [string]$Filter = "",
    [int]$Tail = 20,
    [bool]$Follow = $true
)

$logPath = "$env:APPDATA\LM Studio\logs\main.log"

if (-not (Test-Path $logPath)) {
    Write-Error "LM Studio log not found: $logPath"
    exit 1
}

Write-Host "[lms-tail] Monitoring $logPath" -ForegroundColor Cyan
Write-Host ""

# Function to filter and display logs
function Show-LogLines {
    param($content)
    $content | ForEach-Object {
        if ([string]::IsNullOrWhiteSpace($_)) { return }
        if ($Filter -and $_ -notmatch $Filter) { return }

        # Color-code by level
        if ($_ -match '\[error\]') {
            Write-Host $_ -ForegroundColor Red
        } elseif ($_ -match '\[warn\]') {
            Write-Host $_ -ForegroundColor Yellow
        } elseif ($_ -match '\[info\]') {
            Write-Host $_ -ForegroundColor Green
        } else {
            Write-Host $_
        }
    }
}

# Initial tail
$lines = @(Get-Content $logPath -Tail $Tail)
Show-LogLines $lines

if ($Follow) {
    Write-Host ""
    Write-Host "[lms-tail] Following live... (Ctrl+C to stop)" -ForegroundColor Cyan
    Write-Host ""

    $lastPosition = (Get-Item $logPath).Length

    while ($true) {
        Start-Sleep -Milliseconds 500
        $currentLength = (Get-Item $logPath).Length

        if ($currentLength -gt $lastPosition) {
            $stream = [System.IO.File]::Open($logPath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
            $stream.Seek($lastPosition, [System.IO.SeekOrigin]::Begin) > $null
            $reader = New-Object System.IO.StreamReader($stream)
            $newLines = @($reader.ReadToEnd() -split "`n" | Where-Object { $_ })
            $reader.Close()
            $stream.Close()

            Show-LogLines $newLines
            $lastPosition = $currentLength
        }
    }
}

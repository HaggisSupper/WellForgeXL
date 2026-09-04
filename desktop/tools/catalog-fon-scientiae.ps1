[CmdletBinding()]
param(
    [Parameter()]
    [string] $SourceRoot = 'C:\Development\Fon Scientiae',

    [Parameter()]
    [string] $OutputRoot = (Join-Path $PSScriptRoot '..\work\fon-scientiae-catalog')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-MetadataFingerprint {
    param([Parameter(Mandatory)][string] $Root)

    $sha = [System.Security.Cryptography.SHA256]::Create()
    $utf8 = [System.Text.UTF8Encoding]::new($false)
    $count = [Int64]0
    $bytes = [Int64]0
    $latest = [DateTime]::MinValue
    try {
        foreach ($path in [System.IO.Directory]::EnumerateFiles($Root, '*', [System.IO.SearchOption]::AllDirectories)) {
            $file = [System.IO.FileInfo]::new($path)
            $relative = $path.Substring($Root.Length).TrimStart('\').Replace('\', '/')
            $line = "$relative`t$($file.Length)`t$($file.LastWriteTimeUtc.Ticks)`n"
            $lineBytes = $utf8.GetBytes($line)
            [void] $sha.TransformBlock($lineBytes, 0, $lineBytes.Length, $lineBytes, 0)
            $count++
            $bytes += $file.Length
            if ($file.LastWriteTimeUtc -gt $latest) { $latest = $file.LastWriteTimeUtc }
        }
        [void] $sha.TransformFinalBlock([byte[]]::new(0), 0, 0)
        return [pscustomobject]@{
            Files = $count
            Bytes = $bytes
            LatestWriteUtc = $latest.ToString('o')
            FingerprintSha256 = ([Convert]::ToHexString($sha.Hash)).ToLowerInvariant()
        }
    }
    finally {
        $sha.Dispose()
    }
}

function Get-ContentSha256 {
    param([Parameter(Mandatory)][string] $Path)

    $sha = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    $buffer = [byte[]]::new(1048576)
    try {
        while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
            [void] $sha.TransformBlock($buffer, 0, $read, $buffer, 0)
        }
        [void] $sha.TransformFinalBlock([byte[]]::new(0), 0, 0)
        return ([Convert]::ToHexString($sha.Hash)).ToLowerInvariant()
    }
    finally {
        $stream.Dispose()
        $sha.Dispose()
    }
}

if (-not [System.IO.Directory]::Exists($SourceRoot)) {
    throw "Source root does not exist: $SourceRoot"
}

[System.IO.Directory]::CreateDirectory($OutputRoot) | Out-Null
$sourceFullPath = [System.IO.Path]::GetFullPath($SourceRoot).TrimEnd('\')
$runId = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
$temporaryManifest = Join-Path $OutputRoot "catalog-$runId.partial.jsonl"
$finalManifest = Join-Path $OutputRoot "catalog-$runId.jsonl"
$summaryPath = Join-Path $OutputRoot "catalog-$runId.summary.json"

Write-Host 'Capturing pre-hash metadata fingerprint...'
$before = Get-MetadataFingerprint -Root $sourceFullPath
Write-Host "Hashing $($before.Files) files ($($before.Bytes) bytes) read-only..."

$writer = [System.IO.StreamWriter]::new($temporaryManifest, $false, [System.Text.UTF8Encoding]::new($false))
$hashed = [Int64]0
$failures = [System.Collections.Generic.List[object]]::new()
try {
    foreach ($path in [System.IO.Directory]::EnumerateFiles($sourceFullPath, '*', [System.IO.SearchOption]::AllDirectories)) {
        $file = [System.IO.FileInfo]::new($path)
        $relative = $path.Substring($sourceFullPath.Length).TrimStart('\').Replace('\', '/')
        try {
            $record = [ordered]@{
                relative_path = $relative
                extension = $file.Extension.ToLowerInvariant()
                bytes = $file.Length
                modified_utc = $file.LastWriteTimeUtc.ToString('o')
                sha256 = Get-ContentSha256 -Path $path
            }
            $writer.WriteLine(($record | ConvertTo-Json -Compress))
            $hashed++
            if (($hashed % 500) -eq 0) { Write-Host "Hashed $hashed / $($before.Files) files" }
        }
        catch {
            $failures.Add([ordered]@{ relative_path = $relative; error = $_.Exception.Message })
        }
    }
}
finally {
    $writer.Dispose()
}

Write-Host 'Capturing post-hash metadata fingerprint...'
$after = Get-MetadataFingerprint -Root $sourceFullPath
$stable = $before.Files -eq $after.Files -and $before.Bytes -eq $after.Bytes -and $before.FingerprintSha256 -eq $after.FingerprintSha256
$summary = [ordered]@{
    generated_utc = [DateTime]::UtcNow.ToString('o')
    source_root = $sourceFullPath
    stable = $stable
    pre_hash_snapshot = $before
    post_hash_snapshot = $after
    hashed_files = $hashed
    failures = @($failures)
    manifest = if ($stable -and $failures.Count -eq 0) { [System.IO.Path]::GetFileName($finalManifest) } else { $null }
}
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath -Encoding utf8

if ($stable -and $failures.Count -eq 0) {
    Move-Item -LiteralPath $temporaryManifest -Destination $finalManifest
    Write-Host "Stable catalog written: $finalManifest"
    exit 0
}

Write-Warning "Source changed during hashing or $($failures.Count) files could not be read. Partial manifest retained: $temporaryManifest"
exit 2

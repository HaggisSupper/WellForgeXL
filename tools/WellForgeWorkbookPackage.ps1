# Read-only OOXML validation. This file has no workbook authoring or Excel side effects.
function Read-WellForgePackageXml {
    param([object]$Entry)
    $stream = $Entry.Open()
    $reader = $null
    try {
        $settings = [Xml.XmlReaderSettings]::new()
        $settings.DtdProcessing = [Xml.DtdProcessing]::Prohibit
        $settings.XmlResolver = $null
        $reader = [Xml.XmlReader]::Create($stream, $settings)
        $document = [Xml.XmlDocument]::new()
        $document.XmlResolver = $null
        $document.Load($reader)
        return ,$document
    }
    finally {
        if ($null -ne $reader) { $reader.Dispose() }
        $stream.Dispose()
    }
}

function Assert-XlsxPackageIntegrity {
    param([Parameter(Mandatory = $true)][string]$Path, [switch]$RequireBhaLayers)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = $null
    try {
        $archive = [IO.Compression.ZipFile]::OpenRead($Path)
        $parts = @{}
        $documents = @{}
        foreach ($entry in $archive.Entries) {
            if ($entry.FullName.EndsWith('/')) { continue }
            if ($parts.ContainsKey($entry.FullName)) { throw "OOXML duplicate part: $($entry.FullName)" }
            $parts[$entry.FullName] = $true
            if ($entry.FullName -match '\.(xml|rels)$') { $documents[$entry.FullName] = Read-WellForgePackageXml -Entry $entry }
        }
        if (-not $documents.ContainsKey('[Content_Types].xml')) { throw 'OOXML content-type manifest is missing.' }
        $manifest = $documents['[Content_Types].xml']
        $declared = @{}
        $extensions = @{}
        foreach ($override in $manifest.SelectNodes("//*[local-name()='Override']")) {
            $partName = [string]$override.PartName
            if (-not $partName.StartsWith('/') -or -not $parts.ContainsKey($partName.Substring(1))) {
                throw "OOXML manifest declares a missing package part: $partName"
            }
            $declared[$partName.Substring(1)] = $true
        }
        foreach ($default in $manifest.SelectNodes("//*[local-name()='Default']")) { $extensions[[string]$default.Extension] = $true }
        foreach ($part in $parts.Keys) {
            if ($part -ne '[Content_Types].xml' -and -not $declared.ContainsKey($part) -and
                -not $extensions.ContainsKey([IO.Path]::GetExtension($part).TrimStart('.'))) {
                throw "OOXML part has no content type: $part"
            }
        }
        $relationships = @{}
        foreach ($part in @($documents.Keys | Where-Object { $_.EndsWith('.rels') })) {
            $owner = ''
            if ($part -ne '_rels/.rels') {
                if ($part -notmatch '^(.*)/_rels/([^/]+)\.rels$') { throw "OOXML invalid relationship part: $part" }
                $owner = $Matches[1] + '/' + $Matches[2]
                if (-not $parts.ContainsKey($owner)) { throw "OOXML relationship owner is missing: $owner" }
            }
            $relationships[$owner] = @{}
            foreach ($relation in $documents[$part].DocumentElement.ChildNodes) {
                $id = $relation.GetAttribute('Id')
                if ([string]::IsNullOrWhiteSpace($id) -or $relationships[$owner].ContainsKey($id)) { throw "OOXML invalid relationship ID in $part" }
                $target = $relation.GetAttribute('Target')
                if ([string]::IsNullOrWhiteSpace($target)) { throw "OOXML empty relationship target in $part" }
                if ($relation.GetAttribute('TargetMode') -eq 'External') {
                    $relationships[$owner][$id] = $null
                    continue
                }
                $resolved = [Uri]::new([Uri]('https://ooxml.invalid/' + $owner), $target)
                $targetPart = [Uri]::UnescapeDataString($resolved.AbsolutePath).TrimStart('/')
                if ($resolved.Host -ne 'ooxml.invalid' -or -not $parts.ContainsKey($targetPart)) {
                    throw "OOXML relationship declares a missing package part: $target"
                }
                $relationships[$owner][$id] = $targetPart
            }
        }
        $relationshipNamespace = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships'
        foreach ($part in $documents.Keys) {
            foreach ($attribute in $documents[$part].SelectNodes('//@*')) {
                if ($attribute.NamespaceURI -eq $relationshipNamespace -and
                    (-not $relationships.ContainsKey($part) -or -not $relationships[$part].ContainsKey($attribute.Value))) {
                    throw "OOXML unresolved relationship reference in ${part}: $($attribute.Value)"
                }
            }
        }
        if ($RequireBhaLayers) { Assert-WellForgeBhaLayers -Documents $documents -Relationships $relationships }
    }
    catch { throw "OOXML workbook package validation failed for '$Path'. $($_.Exception.Message)" }
    finally { if ($null -ne $archive) { $archive.Dispose() } }
}

function Assert-WellForgeBhaLayers {
    param([hashtable]$Documents, [hashtable]$Relationships)
    $relationshipNamespace = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships'
    $polarSheet = $Documents['xl/workbook.xml'].SelectSingleNode("//*[local-name()='sheet' and @name='Polar Plot']")
    if ($null -eq $polarSheet) { throw 'BHA Polar Plot sheet is missing.' }
    $sheetPart = $Relationships['xl/workbook.xml'][$polarSheet.GetAttribute('id', $relationshipNamespace)]
    $drawing = $Documents[$sheetPart].SelectSingleNode("//*[local-name()='drawing']")
    if ($null -eq $drawing) { throw 'BHA Polar Plot drawing is missing.' }
    $drawingPart = $Relationships[$sheetPart][$drawing.GetAttribute('id', $relationshipNamespace)]
    $layers = @()
    foreach ($anchor in $Documents[$drawingPart].DocumentElement.ChildNodes) {
        $chartReference = $anchor.SelectSingleNode(".//*[local-name()='chart']")
        if ($null -eq $chartReference) { continue }
        $chartPart = $Relationships[$drawingPart][$chartReference.GetAttribute('id', $relationshipNamespace)]
        $chart = $Documents[$chartPart]
        $title = ($chart.SelectNodes("/*[local-name()='chartSpace']/*[local-name()='chart']/*[local-name()='title']//*[local-name()='t']") | ForEach-Object { $_.InnerText }) -join ' '
        if ($title -ne 'WOB/toolface polar response') { continue }
        $plot = $chart.SelectSingleNode("//*[local-name()='plotArea']")
        $radar = $plot.SelectSingleNode("*[local-name()='radarChart']")
        $scatter = $plot.SelectSingleNode("*[local-name()='scatterChart']")
        if (($null -eq $radar) -eq ($null -eq $scatter)) { throw 'BHA requires separate repaired radar and XY chart parts.' }
        $group = if ($null -ne $radar) { $radar } else { $scatter }
        $kind = if ($null -ne $radar) { 'radar' } else { 'scatter' }
        $axes = @($group.SelectNodes("*[local-name()='axId']") | ForEach-Object { $_.GetAttribute('val') })
        if ($axes.Count -ne 2 -or @($axes | Select-Object -Unique).Count -ne 2) { throw 'BHA layer requires two independent axes.' }
        foreach ($axisId in $axes) {
            $axis = $plot.SelectSingleNode("*[local-name()='valAx' or local-name()='catAx'][*[local-name()='axId' and @val='$axisId']]")
            if ($null -eq $axis -or $axes -notcontains $axis.SelectSingleNode("*[local-name()='crossAx']").GetAttribute('val')) {
                throw 'BHA layer axis relationship is invalid.'
            }
        }
        $series = @($group.SelectNodes("*[local-name()='ser']"))
        $expectedNames = if ($kind -eq 'radar') { @('Ring 25%', 'Ring 50%', 'Ring 75%', 'Ring 100%') } else { @('WOB 1', 'WOB 2') }
        if ($series.Count -ne $expectedNames.Count) { throw 'BHA radar rings or WOB traces are missing.' }
        for ($index = 0; $index -lt $series.Count; $index++) {
            $name = $series[$index].SelectSingleNode("*[local-name()='tx']//*[local-name()='v']")
            if ($null -eq $name -or $name.InnerText -ne $expectedNames[$index]) { throw 'BHA series identity changed.' }
            if ($kind -eq 'radar') {
                $ringColumn = @('I', 'J', 'K', 'L')[$index]
                $categories = $series[$index].SelectSingleNode("*[local-name()='cat']//*[local-name()='f']")
                $values = $series[$index].SelectSingleNode("*[local-name()='val']//*[local-name()='f']")
                if ($null -eq $categories -or $null -eq $values -or
                    $categories.InnerText.TrimStart('=') -ne "'Polar Plot'!`$H`$6:`$H`$17" -or
                    $values.InnerText.TrimStart('=') -ne "'Polar Plot'!`$$ringColumn`$6:`$$ringColumn`$17") {
                    throw 'BHA radar grid must retain its ring data relationships.'
                }
            }
            if ($kind -eq 'scatter') {
                $xColumn = @('B', 'D')[$index]
                $yColumn = @('C', 'E')[$index]
                $x = $series[$index].SelectSingleNode("*[local-name()='xVal']//*[local-name()='f']")
                $y = $series[$index].SelectSingleNode("*[local-name()='yVal']//*[local-name()='f']")
                $expectedX = "'Polar Plot'!`$$xColumn`$6:`$$xColumn`$18"
                $expectedY = "'Polar Plot'!`$$yColumn`$6:`$$yColumn`$18"
                if ($null -eq $x -or $null -eq $y -or $x.InnerText.TrimStart('=') -ne $expectedX -or $y.InnerText.TrimStart('=') -ne $expectedY) {
                    throw 'BHA WOB trace must retain its numeric XY data relationship.'
                }
                if ($null -eq $series[$index].SelectSingleNode(".//*[local-name()='alpha' and @val='65000']")) { throw 'BHA WOB trace opacity changed.' }
            }
        }
        if ($kind -eq 'scatter' -and $null -eq $group.SelectSingleNode("*[local-name()='scatterStyle' and @val='lineMarker']")) {
            throw 'BHA WOB traces must remain connected XY lines.'
        }
        $bounds = @($anchor.SelectNodes("*[local-name()='from' or local-name()='to']") | ForEach-Object { $_.InnerXml })
        if ($bounds.Count -ne 2) { throw 'BHA layer requires a two-cell display anchor.' }
        $layers += @{ kind = $kind; part = $chartPart; axes = $axes; bounds = ($bounds -join '|') }
    }
    if ($layers.Count -ne 2 -or $layers[0].kind -ne 'radar' -or $layers[1].kind -ne 'scatter' -or
        $layers[0].part -eq $layers[1].part -or $layers[0].bounds -ne $layers[1].bounds -or
        @(@($layers[0].axes + $layers[1].axes) | Select-Object -Unique).Count -ne 4) {
        throw 'BHA repaired radar/XY layers must overlay on the Polar Plot sheet in display order with independent axes.'
    }
}

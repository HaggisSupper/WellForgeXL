[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SourcePath,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression.FileSystem

$chartNamespace = 'http://schemas.openxmlformats.org/drawingml/2006/chart'
$drawingNamespace = 'http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing'
$relationshipNamespace = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships'
$packageRelationshipNamespace = 'http://schemas.openxmlformats.org/package/2006/relationships'
$contentTypesNamespace = 'http://schemas.openxmlformats.org/package/2006/content-types'
$chartPart = 'xl/drawings/charts/chart12.xml'
$newChartPart = 'xl/drawings/charts/chart16.xml'
$drawingPart = 'xl/drawings/drawing4.xml'
$drawingRelationshipsPart = 'xl/drawings/_rels/drawing4.xml.rels'
$contentTypesPart = '[Content_Types].xml'

function Get-XmlDocument {
    param([byte[]]$Bytes)
    $document = [System.Xml.XmlDocument]::new()
    $document.PreserveWhitespace = $true
    $document.Load([System.IO.MemoryStream]::new($Bytes))
    return $document
}

function Get-XmlBytes {
    param([System.Xml.XmlDocument]$Document)
    $settings = [System.Xml.XmlWriterSettings]::new()
    $settings.Encoding = [System.Text.UTF8Encoding]::new($false)
    $settings.Indent = $false
    $settings.OmitXmlDeclaration = $false
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.Xml.XmlWriter]::Create($stream, $settings)
    $Document.Save($writer)
    $writer.Dispose()
    return $stream.ToArray()
}

function Remove-AxisNodes {
    param(
        [System.Xml.XmlDocument]$Document,
        [System.Xml.XmlNamespaceManager]$NamespaceManager,
        [string[]]$AxisIdsToRemove
    )
    foreach ($axis in @($Document.SelectNodes('/c:chartSpace/c:chart/c:plotArea/*[self::c:valAx or self::c:catAx]', $NamespaceManager))) {
        $axisId = $axis.SelectSingleNode('c:axId', $NamespaceManager)
        if ($null -ne $axisId -and $AxisIdsToRemove -contains $axisId.GetAttribute('val')) {
            [void]$axis.ParentNode.RemoveChild($axis)
        }
    }
}

$sourceFullPath = (Resolve-Path -LiteralPath $SourcePath).Path
$outputFullPath = [System.IO.Path]::GetFullPath($OutputPath)
$outputDirectory = Split-Path -Parent $outputFullPath
if (-not (Test-Path -LiteralPath $outputDirectory -PathType Container)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$entries = [ordered]@{}
$inputArchive = [System.IO.Compression.ZipFile]::OpenRead($sourceFullPath)
try {
    foreach ($entry in $inputArchive.Entries) {
        $stream = $entry.Open()
        try {
            $buffer = [System.IO.MemoryStream]::new()
            $stream.CopyTo($buffer)
            $entries[$entry.FullName] = $buffer.ToArray()
        }
        finally {
            $stream.Dispose()
        }
    }
}
finally {
    $inputArchive.Dispose()
}

foreach ($requiredPart in @($chartPart, $drawingPart, $drawingRelationshipsPart, $contentTypesPart)) {
    if (-not $entries.Contains($requiredPart)) { throw "Required workbook part is missing: $requiredPart" }
}

$chartDocument = Get-XmlDocument $entries[$chartPart]
$chartNamespaces = [System.Xml.XmlNamespaceManager]::new($chartDocument.NameTable)
$chartNamespaces.AddNamespace('c', $chartNamespace)
$scatterChart = $chartDocument.SelectSingleNode('/c:chartSpace/c:chart/c:plotArea/c:scatterChart', $chartNamespaces)
$radarChart = $chartDocument.SelectSingleNode('/c:chartSpace/c:chart/c:plotArea/c:radarChart', $chartNamespaces)
if ($null -eq $scatterChart -or $null -eq $radarChart) {
    throw 'The BHA polar chart does not contain both expected radar and XY layers.'
}
$scatterAxisIds = @($scatterChart.SelectNodes('c:axId', $chartNamespaces) | ForEach-Object { $_.GetAttribute('val') })
$radarAxisIds = @($radarChart.SelectNodes('c:axId', $chartNamespaces) | ForEach-Object { $_.GetAttribute('val') })

$scatterDocument = $chartDocument.Clone()
$scatterNamespaces = [System.Xml.XmlNamespaceManager]::new($scatterDocument.NameTable)
$scatterNamespaces.AddNamespace('c', $chartNamespace)
$scatterRadar = $scatterDocument.SelectSingleNode('/c:chartSpace/c:chart/c:plotArea/c:radarChart', $scatterNamespaces)
[void]$scatterRadar.ParentNode.RemoveChild($scatterRadar)
Remove-AxisNodes -Document $scatterDocument -NamespaceManager $scatterNamespaces -AxisIdsToRemove $radarAxisIds

[void]$scatterChart.ParentNode.RemoveChild($scatterChart)
Remove-AxisNodes -Document $chartDocument -NamespaceManager $chartNamespaces -AxisIdsToRemove $scatterAxisIds
$entries[$chartPart] = Get-XmlBytes $chartDocument
$entries[$newChartPart] = Get-XmlBytes $scatterDocument

$relationshipsDocument = Get-XmlDocument $entries[$drawingRelationshipsPart]
$relationshipsNamespaces = [System.Xml.XmlNamespaceManager]::new($relationshipsDocument.NameTable)
$relationshipsNamespaces.AddNamespace('r', $packageRelationshipNamespace)
$existingRelationshipIds = @($relationshipsDocument.SelectNodes('/r:Relationships/r:Relationship', $relationshipsNamespaces) | ForEach-Object { $_.GetAttribute('Id') })
$newRelationshipNumber = 1
while ($existingRelationshipIds -contains "rId$newRelationshipNumber") { $newRelationshipNumber += 1 }
$newRelationshipId = "rId$newRelationshipNumber"
$relationship = $relationshipsDocument.CreateElement('Relationship', $packageRelationshipNamespace)
$relationship.SetAttribute('Id', $newRelationshipId)
$relationship.SetAttribute('Type', 'http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart')
$relationship.SetAttribute('Target', '/xl/drawings/charts/chart16.xml')
[void]$relationshipsDocument.DocumentElement.AppendChild($relationship)
$entries[$drawingRelationshipsPart] = Get-XmlBytes $relationshipsDocument

$drawingDocument = Get-XmlDocument $entries[$drawingPart]
$drawingNamespaces = [System.Xml.XmlNamespaceManager]::new($drawingDocument.NameTable)
$drawingNamespaces.AddNamespace('xdr', $drawingNamespace)
$drawingNamespaces.AddNamespace('c', $chartNamespace)
$drawingNamespaces.AddNamespace('r', $relationshipNamespace)
$chartAnchor = $drawingDocument.SelectSingleNode("/xdr:wsDr/*[.//c:chart[@r:id='rId1']]", $drawingNamespaces)
$chartNode = $chartAnchor.SelectSingleNode('.//c:chart', $drawingNamespaces)
$sourceRelationship = $chartNode.GetAttribute('id', $relationshipNamespace)
$sourceRelationshipNode = $relationshipsDocument.SelectSingleNode("/r:Relationships/r:Relationship[@Id='$sourceRelationship']", $relationshipsNamespaces)
if ($null -eq $sourceRelationshipNode -or -not $sourceRelationshipNode.GetAttribute('Target').EndsWith('/chart12.xml')) {
    throw 'The BHA polar chart drawing anchor could not be identified.'
}
$overlayAnchor = $chartAnchor.CloneNode($true)
$overlayChartNode = $overlayAnchor.SelectSingleNode('.//c:chart', $drawingNamespaces)
[void]$overlayChartNode.SetAttribute('id', $relationshipNamespace, $newRelationshipId)
$maxDrawingId = 0
foreach ($properties in @($drawingDocument.SelectNodes('//xdr:cNvPr', $drawingNamespaces))) {
    $id = 0
    if ([int]::TryParse($properties.GetAttribute('id'), [ref]$id) -and $id -gt $maxDrawingId) { $maxDrawingId = $id }
}
$overlayProperties = $overlayAnchor.SelectSingleNode('.//xdr:cNvPr', $drawingNamespaces)
$overlayProperties.SetAttribute('id', [string]($maxDrawingId + 1))
$overlayProperties.SetAttribute('name', 'Chart 16')
[void]$drawingDocument.DocumentElement.AppendChild($overlayAnchor)
$entries[$drawingPart] = Get-XmlBytes $drawingDocument

$contentTypesDocument = Get-XmlDocument $entries[$contentTypesPart]
$contentTypesNamespaces = [System.Xml.XmlNamespaceManager]::new($contentTypesDocument.NameTable)
$contentTypesNamespaces.AddNamespace('ct', $contentTypesNamespace)
if ($null -eq $contentTypesDocument.SelectSingleNode("/ct:Types/ct:Override[@PartName='/xl/drawings/charts/chart16.xml']", $contentTypesNamespaces)) {
    $override = $contentTypesDocument.CreateElement('Override', $contentTypesNamespace)
    $override.SetAttribute('PartName', '/xl/drawings/charts/chart16.xml')
    $override.SetAttribute('ContentType', 'application/vnd.openxmlformats-officedocument.drawingml.chart+xml')
    [void]$contentTypesDocument.DocumentElement.AppendChild($override)
}
$entries[$contentTypesPart] = Get-XmlBytes $contentTypesDocument

if (Test-Path -LiteralPath $outputFullPath) { Remove-Item -LiteralPath $outputFullPath -Force }
$outputArchive = [System.IO.Compression.ZipFile]::Open($outputFullPath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($partName in $entries.Keys) {
        $entry = $outputArchive.CreateEntry($partName, [System.IO.Compression.CompressionLevel]::Optimal)
        $stream = $entry.Open()
        try { $stream.Write($entries[$partName], 0, $entries[$partName].Length) }
        finally { $stream.Dispose() }
    }
}
finally {
    $outputArchive.Dispose()
}

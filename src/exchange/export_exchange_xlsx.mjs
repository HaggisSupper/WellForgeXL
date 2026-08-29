import JSZip from 'jszip';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';

const XLSX_MIME = 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';
const FIXED_ZIP_DATE = new Date(1980, 0, 1, 0, 0, 0);
const protectedSheets = Object.freeze(['Exchange Map', 'Exchange State']);

function sheetTag(xml, name) {
  return xml.match(new RegExp(`<x:sheet\\b[^>]*name="${name}"[^>]*/>`))?.[0];
}

function relationshipTarget(workbookXml, relationshipsXml, name) {
  const tag = sheetTag(workbookXml, name);
  const id = tag?.match(/r:id="([^"]+)"/)?.[1];
  const relationship = id && [...relationshipsXml.matchAll(/<Relationship\b[^>]*\/>/g)]
    .map(([value]) => value)
    .find((value) => value.match(/\bId="([^"]+)"/)?.[1] === id);
  const target = relationship?.match(/\bTarget="([^"]+)"/)?.[1];
  if (!target) throw new Error(`Unable to resolve serialized worksheet ${name}`);
  return target.startsWith('/xl/') ? target.slice(1) : `xl/${target.replace(/^\//, '')}`;
}

function setSheetState(workbookXml, name, state) {
  const tag = sheetTag(workbookXml, name);
  if (!tag) throw new Error(`Serialized workbook is missing ${name}`);
  const withoutState = tag.replace(/\sstate="[^"]*"/, '');
  const replacement = state === 'visible' ? withoutState : withoutState.replace('/>', ` state="${state}"/>`);
  return workbookXml.replace(tag, replacement);
}

function ensureSheetProtection(worksheetXml, editableRange = undefined) {
  const withoutProtection = worksheetXml
    .replace(/<x:sheetProtection\b[^>]*\/?>(?:<\/x:sheetProtection>)?/, '')
    .replace(/<x:protectedRanges\b[^>]*>[\s\S]*?<\/x:protectedRanges>/, '');
  const sheetDataEnd = withoutProtection.indexOf('</x:sheetData>');
  if (sheetDataEnd < 0) throw new Error('Serialized worksheet is missing sheetData');
  const afterSheetData = sheetDataEnd + '</x:sheetData>'.length;
  const calcProperties = withoutProtection.slice(afterSheetData).match(/^\s*<x:sheetCalcPr\b[^>]*\/>/)?.[0] ?? '';
  const insertionPoint = afterSheetData + calcProperties.length;
  const protection = '<x:sheetProtection sheet="1" objects="1" scenarios="1"/>';
  const protectedRanges = editableRange
    ? `<x:protectedRanges><x:protectedRange name="ExchangeMapDocumentation" sqref="${editableRange}" /></x:protectedRanges>`
    : '';
  return `${withoutProtection.slice(0, insertionPoint)}${protection}${protectedRanges}${withoutProtection.slice(insertionPoint)}`;
}

function relationshipSourcePath(relationshipPath) {
  const marker = '/_rels/';
  const index = relationshipPath.lastIndexOf(marker);
  if (index < 0) return undefined;
  const directory = relationshipPath.slice(0, index);
  const relationshipName = relationshipPath.slice(index + marker.length);
  if (!relationshipName.endsWith('.rels')) return undefined;
  return `${directory}/${relationshipName.slice(0, -'.rels'.length)}`;
}

function replaceRelationshipReferences(sourceXml, replacements) {
  let normalized = sourceXml;
  for (const [previousId, nextId] of replacements) {
    const escaped = previousId.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    normalized = normalized.replace(new RegExp(`(["'])${escaped}\\1`, 'g'), (_, quote) => `${quote}${nextId}${quote}`);
  }
  return normalized;
}

async function normalizeRelationshipIds(zip) {
  const relationshipPaths = Object.keys(zip.files).filter((name) => name.endsWith('.rels')).sort();
  for (const relationshipPath of relationshipPaths) {
    const relationshipFile = zip.file(relationshipPath);
    if (!relationshipFile) continue;
    const xml = await relationshipFile.async('string');
    const tags = [...xml.matchAll(/<Relationship\b[^>]*\/>/g)].map(([tag]) => tag);
    const ordered = tags.map((tag) => ({
      tag,
      previousId: tag.match(/\bId="([^"]+)"/)?.[1],
      key: [tag.match(/\bType="([^"]+)"/)?.[1] ?? '', tag.match(/\bTarget="([^"]+)"/)?.[1] ?? '', tag.match(/\bTargetMode="([^"]+)"/)?.[1] ?? ''].join('\u0000'),
    })).sort((left, right) => (left.key < right.key ? -1 : left.key > right.key ? 1 : 0));
    if (ordered.some(({ previousId }) => !previousId)) throw new Error(`Invalid relationship ID in ${relationshipPath}`);
    const replacements = new Map(ordered.map(({ previousId }, index) => [previousId, `rId${index + 1}`]));
    const openingEnd = xml.indexOf('>', xml.indexOf('<Relationships')) + 1;
    const closingStart = xml.lastIndexOf('</Relationships>');
    if (openingEnd <= 0 || closingStart < openingEnd) throw new Error(`Invalid relationships XML in ${relationshipPath}`);
    const normalizedTags = ordered.map(({ tag, previousId }) => tag.replace(/\bId="[^"]+"/, `Id="${replacements.get(previousId)}"`));
    zip.file(relationshipPath, `${xml.slice(0, openingEnd)}${normalizedTags.join('')}${xml.slice(closingStart)}`);

    const sourcePath = relationshipSourcePath(relationshipPath);
    const sourceFile = sourcePath && zip.file(sourcePath);
    if (sourceFile) {
      const sourceXml = await sourceFile.async('string');
      zip.file(sourcePath, replaceRelationshipReferences(sourceXml, replacements));
    }
  }
}

// PolarPlotter2010 uses a single combo chart: radar for the polar grid and
// XY scatter for the actual data. artifact-tool exports those layers as two
// overlapping charts, so merge their OOXML plot areas after authoring.
async function mergePolarPlotterChart(zip) {
  const chartPaths = Object.keys(zip.files).filter((name) => /^xl\/drawings\/charts\/chart\d+\.xml$/.test(name));
  let gridPath; let dataPath; let gridXml; let dataXml;
  for (const chartPath of chartPaths) {
    const xml = await zip.file(chartPath).async('string');
    if (xml.includes('Polar grid')) { gridPath = chartPath; gridXml = xml; }
    if (xml.includes('WOB/toolface polar response')) { dataPath = chartPath; dataXml = xml; }
  }
  if (!gridPath || !dataPath) return;
  let radar = gridXml.match(/<c:radarChart>[\s\S]*?<\/c:radarChart>/)?.[0];
  let catAxes = [...gridXml.matchAll(/<c:catAx>[\s\S]*?<\/c:catAx>/g)].map(([value]) => value).join('');
  let valAxes = [...gridXml.matchAll(/<c:valAx>[\s\S]*?<\/c:valAx>/g)].map(([value]) => value).join('');
  if (!radar || !catAxes || !valAxes) throw new Error('Polar grid chart is missing radar or axis XML');
  // Artifact-tool assigns the same axis IDs to separately-created charts on a
  // worksheet. A valid Radar + XY combination requires an independent axis
  // pair for each chart group, matching the PolarPlotter construction.
  const radarAxisIds = [...new Set([...radar.matchAll(/<c:axId val="([^"]+)"/g)].map((match) => match[1]))];
  for (const oldId of radarAxisIds) {
    let candidate = String(Number(oldId) + 100000000);
    while (dataXml.includes(`val="${candidate}"`)) candidate = String(Number(candidate) + 1);
    const replaceAxisId = (xml) => xml.replaceAll(`val="${oldId}"`, `val="${candidate}"`);
    radar = replaceAxisId(radar);
    catAxes = replaceAxisId(catAxes);
    valAxes = replaceAxisId(valAxes);
  }
  let merged = dataXml.replace('<c:scatterChart>', `${radar}<c:scatterChart>`).replace('</c:plotArea>', `${catAxes}${valAxes}</c:plotArea>`);
  merged = merged.replace(/<c:scatterStyle val="marker"\/>/g, '<c:scatterStyle val="lineMarker"/>');
  zip.file(dataPath, merged);

  const gridName = gridPath.split('/').at(-1);
  const relationshipPaths = Object.keys(zip.files).filter((name) => /^xl\/drawings\/_rels\/drawing\d+\.xml\.rels$/.test(name));
  for (const relationshipPath of relationshipPaths) {
    const relationshipXml = await zip.file(relationshipPath).async('string');
    const relationshipTag = [...relationshipXml.matchAll(/<Relationship\b[^>]*\/>/g)].map(([value]) => value).find((value) => value.includes(gridName));
    if (!relationshipTag) continue;
    const relationshipId = relationshipTag.match(/\bId="([^"]+)"/)?.[1];
    const drawingPath = relationshipSourcePath(relationshipPath);
    const drawingFile = drawingPath && zip.file(drawingPath);
    if (!relationshipId || !drawingFile) throw new Error('Unable to resolve Polar grid drawing anchor');
    const drawingXml = await drawingFile.async('string');
    const anchors = [...drawingXml.matchAll(/<xdr:twoCellAnchor>[\s\S]*?<\/xdr:twoCellAnchor>/g)].map(([value]) => value);
    const gridAnchor = anchors.find((value) => value.includes(`r:id="${relationshipId}"`));
    if (!gridAnchor) throw new Error('Unable to locate Polar grid anchor');
    zip.file(drawingPath, drawingXml.replace(gridAnchor, ''));
    zip.file(relationshipPath, relationshipXml.replace(relationshipTag, ''));
    zip.remove(gridPath);
    const contentTypesPath = '[Content_Types].xml';
    const contentTypesFile = zip.file(contentTypesPath);
    if (!contentTypesFile) throw new Error('OOXML package is missing its content-type manifest');
    const contentTypesXml = await contentTypesFile.async('string');
    const partName = `/${gridPath}`;
    const escapedPartName = partName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const overridePattern = new RegExp(`<Override\\b(?=[^>]*\\bPartName="${escapedPartName}")[^>]*/>`);
    const normalizedContentTypes = contentTypesXml.replace(overridePattern, '');
    if (normalizedContentTypes === contentTypesXml) throw new Error(`Unable to remove content-type declaration for ${partName}`);
    zip.file(contentTypesPath, normalizedContentTypes);
    break;
  }
}

async function normalizeDepthProfileCharts(zip) {
  const chartPaths = Object.keys(zip.files).filter((name) => /^xl\/drawings\/charts\/chart\d+\.xml$/.test(name));
  for (const chartPath of chartPaths) {
    const chartFile = zip.file(chartPath);
    if (!chartFile) continue;
    const xml = await chartFile.async('string');
    const isDepthRoadmap = xml.includes('<c:scatterChart>')
      && xml.includes('<c:yVal>')
      && /<c:orientation val="maxMin"\s*\/>/.test(xml);
    const isPolarTrace = xml.includes('<c:scatterChart>')
      && (xml.includes('Toolface / WOB rose plot') || xml.includes('WOB/toolface polar response'));
    const isEngineeringEnvelope = xml.includes('<c:scatterChart>')
      && (xml.includes('Nozzle pressure envelope') || xml.includes('Nozzle optimization'));
    const isBhaScreeningProfile = xml.includes('<c:scatterChart>')
      && (xml.includes('BHA geometry projection')
        || xml.includes('Projected radial clearance')
        || xml.includes('Bending moment versus distance')
        || xml.includes('Bending stress versus distance')
        || xml.includes('Screening Campbell diagram')
        || xml.includes('Component modal frequencies'));
    if (!isDepthRoadmap && !isPolarTrace && !isEngineeringEnvelope && !isBhaScreeningProfile) continue;
    let normalized = xml.replace(/<c:scatterStyle val="marker"\s*\/>/g, '<c:scatterStyle val="lineMarker"/>');
    if (isPolarTrace) {
      normalized = normalized.replace(/<a:srgbClr val="(0F766E|D97706)"\s*\/>/g, '<a:srgbClr val="$1"><a:alpha val="65000"/></a:srgbClr>');
    }
    if (!isDepthRoadmap) {
      zip.file(chartPath, normalized);
      continue;
    }
    normalized = normalized.replace(/<c:valAx>[\s\S]*?<\/c:valAx>/g, (axisXml) => {
      if (/<c:orientation val="maxMin"\s*\/>/.test(axisXml)) {
        return axisXml.replace(/<c:axPos val="[^"]+"\s*\/>/, '<c:axPos val="l"/>');
      }
      if (/<c:orientation val="minMax"\s*\/>/.test(axisXml)) {
        return axisXml.replace(/<c:axPos val="[^"]+"\s*\/>/, '<c:axPos val="t"/>');
      }
      return axisXml;
    });
    zip.file(chartPath, normalized);
  }
}

async function deterministicBlob(zip) {
  const normalized = new JSZip();
  const names = Object.keys(zip.files).filter((name) => !zip.files[name].dir).sort();
  for (const name of names) {
    const contents = await zip.file(name).async('uint8array');
    normalized.file(name, contents, {
      binary: true,
      createFolders: false,
      date: FIXED_ZIP_DATE,
      compression: 'DEFLATE',
      compressionOptions: { level: 9 },
    });
  }
  const data = await normalized.generateAsync({
    type: 'uint8array',
    platform: 'DOS',
    compression: 'DEFLATE',
    compressionOptions: { level: 9 },
    mimeType: XLSX_MIME,
  });
  return new FileBlob(data, XLSX_MIME);
}

export async function exportExchangeXlsx(workbook) {
  const raw = await SpreadsheetFile.exportXlsx(workbook);
  const zip = await JSZip.loadAsync(raw.data);
  await normalizeDepthProfileCharts(zip);
  await mergePolarPlotterChart(zip);
  await normalizeRelationshipIds(zip);

  let workbookXml = await zip.file('xl/workbook.xml').async('string');
  const relationshipsXml = await zip.file('xl/_rels/workbook.xml.rels').async('string');
  workbookXml = setSheetState(workbookXml, 'Exchange Map', 'visible');
  workbookXml = setSheetState(workbookXml, 'Exchange Buffer', 'visible');
  workbookXml = setSheetState(workbookXml, 'Exchange State', 'hidden');
  workbookXml = setSheetState(workbookXml, 'Calc', 'hidden');
  zip.file('xl/workbook.xml', workbookXml);

  for (const name of protectedSheets) {
    const worksheetPath = relationshipTarget(workbookXml, relationshipsXml, name);
    const worksheetFile = zip.file(worksheetPath);
    if (!worksheetFile) throw new Error(`Unable to read serialized worksheet ${name}`);
    const worksheetXml = await worksheetFile.async('string');
    zip.file(worksheetPath, ensureSheetProtection(worksheetXml, name === 'Exchange Map' ? 'A3:M3' : undefined));
  }
  return deterministicBlob(zip);
}

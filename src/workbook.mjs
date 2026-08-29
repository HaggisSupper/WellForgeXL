import { Workbook } from '@oai/artifact-tool';
import { UNIT_SYSTEMS, CUSTOM_UNIT_SYSTEMS, UNIT_ROWS, COLORS, baseSheet, formatInput, formatOutput, sectionHeader } from './common.mjs';
import { EXCHANGE_SHEET_NAMES } from './exchange/workbook_maps.mjs';

export const DEFAULT_SHEET_NAMES = Object.freeze(['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEET_NAMES]);

export function createSuiteWorkbook(title, options = {}) {
  const workbook = Workbook.create();
  let names = options.sheetNames ? [...options.sheetNames] : (options.extraSheetNames?.length
    ? ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', ...options.extraSheetNames, 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEET_NAMES]
    : DEFAULT_SHEET_NAMES);
  if (!names.includes('Chart Settings')) {
    const unitMapIndex = names.indexOf('Unit Map');
    names.splice(unitMapIndex >= 0 ? unitMapIndex : names.length, 0, 'Chart Settings');
  }
  if (new Set(names).size !== names.length) throw new Error('Duplicate sheet name in workbook topology');
  const sheets = Object.fromEntries(names.map((name) => [name, workbook.worksheets.add(name)]));
  for (const [name, sheet] of Object.entries(sheets)) baseSheet(sheet, name === 'Summary' ? title : name);
  if (sheets.Calc) sheets.Calc.visibility = 'hidden';
  addChartSettings(sheets['Chart Settings']);
  addUnitMap(sheets['Unit Map']);
  addChecks(sheets.Checks);
  return { workbook, sheets };
}

export function addChartSettings(sheet) {
  sectionHeader(sheet, 'A3:C3', 'Persisted engineering-chart configuration');
  sheet.getRange('D3:J3').merge();
  sheet.getRange('D3').values = [['These settings are saved with the case and reapplied by the VBA engine.']];
  sheet.getRange('D3:J3').format = { fill: COLORS.tealLight, font: { italic: true, color: COLORS.charcoal } };
  sheet.getRange('A5:C5').values = [['Setting', 'Value', 'Purpose']];
  sheet.getRange('A6:C13').values = [
    ['Selected MD', 0, 'Canonical SI depth used by the screen reader'],
    ['Observed data visible', 'Yes', 'Show measured/observed series when supplied'],
    ['Low friction multiplier', 0.8, 'Low sensitivity family'],
    ['Base friction multiplier', 1, 'Base model'],
    ['High friction multiplier', 1.2, 'High sensitivity family'],
    ['Show well context', 'Yes', 'Show section/component boundary context'],
    ['Show limits', 'Yes', 'Show ratings, thresholds, and operating envelopes'],
    ['Report composition', 'Engineering review', 'Saved chart/report selection'],
  ];
  tableHeader(sheet, 'A5:C5');
  inputTableStyle(sheet, 'B6:B13');
  sheet.getRange('B7').dataValidation = { rule: { type: 'list', values: ['Yes', 'No'] } };
  sheet.getRange('B11:B12').dataValidation = { rule: { type: 'list', values: ['Yes', 'No'] } };
  sheet.getRange('A:A').format.columnWidth = 26;
  sheet.getRange('B:B').format.columnWidth = 18;
  sheet.getRange('C:C').format.columnWidth = 44;
}

export function addUnitMap(sheet) {
  sectionHeader(sheet, 'A3:J3', 'Display-unit control — calculations and stored inputs remain SI');
  sheet.getRange('J:J').format.columnWidth = 18;
  sheet.getRange('A5:B5').values = [['Display system', 'SI']];
  formatInput(sheet.getRange('B5'));
  sheet.getRange('B5').dataValidation = { rule: { type: 'list', values: UNIT_SYSTEMS } };
  sheet.getRange('D5:J5').merge();
  sheet.getRange('D5').values = [['Choose Custom, then set each domain dropdown in column J.']];
  sheet.getRange('D5:J5').format = { fill: COLORS.tealLight, font: { italic: true, color: COLORS.charcoal } };
  sheet.getRange('A7:J7').values = [['Domain', 'SI unit', 'Imperial', 'Mixed', 'Imp factor', 'Mixed factor', 'Offset', 'Selected unit', 'Selected factor', 'Custom selection']];
  sheet.getRange(`A8:G${7 + UNIT_ROWS.length}`).values = UNIT_ROWS.map((row) => [row.domain, row.siUnit, row.imperialUnit, row.mixedUnit, row.imperialMultiplier, row.mixedMultiplier, row.offset]);
  for (let r = 8; r < 8 + UNIT_ROWS.length; r += 1) {
    sheet.getRange(`J${r}`).values = [['SI']];
    sheet.getRange(`J${r}`).dataValidation = { rule: { type: 'list', values: CUSTOM_UNIT_SYSTEMS } };
    sheet.getRange(`H${r}`).formulas = [[`=IF($B$5="SI",B${r},IF($B$5="Imperial",C${r},IF($B$5="Mixed",D${r},IF($B$5="Custom",IF(J${r}="SI",B${r},IF(J${r}="Imperial",C${r},IF(J${r}="Mixed",D${r},"INVALID"))),"INVALID"))))`]];
    sheet.getRange(`I${r}`).formulas = [[`=IF($B$5="SI",1,IF($B$5="Imperial",E${r},IF($B$5="Mixed",F${r},IF($B$5="Custom",IF(J${r}="SI",1,IF(J${r}="Imperial",E${r},IF(J${r}="Mixed",F${r},NA()))),NA()))))`]];
  }
  formatInput(sheet.getRange(`J8:J${7 + UNIT_ROWS.length}`));
  sheet.getRange(`A7:J${7 + UNIT_ROWS.length}`).format.borders = { preset: 'all', style: 'thin', color: COLORS.line };
  sheet.getRange('A7:J7').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
  sheet.getRange(`E8:G${7 + UNIT_ROWS.length}`).format.numberFormat = '0.000000';
}

export function addChecks(sheet) {
  sectionHeader(sheet, 'A3:D3', 'Model checks and operating-status gates');
  sheet.getRange('A5:D5').values = [['Check', 'Result', 'Status', 'Required action']];
  sheet.getRange('A6:D10').values = [
    ['SI input source', 'All calculation sources use SI units', 'INFO', 'Select display units only in Unit Map'],
    ['Input completeness', 'Refer to Inputs input-status cells', 'PENDING', 'Resolve blanks / invalid geometry'],
    ['Limit screening', 'Refer to Summary governing utilisation', 'PENDING', 'Review any value above 100%'],
    ['Method basis', 'Screening model; not controlled operational approval', 'INFO', 'Use approved engineering workflow for sign-off'],
    ['External links / VBA', 'None', 'PASS', 'Workbook remains portable and auditable'],
  ];
  sheet.getRange('A5:D10').format.borders = { preset: 'all', style: 'thin', color: COLORS.line };
  sheet.getRange('A5:D5').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
}

export function tableHeader(sheet, range) {
  sheet.getRange(range).format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, borders: { preset: 'all', style: 'thin', color: COLORS.line } };
}

export function resultsTableStyle(sheet, range) {
  sheet.getRange(range).format.borders = { preset: 'all', style: 'thin', color: COLORS.line };
  formatOutput(sheet.getRange(range));
}

export function inputTableStyle(sheet, range) {
  sheet.getRange(range).format.borders = { preset: 'all', style: 'thin', color: COLORS.line };
  formatInput(sheet.getRange(range));
}

export function addLineChart(sheet, sourceRange, title, startCell, endCell) {
  const chart = sheet.charts.add('line', sheet.getRange(sourceRange));
  chart.title = title;
  chart.hasLegend = true;
  chart.xAxis = { axisType: 'textAxis' };
  chart.setPosition(startCell, endCell);
  return chart;
}

export function addScatterChart(sheet, sourceRange, title, startCell, endCell) {
  const match = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/i.exec(sourceRange);
  if (!match) throw new Error(`Invalid XY source range: ${sourceRange}`);
  const [, firstColumnText, headerRowText, lastColumnText, lastRowText] = match;
  const firstColumn = columnNumber(firstColumnText.toUpperCase());
  const lastColumn = columnNumber(lastColumnText.toUpperCase());
  const headerRow = Number(headerRowText);
  const lastRow = Number(lastRowText);
  if (lastColumn <= firstColumn || lastRow <= headerRow) throw new Error(`XY range needs a header, X column, and response column: ${sourceRange}`);

  const chart = sheet.charts.add('scatter', sheet.getRange('A1:B2'));
  chart.title = title;
  chart.hasLegend = true;
  try { chart.scatterStyle = 'lineMarker'; } catch { /* renderer compatibility */ }
  chart.series.deleteAll();
  const xColumn = columnLetters(firstColumn);
  const xFormula = absoluteRangeFormula(sheet.name, xColumn, headerRow + 1, lastRow);
  const headers = sheet.getRange(`${firstColumnText}${headerRow}:${lastColumnText}${headerRow}`).values[0];
  for (let column = firstColumn + 1; column <= lastColumn; column += 1) {
    const yColumn = columnLetters(column);
    const series = chart.series.add(headers[column - firstColumn] ?? `Series ${column - firstColumn}`);
    series.xFormula = xFormula;
    series.formula = absoluteRangeFormula(sheet.name, yColumn, headerRow + 1, lastRow);
  }
  chart.xAxis = { axisType: 'value' };
  chart.yAxis = { axisType: 'value' };
  chart.xAxis.title = { text: headers[0] ?? 'X' };
  chart.yAxis.title = { text: headers[1] ?? 'Y' };
  chart.setPosition(startCell, endCell);
  return chart;
}

function columnNumber(columnLetters) {
  return [...columnLetters].reduce((value, letter) => value * 26 + letter.charCodeAt(0) - 64, 0);
}

function columnLetters(columnNumberValue) {
  let value = columnNumberValue;
  let result = '';
  while (value > 0) {
    value -= 1;
    result = String.fromCharCode(65 + (value % 26)) + result;
    value = Math.floor(value / 26);
  }
  return result;
}

function absoluteRangeFormula(sheetName, column, firstRow, lastRow) {
  const escapedSheetName = sheetName.replaceAll("'", "''");
  return `='${escapedSheetName}'!$${column}$${firstRow}:$${column}$${lastRow}`;
}

// Drilling depth-roadmap convention: each calculated response is an X series;
// MD/TVD is the shared Y series and increases down the page from zero at top.
export function addDepthProfileChart(chartSheet, dataSheet, sourceRange, title, startCell, endCell, options = {}) {
  const match = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/i.exec(sourceRange);
  if (!match) throw new Error(`Invalid depth-profile source range: ${sourceRange}`);
  const [, firstColumnText, headerRowText, lastColumnText, lastRowText] = match;
  const firstColumn = columnNumber(firstColumnText.toUpperCase());
  const lastColumn = columnNumber(lastColumnText.toUpperCase());
  const headerRow = Number(headerRowText);
  const lastRow = Number(lastRowText);
  if (lastColumn <= firstColumn || lastRow <= headerRow) throw new Error(`Depth-profile range needs a header, depth, and response column: ${sourceRange}`);

  const chart = chartSheet.charts.add('scatter', chartSheet.getRange('A1:B2'));
  chart.title = title;
  chart.hasLegend = options.hasLegend ?? true;
  try { chart.scatterStyle = options.scatterStyle ?? 'lineMarker'; } catch { /* renderer compatibility */ }
  chart.series.deleteAll();

  const depthColumn = columnLetters(firstColumn);
  const depthFormula = absoluteRangeFormula(dataSheet.name, depthColumn, headerRow + 1, lastRow);
  const headers = dataSheet.getRange(`${firstColumnText}${headerRow}:${lastColumnText}${headerRow}`).values[0];
  for (let column = firstColumn + 1; column <= lastColumn; column += 1) {
    const responseColumn = columnLetters(column);
    const index = column - firstColumn - 1;
    const series = chart.series.add(options.seriesNames?.[index] ?? headers[index + 1] ?? `Series ${index + 1}`);
    series.xFormula = absoluteRangeFormula(dataSheet.name, responseColumn, headerRow + 1, lastRow);
    series.formula = depthFormula;
    const style = options.seriesStyles?.[index];
    if (style) {
      try { if (style.color) series.format.line.color = style.color; } catch { /* renderer compatibility */ }
      try { if (style.transparency != null) series.format.line.transparency = style.transparency; } catch { /* renderer compatibility */ }
      try { if (style.weight != null) series.format.line.weight = style.weight; } catch { /* renderer compatibility */ }
    }
  }

  chart.xAxis = { axisType: 'value' };
  chart.yAxis = { axisType: 'value' };
  chart.yAxis.orientation = 'maxMin';
  if (options.xTitle) chart.xAxis.title = { text: options.xTitle };
  if (options.depthTitle) chart.yAxis.title = { text: options.depthTitle };
  chart.setPosition(startCell, endCell);
  return chart;
}

// Andy Pope / PolarPlotter-inspired construction: use true XY geometry for
// the data and keep the polar grid as a separate, auditable helper layer.
// `series` entries are {name, xRange, yRange}; ranges are sheet-local or
// fully-qualified formulas accepted by the chart API.
export function addPolarScatterChart(sheet, series, title, startCell, endCell, options = {}) {
  const chart = sheet.charts.add(options.chartType ?? 'scatter', sheet.getRange('A1:B2'));
  chart.title = title;
  chart.hasLegend = options.hasLegend ?? true;
  try { chart.scatterStyle = options.scatterStyle ?? 'lineMarker'; } catch { /* renderer compatibility */ }
  chart.series.deleteAll();
  for (const spec of series) {
    const s = chart.series.add(spec.name);
    s.xFormula = spec.xRange;
    s.formula = spec.yRange;
    if (spec.lineColor && s.format?.line) {
      try { s.format.line.color = spec.lineColor; } catch { /* renderer compatibility */ }
    }
    if (spec.transparency != null && s.format?.line) {
      try { s.format.line.transparency = spec.transparency; } catch { /* renderer compatibility */ }
    }
  }
  chart.xAxis = { axisType: 'value' };
  chart.yAxis = { axisType: 'value' };
  if (options.xTitle) chart.xAxis.title = { text: options.xTitle };
  if (options.yTitle) chart.yAxis.title = { text: options.yTitle };
  chart.setPosition(startCell, endCell);
  return chart;
}

export function addHeatmapConditionalFormatting(range, min = 0, max = 1) {
  range.conditionalFormats.addColorScale({
    minColor: COLORS.tealLight,
    midColor: COLORS.amberLight,
    maxColor: COLORS.redLight,
  });
}

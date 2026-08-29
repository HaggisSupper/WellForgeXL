import { createHash } from 'node:crypto';
import { createSuiteWorkbook, tableHeader } from './workbook.mjs';
import { applyTwoDecimalDisplayPrecision, COLORS, sectionHeader } from './common.mjs';
import { directionalReferenceData } from './directional_reference_data.mjs';
import { addExchangeSheets } from './exchange/add_exchange_sheets.mjs';
import { DIRECTIONAL_INPUT_CELLS, DIRECTIONAL_SHEET_NAMES, DIRECTIONAL_TABLES } from './directional_contract.mjs';
import * as f from './directional_formulas.mjs';
import { buildCanonicalModel, buildDecisionSurfaces, buildDirectionalCharts, buildTargetSlideFormation, linkVisibleTables } from './directional_workbook_model.mjs';

const formulaFactories = Object.freeze({
  doglegAngle: f.doglegAngleFormula, ratioFactor: f.ratioFactorFormula, deltaTvd: f.deltaTvdFormula,
  deltaNorth: f.deltaNorthFormula, deltaEast: f.deltaEastFormula, doglegSeverity: f.doglegSeverityFormula,
  slerpNorth: f.slerpNorthFormula, slerpEast: f.slerpEastFormula, slerpVertical: f.slerpVerticalFormula,
  partialPosition: f.partialPositionFormula, crosslineError: f.crosslineErrorFormula, error3d: f.error3dFormula,
  effectiveTurn: f.effectiveTurnFormula, responseToolface: f.responseToolfaceFormula, targetEnvelope: f.targetEnvelopeFormula,
  formationHighLow: f.formationHighLowFormula,
});

export function directionalFormulaPlan(row = 7) {
  return Object.fromEntries(Object.entries(formulaFactories).map(([key, factory]) => [key, factory(row)]));
}

function validation(range, rule) { range.dataValidation = { rule }; }
function positive(range, allowZero = false) { validation(range, { type: 'decimal', operator: allowZero ? 'greaterThanOrEqual' : 'greaterThan', formula1: 0 }); }
function between(range, low, high) { validation(range, { type: 'decimal', operator: 'between', formula1: low, formula2: high }); }

function fixtureContentHash(role, value) {
  return `sha256:${createHash('sha256').update(JSON.stringify({ role, value })).digest('hex')}`;
}

function buildTrajectoryProvenanceSurface(sheet) {
  sectionHeader(sheet, 'P3:V3', 'Rust trajectory provenance — explicit authoritative identities');
  sheet.getRange('P5:V5').values = [['Role', 'UUID', 'URI', 'Object type', 'Content hash', 'Citation name', 'Source system']];
  sheet.getRange('P6:V9').values = [
    ['Well', '11111111-1111-4111-8111-111111111111', 'eml:///wellforge/directional/well/1', 'well', fixtureContentHash('well', directionalReferenceData.metadata), 'Sanitized directional fixture well', 'wellforge-directional-fixture/1.0'],
    ['Wellbore', '22222222-2222-4222-8222-222222222222', 'eml:///wellforge/directional/wellbore/1', 'wellbore', fixtureContentHash('wellbore', directionalReferenceData.metadata), 'Sanitized directional fixture wellbore', 'wellforge-directional-fixture/1.0'],
    ['Plan trajectory', '33333333-3333-4333-8333-333333333333', 'eml:///wellforge/directional/trajectory/plan/1', 'trajectory', fixtureContentHash('plan', directionalReferenceData.plan), 'Sanitized directional fixture plan', 'wellforge-directional-fixture/1.0'],
    ['Survey trajectory', '44444444-4444-4444-8444-444444444444', 'eml:///wellforge/directional/trajectory/survey/1', 'trajectory', fixtureContentHash('survey', directionalReferenceData.survey), 'Sanitized directional fixture survey', 'wellforge-directional-fixture/1.0'],
  ];
  sheet.getRange('P12:P17').values = [['Analysis UUID'], ['MD datum UUID'], ['MD datum name'], ['MD datum kind'], ['Azimuth reference'], ['Contract version']];
  sheet.getRange('Q12:Q17').values = [
    ['aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'],
    ['55555555-5555-4555-8555-555555555555'],
    ['RKB'],
    ['rotary_kelly_bushing'],
    ['grid_north'],
    ['1.0.0'],
  ];
  sheet.getRange('Q6:V9').format.fill = COLORS.input;
  sheet.getRange('Q12:Q17').format.fill = COLORS.input;
  validation(sheet.getRange('Q15'), { type: 'list', values: ['rotary_kelly_bushing', 'drill_floor', 'mean_sea_level', 'other'] });
  validation(sheet.getRange('Q16'), { type: 'list', values: ['true_north', 'grid_north', 'magnetic_north'] });
  sheet.getRange('P:V').format.columnWidth = 22;
  sheet.getRange('P:P').format.columnWidth = 20;
  sheet.getRange('R:R').format.columnWidth = 42;
  sheet.getRange('T:T').format.columnWidth = 74;
  sheet.getRange('U:V').format.columnWidth = 34;
}

function trajectoryRowUuid(namespace, index) {
  return `00000000-0000-4000-8${namespace.toString(16).padStart(3, '0')}-${index.toString(16).padStart(12, '0')}`;
}

function buildTrajectoryIdentitySurface(sheet) {
  sheet.getRange('JA6:JE6').values = [['Plan UUID', 'Survey UUID', 'Target UUID', 'Slide UUID', 'Formation UUID']];
  sheet.getRange('JA7:JA506').values = Array.from({ length: 500 }, (_, index) => [trajectoryRowUuid(1, index + 1)]);
  sheet.getRange('JB7:JB506').values = Array.from({ length: 500 }, (_, index) => [trajectoryRowUuid(2, index + 1)]);
  sheet.getRange('JC7:JC106').values = Array.from({ length: 100 }, (_, index) => [trajectoryRowUuid(3, index + 1)]);
  sheet.getRange('JD7:JD206').values = Array.from({ length: 200 }, (_, index) => [trajectoryRowUuid(4, index + 1)]);
  sheet.getRange('JE7:JE106').values = Array.from({ length: 100 }, (_, index) => [trajectoryRowUuid(5, index + 1)]);
  sheet.getRange('JA6:JE6').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
}

function buildInputs(sheet) {
  sheet.getRange('A1:N1').unmerge();
  sheet.getRange('A1:N1').merge();
  sectionHeader(sheet, 'A3:B3', 'Well / reference metadata');
  sectionHeader(sheet, 'D3:E3', 'Raw input units');
  sectionHeader(sheet, 'G3:H3', 'DLS operating limit');
  sectionHeader(sheet, 'J3:K3', 'Deterministic projection');
  sectionHeader(sheet, 'M3:N3', 'Quality controls');
  const meta = DIRECTIONAL_INPUT_CELLS.metadata;
  const labels = ['Well name', 'Well identifier', 'Operator', 'Field / pad', 'Rig', 'Datum', 'North reference', 'Coordinate reference', 'Surface North', 'Surface East', 'Ground elevation', 'Vertical-section azimuth'];
  sheet.getRange('A5:A16').values = labels.map((label) => [label]);
  const fixtureMeta = Object.fromEntries(directionalReferenceData.metadata.map(({ key, value }) => [key, value]));
  sheet.getRange('B5:B16').values = [[fixtureMeta.wellName], [''], [''], [fixtureMeta.fieldPad], [fixtureMeta.rig], [fixtureMeta.datum], [fixtureMeta.northReference], ['Local grid coordinates; surface origin in B13:B14'], [fixtureMeta.surfaceNorthFt], [fixtureMeta.surfaceEastFt], [fixtureMeta.groundElevationFt], [fixtureMeta.verticalSectionAzimuthDeg]];
  sheet.getRange('B5:B16').format.fill = COLORS.input;
  const raw = DIRECTIONAL_INPUT_CELLS.rawUnits;
  sheet.getRange('D5:D11').values = [['Plan length'], ['Plan angle'], ['Survey length'], ['Survey angle'], ['Target length'], ['Slide length'], ['Formation length']];
  sheet.getRange('E5:E11').values = [['ft'], ['deg'], ['ft'], ['deg'], ['ft'], ['ft'], ['ft']];
  sheet.getRange('E5:E11').format.fill = COLORS.input;
  for (const cell of [raw.planLength, raw.surveyLength, raw.targetLength, raw.slideLength, raw.formationLength]) validation(sheet.getRange(cell), { type: 'list', values: ['m', 'ft'] });
  for (const cell of [raw.planAngle, raw.surveyAngle]) validation(sheet.getRange(cell), { type: 'list', values: ['deg', 'rad'] });
  sheet.getRange('G5:G7').values = [['DLS limit'], ['DLS input unit'], ['Source / reference']];
  sheet.getRange('H5:H7').values = [[10], ['deg/100ft'], ['User/operator limit; validate against approved program']];
  sheet.getRange('H5:H7').format.fill = COLORS.input;
  positive(sheet.getRange('H5'));
  validation(sheet.getRange('H6'), { type: 'list', values: ['rad/m', 'deg/100ft', 'deg/30m'] });
  sheet.getRange('J5:J9').values = [['Projection-to-bit MD'], ['Project-ahead distance'], ['Build tendency'], ['Effective-turn tendency'], ['Gradient input unit']];
  sheet.getRange('K5:K9').values = [[16140.816], [500], [0.3], [0], ['deg/100ft']];
  sheet.getRange('K5:K9').format.fill = COLORS.input;
  positive(sheet.getRange('K5')); positive(sheet.getRange('K6'), true);
  validation(sheet.getRange('K9'), { type: 'list', values: ['rad/m', 'deg/100ft', 'deg/30m'] });
  sheet.getRange('M5:M9').values = [['Low-inclination threshold'], ['Minimum slide length'], ['Slide-yield outlier limit'], ['Calibration window (stands)'], ['Survey-gap warning']];
  sheet.getRange('N5:N9').values = [[5], [10], [20], [3], [600]];
  sheet.getRange('N5:N9').format.fill = COLORS.input;
  between(sheet.getRange('N5'), 0, 180); positive(sheet.getRange('N6')); positive(sheet.getRange('N7'));
  validation(sheet.getRange('N8'), { type: 'whole', operator: 'between', formula1: 1, formula2: 200 }); positive(sheet.getRange('N9'));
  sheet.getRange('D14:N14').merge();
  sheet.getRange('D14').values = [['Display units: Unit Map!B5 is display-only; raw selectors above control input conversion and canonical calculations remain SI.']];
  sheet.getRange('D14:N14').format = { fill: COLORS.grey, wrapText: true };
  sheet.getRange('A19:N19').merge();
  sheet.getRange('A19').values = [['Method / scope notes']];
  sheet.getRange('A19:N19').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
  sheet.getRange('A20:N23').merge(true);
  sheet.getRange('A20:A23').values = [
    ['Minimum curvature; deterministic projection; planning/review only. Surface North/East use the Plan length selector; target coordinates use their selector and are translated to the local surface origin for Rust. Bounded table rows are fixed identity slots: edit in place; do not sort or reorder.'],
    ['No ISCWSA covariance. No anti-collision calculation or separation-factor result.'],
    ['No pipe-fatigue calculation.'],
    ['RUST REQUIRED — NO VBA FALLBACK; no external links and no VBA physics; a hash-verified local Rust executable is required.'],
  ];
  sheet.getRange('A20:N23').format = { fill: COLORS.grey, wrapText: true };
  sheet.getRange('A:N').format.columnWidth = 15;
  sheet.getRange('A:A').format.columnWidth = 24;
  sheet.getRange('D:D').format.columnWidth = 22;
  sheet.getRange('G:G').format.columnWidth = 22;
  sheet.getRange('H:H').format.columnWidth = 27;
  sheet.getRange('J:J').format.columnWidth = 23;
  sheet.getRange('K:K').format.columnWidth = 17;
  sheet.getRange('M:M').format.columnWidth = 24;
  sheet.getRange('N:N').format.columnWidth = 17;
  sheet.getRange('H7').format.wrapText = true;
  sheet.getRange('7:7').format.rowHeight = 32;
  buildTrajectoryProvenanceSurface(sheet);
}

function addCapacityAndTable(sheet, contract) {
  const lastLetter = Object.values(contract.columns).at(-1).letter;
  sheet.getRange('A1:N1').unmerge();
  sheet.getRange(`A1:${lastLetter}1`).merge();
  sheet.getRange('A1').values = [[`WellForge | ${contract.sheetName} | Formula-driven planning and review workbook`]];
  sectionHeader(sheet, `A3:${lastLetter}3`, `${contract.sheetName} — bounded input and future calculated outputs`);
  sheet.getRange('A4:D4').values = [['Used rows', null, 'Capacity', contract.capacity]];
  sheet.getRange('B4').formulas = [[`=COUNTA(A${contract.firstDataRow}:A${contract.lastDataRow})`]];
  const headers = Object.values(contract.columns).map(({ header }) => header);
  sheet.getRange(`A${contract.headerRow}:${lastLetter}${contract.headerRow}`).values = [headers];
  const table = sheet.tables.add(`A${contract.headerRow}:${lastLetter}${contract.headerRow}`, true, contract.tableName);
  table.rows.add(null, Array.from({ length: contract.capacity }, () => Array(headers.length).fill(null)));
  table.style = 'TableStyleMedium2';
  tableHeader(sheet, `A${contract.headerRow}:${lastLetter}${contract.headerRow}`);
  for (const key of contract.editableColumns) sheet.getRange(`${contract.columns[key].letter}${contract.firstDataRow}:${contract.columns[key].letter}${contract.lastDataRow}`).format.fill = COLORS.input;
  for (const key of contract.calculatedColumns) {
    const letter = contract.columns[key].letter;
    const first = sheet.getRange(`${letter}${contract.firstDataRow}`);
    first.formulas = [[key === 'active' ? `=IF($A${contract.firstDataRow}="","",TRUE)` : `=IF($A${contract.firstDataRow}="","","")`]];
    sheet.getRange(`${letter}${contract.firstDataRow}:${letter}${contract.lastDataRow}`).fillDown();
    sheet.getRange(`${letter}${contract.firstDataRow}:${letter}${contract.lastDataRow}`).format.fill = COLORS.grey;
  }
  sheet.freezePanes.freezeRows(contract.headerRow);
  sheet.getRange(`A:${lastLetter}`).format.columnWidth = 14;
  sheet.getRange('A:A').format.columnWidth = 24;
  return table;
}

function addRawValidations(sheet, contract) {
  const range = (key) => sheet.getRange(`${contract.columns[key].letter}${contract.firstDataRow}:${contract.columns[key].letter}${contract.lastDataRow}`);
  for (const key of ['md', 'centerNorth', 'centerEast', 'centerTvd', 'major', 'minor', 'verticalTolerance', 'mdIn', 'mdOut', 'slideLength', 'rotateLength', 'prognosedMd', 'prognosedTvd', 'actualPickMd']) if (contract.columns[key]) positive(range(key), true);
  for (const key of ['inc']) if (contract.columns[key]) between(range(key), 0, 180);
  for (const key of ['azi', 'rotation', 'commandedToolface']) if (contract.columns[key]) between(range(key), -360, 360);
  if (contract.columns.type) validation(range('type'), { type: 'list', values: ['Point', 'Circle', 'Ellipse', 'Box'] });
}

function seedTables(sheets) {
  sheets.Plan.getRange('A7:E66').values = directionalReferenceData.plan.map((r) => [r.station, r.mdFt, r.incDeg, r.aziDeg, 'Sanitized reference plan']);
  sheets.Plan.getRange('N7:N66').values = directionalReferenceData.plan.map((r) => [`plan-${String(r.station).padStart(3, '0')}`]);
  sheets.Survey.getRange('A7:E66').values = directionalReferenceData.survey.map((r) => [r.station, r.mdFt, r.incDeg, r.aziDeg, 'Sanitized reference survey']);
  sheets.Survey.getRange('Z7:Z66').values = directionalReferenceData.survey.map((r) => [`survey-${String(r.station).padStart(3, '0')}`]);
  const targetMds = [6125, 11125, 16125];
  sheets.Targets.getRange('A7:K9').values = directionalReferenceData.targets.map((r, index) => [r.id, targetMds[index], r.centerNorthFt, r.centerEastFt, r.centerTvdFt, 'Circle', r.radiusFt, r.radiusFt, 0, r.radiusFt, r.note]);
  sheets['Slide Performance'].getRange('A7:J12').values = directionalReferenceData.slideIntervals.map((r) => [r.stand, r.dateSerial, r.mdInFt, r.mdOutFt, r.slideFt, r.rotateFt, r.commandedToolfaceDeg, null, null, 'Sanitized reference fixture']);
  sheets['Slide Performance'].getRange('T7:T12').values = directionalReferenceData.slideIntervals.map((r) => [`slide-${String(r.stand).padStart(2, '0')}`]);
  sheets['Formation Tops'].getRange('A7:F10').values = directionalReferenceData.formationTops.map((r) => [r.name, r.prognosedMdFt, r.prognosedTvdFt, null, null, `Local dip ${r.localDipDeg} deg; sanitized reference fixture`]);
  sheets['Formation Tops'].getRange('L7:L10').values = directionalReferenceData.formationTops.map((r) => [r.id]);
}

function buildFormulaTrace(sheet) {
  const plan = directionalFormulaPlan(7);
  sheet.getRange('BU5:BU20').values = Object.keys(plan).map((key) => [key]);
  // This is an audit catalogue, not a live calculation block. Executing the
  // isolated fragments here points them at unrelated Calc cells and creates
  // misleading #VALUE!, #DIV/0!, and #NAME? errors.
  sheet.getRange('BV5:BV20').values = Object.values(plan).map((formula) => [`'${formula}`]);
  sheet.getRange('BU5:BU20').format.fill = COLORS.grey;
  sheet.getRange('BU:BU').format.columnWidth = 24;
  sheet.getRange('BV:BV').format.columnWidth = 80;
}

export function buildDirectionalWorkbook() {
  const { workbook, sheets } = createSuiteWorkbook('Directional Drilling Wellplan and Survey — SI', { sheetNames: DIRECTIONAL_SHEET_NAMES });
  buildInputs(sheets.Inputs);
  for (const contract of Object.values(DIRECTIONAL_TABLES)) {
    const sheet = sheets[contract.sheetName];
    addCapacityAndTable(sheet, contract);
    addRawValidations(sheet, contract);
  }
  seedTables(sheets);
  buildCanonicalModel(sheets);
  buildTrajectoryIdentitySurface(sheets.Calc);
  linkVisibleTables(sheets);
  buildTargetSlideFormation(sheets);
  buildDecisionSurfaces(sheets);
  sheets.Results.getRange('M25').values = [['Exchange record ID']];
  sheets.Results.getRange('M26:M525').formulas = Array.from({ length: DIRECTIONAL_TABLES.survey.capacity }, (_, index) => {
    const surveyRow = DIRECTIONAL_TABLES.survey.firstDataRow + index;
    return [`=IF(Survey!Z${surveyRow}="","",Survey!Z${surveyRow})`];
  });
  buildDirectionalCharts(sheets);
  buildFormulaTrace(sheets.Calc);
  addExchangeSheets(workbook, 'directional');
  applyTwoDecimalDisplayPrecision(workbook);
  return workbook;
}

import test from 'node:test';
import assert from 'node:assert/strict';
import {
  DIRECTIONAL_CAPACITIES,
  DIRECTIONAL_INPUT_CELLS,
  DIRECTIONAL_SHEET_NAMES,
  DIRECTIONAL_TABLES,
  directionalUnitRow,
} from '../src/directional_contract.mjs';
import * as formulas from '../src/directional_formulas.mjs';
import { buildDirectionalWorkbook, directionalFormulaPlan } from '../src/build_directional.mjs';
import { COLORS } from '../src/common.mjs';

const expectedSheets = ['Summary', 'Inputs', 'Plan', 'Survey', 'Targets', 'Slide Performance', 'Formation Tops', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', 'Exchange Map', 'Exchange State', 'Exchange Buffer'];
const expectedPlanKeys = ['doglegAngle', 'ratioFactor', 'deltaTvd', 'deltaNorth', 'deltaEast', 'doglegSeverity', 'slerpNorth', 'slerpEast', 'slerpVertical', 'partialPosition', 'crosslineError', 'error3d', 'effectiveTurn', 'responseToolface', 'targetEnvelope', 'formationHighLow'];

test('directional contracts publish stable topology, capacities, tables, cells, and unit rows', () => {
  assert.deepEqual(DIRECTIONAL_SHEET_NAMES, expectedSheets);
  assert.deepEqual(DIRECTIONAL_CAPACITIES, { plan: 500, survey: 500, targets: 100, slideIntervals: 200, formationTops: 100 });
  assert.deepEqual(Object.fromEntries(Object.entries(DIRECTIONAL_TABLES).map(([key, value]) => [key, value.tableName])), {
    plan: 'DirectionalPlanInput',
    survey: 'DirectionalSurveyInput',
    targets: 'DirectionalTargetsInput',
    slideIntervals: 'DirectionalSlidePerformanceInput',
    formationTops: 'DirectionalFormationTopsInput',
  });
  for (const [key, table] of Object.entries(DIRECTIONAL_TABLES)) {
    assert.equal(table.headerRow, 6, key);
    assert.equal(table.firstDataRow, 7, key);
    assert.equal(table.lastDataRow - table.firstDataRow + 1, DIRECTIONAL_CAPACITIES[key], key);
    assert.ok(Object.keys(table.columns).length >= 10, key);
  }
  assert.equal(DIRECTIONAL_INPUT_CELLS.metadata.wellName, 'B5');
  assert.equal(DIRECTIONAL_INPUT_CELLS.rawUnits.planLength, 'E5');
  assert.equal(DIRECTIONAL_INPUT_CELLS.dls.limit, 'H5');
  assert.equal(DIRECTIONAL_INPUT_CELLS.projection.bitMd, 'K5');
  assert.equal(DIRECTIONAL_INPUT_CELLS.controls.calibrationWindow, 'N8');
  assert.equal(directionalUnitRow('Length'), 8);
  assert.equal(directionalUnitRow('Angle'), 18);
  assert.equal(directionalUnitRow('Angular gradient'), 20);
});

test('formula plan uses the production row factories and exposes every required guard and sign convention', () => {
  const plan = directionalFormulaPlan(7);
  assert.deepEqual(Object.keys(plan), expectedPlanKeys);
  const expected = {
    doglegAngle: formulas.doglegAngleFormula(7),
    ratioFactor: formulas.ratioFactorFormula(7),
    deltaTvd: formulas.deltaTvdFormula(7),
    deltaNorth: formulas.deltaNorthFormula(7),
    deltaEast: formulas.deltaEastFormula(7),
    doglegSeverity: formulas.doglegSeverityFormula(7),
    slerpNorth: formulas.slerpNorthFormula(7),
    slerpEast: formulas.slerpEastFormula(7),
    slerpVertical: formulas.slerpVerticalFormula(7),
    partialPosition: formulas.partialPositionFormula(7),
    crosslineError: formulas.crosslineErrorFormula(7),
    error3d: formulas.error3dFormula(7),
    effectiveTurn: formulas.effectiveTurnFormula(7),
    responseToolface: formulas.responseToolfaceFormula(7),
    targetEnvelope: formulas.targetEnvelopeFormula(7),
    formationHighLow: formulas.formationHighLowFormula(7),
  };
  assert.deepEqual(plan, expected);
  assert.match(plan.doglegAngle, /ACOS\(MAX\(-1,MIN\(1,/);
  assert.match(plan.doglegAngle, /MOD\(.+\+PI\(\),2\*PI\(\)\)-PI\(\)/);
  assert.match(plan.ratioFactor, /1\+G7\^2\/12\+G7\^4\/120/);
  assert.match(plan.ratioFactor, /2\*TAN\(G7\/2\)\/G7/);
  assert.match(plan.deltaTvd, /COS\(D6\).*COS\(D7\)/);
  assert.match(plan.deltaNorth, /SIN\(D6\)\*COS\(E6\).*SIN\(D7\)\*COS\(E7\)/);
  assert.match(plan.deltaEast, /SIN\(D6\)\*SIN\(E6\).*SIN\(D7\)\*SIN\(E7\)/);
  assert.match(plan.doglegSeverity, /IF\(F7>0,G7\/F7/);
  for (const key of ['slerpNorth', 'slerpEast', 'slerpVertical']) {
    assert.match(plan[key], /IF\(ABS\('Calc'!\$G7\)<1E-9/);
    assert.match(plan[key], /SQRT\(/);
    assert.match(plan[key], /SIN\(/);
  }
  assert.match(plan.partialPosition, /'Calc'!/);
  assert.match(plan.crosslineError, /-.*SIN\('Inputs'!\$B\$16\).*COS\('Inputs'!\$B\$16\)/);
  assert.match(plan.error3d, /SQRT\(.+\^2\+.+\^2\+.+\^2\)/);
  assert.match(plan.effectiveTurn, /MOD\(.+\+PI\(\),2\*PI\(\)\)-PI\(\)/);
  assert.match(plan.effectiveTurn, /SIN\(\(D6\+D7\)\/2\)/);
  assert.match(plan.responseToolface, /MOD\(ATAN2\('Calc'!\$P7,'Calc'!\$Q7\),2\*PI\(\)\)/);
  assert.match(plan.targetEnvelope, /localMajor/);
  assert.match(plan.targetEnvelope, /localMinor/);
  assert.match(plan.targetEnvelope, /"Point"/);
  assert.match(plan.targetEnvelope, /"Circle"/);
  assert.match(plan.targetEnvelope, /"Ellipse"/);
  assert.match(plan.targetEnvelope, /"Box"/);
  assert.match(plan.formationHighLow, /'Formation Tops'!\$C7-'Formation Tops'!\$G7/);
});

test('directional workbook creates exact topology and five bounded native input tables', () => {
  const workbook = buildDirectionalWorkbook();
  assert.deepEqual(workbook.worksheets.items.map((sheet) => sheet.name), expectedSheets);
  const names = [];
  for (const [key, contract] of Object.entries(DIRECTIONAL_TABLES)) {
    const sheet = workbook.worksheets.getItem(contract.sheetName);
    assert.equal(sheet.tables.items.length, 1, key);
    const table = sheet.tables.items[0];
    names.push(table.name);
    assert.equal(table.name, contract.tableName);
    assert.deepEqual(table.getHeaderRowRange().values[0], Object.values(contract.columns).map((column) => column.header));
    const lastCalculated = contract.columns[contract.calculatedColumns.at(-1)].letter;
    assert.match(sheet.getRange(`${lastCalculated}${contract.lastDataRow}`).formulas[0][0], /^=IF\(/);
    assert.equal(sheet.getRange(`A${contract.lastDataRow}`).values[0][0], null);
  }
  assert.equal(new Set(names).size, 5);
});

test('raw table inputs are blue, future calculated columns are grey, and headers freeze', async () => {
  const workbook = buildDirectionalWorkbook();
  for (const contract of Object.values(DIRECTIONAL_TABLES)) {
    const sheet = workbook.worksheets.getItem(contract.sheetName);
    assert.equal(sheet.showGridLines, false);
    const lastLetter = Object.values(contract.columns).at(-1).letter;
    const inspected = await workbook.inspect({ kind: 'computedStyle', sheetId: contract.sheetName, range: `A${contract.firstDataRow}:${lastLetter}${contract.firstDataRow}`, maxChars: 20000 });
    const styles = Object.fromEntries(inspected.ndjson.split('\n').filter(Boolean).map((line) => JSON.parse(line)).map((entry) => [entry.for.replace(/\d+$/, ''), `#${entry.style.fill.color.value}`]));
    const seededEditable = contract.editableColumns[0];
    assert.equal(styles[contract.columns[seededEditable].letter], COLORS.input, `${contract.sheetName}.${seededEditable}`);
    for (const key of contract.calculatedColumns) assert.equal(styles[contract.columns[key].letter], COLORS.grey, `${contract.sheetName}.${key}`);
  }
});

test('Inputs exposes validated raw units, operating controls, and display-only explanation', () => {
  const workbook = buildDirectionalWorkbook();
  const inputs = workbook.worksheets.getItem('Inputs');
  const raw = DIRECTIONAL_INPUT_CELLS.rawUnits;
  for (const cell of [raw.planLength, raw.surveyLength, raw.targetLength, raw.slideLength, raw.formationLength]) {
    assert.deepEqual(inputs.getRange(cell).dataValidation.rule.values, ['m', 'ft']);
  }
  for (const cell of [raw.planAngle, raw.surveyAngle]) assert.deepEqual(inputs.getRange(cell).dataValidation.rule.values, ['deg', 'rad']);
  assert.deepEqual(inputs.getRange(DIRECTIONAL_INPUT_CELLS.dls.unit).dataValidation.rule.values, ['rad/m', 'deg/100ft', 'deg/30m']);
  assert.match(inputs.getRange('D14:N14').values[0].join(' '), /Unit Map!B5.*display-only/i);
  assert.equal(inputs.getRange(DIRECTIONAL_INPUT_CELLS.dls.limit).dataValidation.rule.operator, 'greaterThan');
  assert.equal(inputs.getRange(DIRECTIONAL_INPUT_CELLS.projection.aheadMd).dataValidation.rule.operator, 'greaterThanOrEqual');
  assert.equal(inputs.getRange(DIRECTIONAL_INPUT_CELLS.controls.calibrationWindow).dataValidation.rule.type, 'whole');
});

test('target types and editable engineering ranges carry validation', () => {
  const workbook = buildDirectionalWorkbook();
  const target = DIRECTIONAL_TABLES.targets;
  const targets = workbook.worksheets.getItem(target.sheetName);
  assert.deepEqual(targets.getRange(`${target.columns.type.letter}${target.firstDataRow}`).dataValidation.rule.values, ['Point', 'Circle', 'Ellipse', 'Box']);
  assert.equal(targets.getRange(`${target.columns.major.letter}${target.firstDataRow}`).dataValidation.rule.operator, 'greaterThanOrEqual');
  const plan = DIRECTIONAL_TABLES.plan;
  const planSheet = workbook.worksheets.getItem(plan.sheetName);
  assert.equal(planSheet.getRange(`${plan.columns.md.letter}${plan.firstDataRow}`).dataValidation.rule.operator, 'greaterThanOrEqual');
  assert.equal(planSheet.getRange(`${plan.columns.inc.letter}${plan.firstDataRow}`).dataValidation.rule.operator, 'between');
});

test('sanitized reference rows seed exactly and all unused capacity remains blank', () => {
  const workbook = buildDirectionalWorkbook();
  const plan = workbook.worksheets.getItem('Plan');
  assert.deepEqual(plan.getRange('A7:E7').values[0].slice(0, 4), [0, 0, 0, 20]);
  assert.deepEqual(plan.getRange('A66:E66').values[0].slice(0, 4), [59, 16125, 90, 75]);
  assert.deepEqual(plan.getRange('A67:E67').values[0], [null, null, null, null, null]);
  const survey = workbook.worksheets.getItem('Survey');
  assert.deepEqual(survey.getRange('A7:E7').values[0].slice(0, 4), [0, 0, 0, 20]);
  assert.deepEqual(survey.getRange('A66:E66').values[0].slice(0, 4), [59, 16140.816, 88.8, 89]);
  assert.deepEqual(survey.getRange('A67:E67').values[0], [null, null, null, null, null]);
  const targets = workbook.worksheets.getItem('Targets');
  assert.deepEqual(targets.getRange('B7:B9').values.flat(), [6125, 11125, 16125]);
  assert.deepEqual(targets.getRange('F7:F9').values.flat(), ['Circle', 'Circle', 'Circle']);
  const slides = workbook.worksheets.getItem('Slide Performance');
  assert.deepEqual(slides.getRange('A7:G12').values.map((row) => row.slice(0, 7)), [
    [1, 46255, 5000, 5090, 88, 2, 0], [2, 46256, 5090, 5180, 87, 3, 0], [3, 46257, 5180, 5270, 86, 4, 0],
    [4, 46258, 5270, 5360, 85, 5, 0], [5, 46259, 5360, 5450, 84, 6, 0], [6, 46260, 5450, 5540, 83, 7, 0],
  ]);
  assert.deepEqual(slides.getRange('A13:J13').values[0], Array(10).fill(null));
  const tops = workbook.worksheets.getItem('Formation Tops');
  assert.deepEqual(tops.getRange('A7:D10').values.map((row) => row.slice(0, 4)), [
    ['Top Niobrara A', 4600, 4550, null], ['Top Niobrara B', 5100, 5040, null],
    ['Top Niobrara C', 5350, 5280, null], ['Top Codell / Landing Zone', 5600, 5716, null],
  ]);
});

test('capacity labels, representative Calc formulas, method limits, and safe text are visible', () => {
  const workbook = buildDirectionalWorkbook();
  for (const contract of Object.values(DIRECTIONAL_TABLES)) {
    const sheet = workbook.worksheets.getItem(contract.sheetName);
    const capacityRow = sheet.getRange('A4:D4').values[0];
    assert.equal(capacityRow[0], 'Used rows');
    assert.equal(capacityRow[2], 'Capacity');
    assert.equal(capacityRow[3], contract.capacity);
    assert.match(sheet.getRange('B4').formulas[0][0], /COUNTA/);
  }
  const calc = workbook.worksheets.getItem('Calc');
  const plan = directionalFormulaPlan(7);
  assert.deepEqual(calc.getRange('BU5:BU20').values.flat(), expectedPlanKeys);
  assert.deepEqual(calc.getRange('BV5:BV20').formulas.flat(), Array(16).fill(''));
  assert.deepEqual(calc.getRange('BV5:BV20').values.flat(), Object.values(plan));
  const text = workbook.worksheets.items.flatMap((sheet) => sheet.getUsedRange()?.values?.flat(2) ?? []).filter((value) => typeof value === 'string').join('\n');
  for (const phrase of ['minimum curvature', 'deterministic projection', 'no ISCWSA covariance', 'no anti-collision', 'no pipe-fatigue', 'no VBA', 'no external links', 'planning/review only']) assert.match(text, new RegExp(phrase, 'i'));
  assert.doesNotMatch(text, /Users[\\/]|home[\\/]|AppData|PERSONAL\.XLSB|macro instructions?/i);
});

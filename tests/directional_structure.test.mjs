import test from 'node:test';
import assert from 'node:assert/strict';
import { createSuiteWorkbook } from '../src/workbook.mjs';
import { UNIT_SYSTEMS, CUSTOM_UNIT_SYSTEMS, UNIT_ROWS } from '../src/common.mjs';

const exchangeNames = ['Exchange Map', 'Exchange State', 'Exchange Buffer'];
const defaultNames = ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...exchangeNames];
const directionalNames = ['Summary', 'Inputs', 'Plan', 'Survey', 'Targets', 'Slide Performance', 'Formation Tops', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...exchangeNames];

test('default workbook topology preserves the established order and appends exchange sheets', () => {
  const { workbook, sheets } = createSuiteWorkbook('Topology regression');
  assert.deepEqual(workbook.worksheets.items.map((sheet) => sheet.name), defaultNames);
  assert.deepEqual(Object.keys(sheets), defaultNames);
});

test('custom directional topology preserves the supplied exact order', () => {
  const { workbook, sheets } = createSuiteWorkbook('Directional topology', { sheetNames: directionalNames });
  assert.deepEqual(workbook.worksheets.items.map((sheet) => sheet.name), directionalNames);
  assert.deepEqual(Object.keys(sheets), directionalNames);
});

test('duplicate sheet names are rejected before workbook sheets are created', () => {
  assert.throws(
    () => createSuiteWorkbook('Duplicate topology', { sheetNames: [...directionalNames, 'Calc'] }),
    /duplicate sheet name/i,
  );
});

test('Unit Map writes Angular gradient factors and selector formulas to workbook ranges', () => {
  const { sheets } = createSuiteWorkbook('Directional units');
  const rowNumber = 8 + UNIT_ROWS.findIndex((row) => row.domain === 'Angular gradient');
  const unitMap = sheets['Unit Map'];
  assert.deepEqual(unitMap.getRange(`A${rowNumber}:G${rowNumber}`).values[0], [
    'Angular gradient', 'rad/m', 'deg/100ft', 'deg/30m', 1746.37535955875, 1718.87338539247, 0,
  ]);
  assert.equal(
    unitMap.getRange(`H${rowNumber}`).formulas[0][0],
    `=IF($B$5="SI",B${rowNumber},IF($B$5="Imperial",C${rowNumber},IF($B$5="Mixed",D${rowNumber},IF($B$5="Custom",IF(J${rowNumber}="SI",B${rowNumber},IF(J${rowNumber}="Imperial",C${rowNumber},IF(J${rowNumber}="Mixed",D${rowNumber},"INVALID"))),"INVALID"))))`,
  );
  assert.equal(
    unitMap.getRange(`I${rowNumber}`).formulas[0][0],
    `=IF($B$5="SI",1,IF($B$5="Imperial",E${rowNumber},IF($B$5="Mixed",F${rowNumber},IF($B$5="Custom",IF(J${rowNumber}="SI",1,IF(J${rowNumber}="Imperial",E${rowNumber},IF(J${rowNumber}="Mixed",F${rowNumber},NA()))),NA()))))`,
  );
  assert.deepEqual(unitMap.getRange(`J${rowNumber}`).dataValidation.rule.values, CUSTOM_UNIT_SYSTEMS);
});

test('Unit Map selector validation is driven by UNIT_SYSTEMS', () => {
  UNIT_SYSTEMS.push('Contract test system');
  try {
    const { sheets } = createSuiteWorkbook('Selector validation');
    assert.deepEqual(sheets['Unit Map'].getRange('B5').dataValidation.rule.values, UNIT_SYSTEMS);
  } finally {
    UNIT_SYSTEMS.pop();
  }
});

test('Unit Map invalid selector formulas explicitly reject tampered values', () => {
  const { sheets } = createSuiteWorkbook('Invalid selector');
  const unitMap = sheets['Unit Map'];
  const rowNumber = 8 + UNIT_ROWS.findIndex((row) => row.domain === 'Stress');
  unitMap.getRange('B5').values = [['Unsupported system']];
  assert.match(unitMap.getRange(`H${rowNumber}`).formulas[0][0], /"INVALID"/);
  assert.match(unitMap.getRange(`I${rowNumber}`).formulas[0][0], /NA\(\)/);
  assert.doesNotMatch(unitMap.getRange(`H${rowNumber}`).formulas[0][0], /,D\d+\)\)$/);
});

test('Unit Map lets Custom select a distinct display convention per engineering domain', () => {
  const { sheets } = createSuiteWorkbook('Custom units');
  const unitMap = sheets['Unit Map'];
  unitMap.getRange('B5').values = [['Custom']];
  unitMap.getRange('J8').values = [['Imperial']];
  unitMap.getRange('J15').values = [['Mixed']];
  assert.match(unitMap.getRange('H8').formulas[0][0], /\$B\$5="Custom"/);
  assert.match(unitMap.getRange('H8').formulas[0][0], /J8="Imperial"/);
  assert.match(unitMap.getRange('I15').formulas[0][0], /J15="Mixed"/);
});

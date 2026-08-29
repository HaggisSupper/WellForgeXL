import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';
import { buildDirectionalWorkbook } from '../src/build_directional.mjs';
import { directionalSourceAudit } from './helpers/directional_source_audit.mjs';

let workbook;
let tempDir;

before(async () => {
  tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'wellforge-directional-test-'));
  const output = await SpreadsheetFile.exportXlsx(buildDirectionalWorkbook());
  const file = path.join(tempDir, 'directional.xlsx');
  await output.save(file);
  workbook = await SpreadsheetFile.importXlsx(await FileBlob.load(file));
});

function close(actual, expected, tolerance, label) {
  assert.ok(Number.isFinite(actual), `${label}: expected numeric value, got ${actual}`);
  assert.ok(Math.abs(actual - expected) <= tolerance, `${label}: ${actual} vs ${expected}`);
}

test('exported Calc canonical plan and survey reproduce immutable SI snapshots', () => {
  const calc = workbook.worksheets.getItem('Calc');
  assert.deepEqual(calc.getRange('A6:R6').values[0], ['Active', 'Station ID', 'MD m', 'Inc rad', 'Azi rad', 'dMD m', 'Dogleg rad', 'RF', 'dTVD m', 'dN m', 'dE m', 'TVD m', 'North m', 'East m', 'VS State', 'Crossline State', 'DLS rad/m', 'Row Status']);
  assert.deepEqual(calc.getRange('T6:AK6').values[0], ['Active', 'Station ID', 'MD m', 'Inc rad', 'Azi rad', 'dMD m', 'Dogleg rad', 'RF', 'dTVD m', 'dN m', 'dE m', 'TVD m', 'North m', 'East m', 'VS State', 'Crossline State', 'DLS rad/m', 'Row Status']);
  for (const [label, startColumn, snapshot] of [['plan', 'C', directionalSourceAudit.plan], ['survey', 'V', directionalSourceAudit.survey]]) {
    for (const index of [0, 11, 19, 39, 59]) {
      const row = 7 + index;
      const md = calc.getRange(`${startColumn}${row}`).values[0][0];
      const geometryStart = label === 'plan' ? 'L' : 'AE';
      const geometry = calc.getRange(`${geometryStart}${row}:${label === 'plan' ? 'Q' : 'AJ'}${row}`).values[0];
      close(md, snapshot[index].source.mdFt / 3.280839895, 1e-7, `${label} row ${row} MD`);
      close(geometry[0], snapshot[index].tvdM, 1e-6, `${label} row ${row} TVD`);
      close(geometry[1], snapshot[index].northM, 1e-6, `${label} row ${row} North`);
      close(geometry[2], snapshot[index].eastM, 1e-6, `${label} row ${row} East`);
      close(geometry[5], snapshot[index].dlsRadPerM, 1e-9, `${label} row ${row} DLS`);
    }
  }
});

test('visible plan and survey outputs are formula-linked through Unit Map display factors', () => {
  for (const [sheetName, range] of [['Plan', 'G7:L7'], ['Survey', 'G7:X7']]) {
    const formulas = workbook.worksheets.getItem(sheetName).getRange(range).formulas.flat().filter(Boolean);
    assert.ok(formulas.some((formula) => formula.includes("'Unit Map'!$I$8")), `${sheetName} lacks display-length conversion`);
    assert.ok(formulas.some((formula) => formula.includes("'Unit Map'!$I$20")), `${sheetName} lacks display-gradient conversion`);
  }
  assert.doesNotMatch(workbook.worksheets.getItem('Calc').getRange('A7:AK506').formulas.flat().join('\n'), /'Unit Map'!\$B\$5|'Unit Map'!\$I\$8/);
});

test('plan-at-survey interpolation exposes exact coverage states and terminal endpoint comparison', () => {
  const calc = workbook.worksheets.getItem('Calc');
  assert.equal(calc.getRange('AL7').values[0][0], 'OK');
  assert.equal(calc.getRange('AL66').values[0][0], 'BEYOND TD');
  const results = workbook.worksheets.getItem('Results');
  assert.equal(results.getRange('A18').values[0][0], 'Terminal endpoint comparison');
  close(results.getRange('B19').values[0][0], 429.3047010992369, 0.001, 'terminal crossline');
  close(results.getRange('B20').values[0][0], 434.10904709911625, 0.001, 'terminal horizontal');
  close(results.getRange('B21').values[0][0], 437.80938810431627, 0.001, 'terminal 3D');
});

test('decision surface, checks, targets, slide and formation outputs remain formula driven', () => {
  const summary = workbook.worksheets.getItem('Summary');
  assert.match(summary.getRange('B5').formulas[0][0], /Checks/);
  assert.match(summary.getRange('B5').values[0][0], /^(READY|CAUTION|STOP)$/);
  for (const address of ['B6', 'B7', 'B8', 'B9', 'B10']) assert.ok(summary.getRange(address).formulas[0][0], address);
  const checks = workbook.worksheets.getItem('Checks');
  assert.ok(checks.getRange('A6:E25').formulas.flat().filter(Boolean).length >= 20);
  assert.match(checks.getRange('A6:A25').values.flat().join('\n'), /DLS|coverage|ISCWSA|fatigue|external/i);
  assert.ok(workbook.worksheets.getItem('Targets').getRange('L7:Q9').formulas.flat().every(Boolean));
  assert.ok(workbook.worksheets.getItem('Slide Performance').getRange('K7:S12').formulas.flat().every(Boolean));
  assert.equal(workbook.worksheets.getItem('Formation Tops').getRange('G7').values[0][0], null);
});

test('exported canonical and decision ranges contain no formula errors', () => {
  const ranges = [
    ['Calc', 'A7:BS506'], ['Calc', 'GM6:GM45'], ['Plan', 'F7:M506'], ['Survey', 'F7:Y506'],
    ['Targets', 'L7:Q106'], ['Slide Performance', 'K7:S206'], ['Formation Tops', 'G7:K106'],
    ['Summary', 'B5:B10'], ['Results', 'B6:B21'], ['Checks', 'B6:D25'],
  ];
  const errors = [];
  for (const [sheetName, address] of ranges) {
    const values = workbook.worksheets.getItem(sheetName).getRange(address).values;
    for (const row of values) for (const value of row) if (typeof value === 'string' && /^#(?:REF!|DIV\/0!|VALUE!|NAME\?|N\/A)$/.test(value)) errors.push(`${sheetName}!${address}: ${value}`);
  }
  assert.deepEqual(errors, []);
});

test('deterministic projection is finite and confidence-labelled', () => {
  const calc = workbook.worksheets.getItem('Calc');
  assert.equal(calc.getRange('GM45').values[0][0], 'DETERMINISTIC');
  assert.equal(calc.getRange('GM44').values[0][0], 'NORMAL');
  for (const address of ['GM31', 'GM41', 'GM42', 'GM43']) assert.ok(Number.isFinite(calc.getRange(address).values[0][0]), address);
});

test('Results publishes a bounded formula-driven 500-row Survey Contract', () => {
  const results = workbook.worksheets.getItem('Results');
  assert.deepEqual(results.getRange('A25:L25').values[0], ['Station_ID', 'MD_m', 'Inc_rad', 'Azi_rad', 'TVD_m', 'North_m', 'East_m', 'VS_State', 'Crossline_State', 'DLS_rad_per_m', 'Source', 'Row_Status']);
  assert.ok(results.getRange('A26:L26').formulas.flat().every(Boolean));
  assert.ok(results.getRange('A525:L525').formulas.flat().every(Boolean));
  assert.equal(results.getRange('A525').values[0][0], null);
});

test('Graphs contains at least eight native formula-backed charts', () => {
  const graphs = workbook.worksheets.getItem('Graphs');
  assert.ok(graphs.charts.items.length >= 8, `found ${graphs.charts.items.length} charts`);
  const expectedSeriesColumns = [
    { chartIndex: 0, x: ['DA', 'DC'], y: ['DB', 'DD'] },
    { chartIndex: 1, x: ['DE', 'DG'], y: ['DF', 'DH'] },
  ];
  for (const { chartIndex, x: expectedX, y: expectedY } of expectedSeriesColumns) {
    const series = graphs.charts.items[chartIndex].series.items;
    assert.equal(series.length, 2);
    for (let index = 0; index < series.length; index += 1) {
      assert.match(series[index].xFormula, new RegExp(`Calc.*\\$${expectedX[index]}\\$`), `chart ${chartIndex + 1} series ${index + 1} must use true X values`);
      assert.match(series[index].formula, new RegExp(`Calc.*\\$${expectedY[index]}\\$`), `chart ${chartIndex + 1} series ${index + 1} must use display-unit Y values`);
    }
  }
  const titles = graphs.charts.items.map((chart) => typeof chart.title === 'string' ? chart.title : chart.title?.text ?? '').join('\n');
  for (const phrase of ['Plan View', 'Vertical Section', 'Inclination', 'DLS', 'Signed', '3D Error', 'Slide Yield', 'Target']) assert.match(titles, new RegExp(phrase, 'i'));
  assert.equal(workbook.worksheets.items.filter((sheet) => sheet.name !== 'Graphs').some((sheet) => sheet.charts.items.length > 0), false);
});

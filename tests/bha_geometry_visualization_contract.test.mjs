import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import JSZip from 'jszip';

test('BHA workbook exposes an indicative hole/OD/ID projection and screening charts', async () => {
  const { buildBhaWorkbook } = await import('../src/build_bha.mjs');
  const workbook = buildBhaWorkbook();
  const inputs = workbook.worksheets.getItem('Inputs');
  const geometry = workbook.worksheets.getItem('BHA Geometry View');
  const calc = workbook.worksheets.getItem('Calc');
  const modes = workbook.worksheets.getItem('Vibration Modes');

  assert.equal(inputs.getRange('A11').values[0][0], 'Hole diameter m');
  assert.equal(inputs.getRange('A12').values[0][0], 'Projection plane');
  assert.equal(inputs.getRange('A13').values[0][0], 'Deflection display scale');
  assert.deepEqual(inputs.getRange('B12').dataValidation.rule.values, ['Highside', 'Lowside']);

  const note = geometry.getRange('A5:N7').values.flat().join(' ');
  assert.match(note, /geometric interference indication/i);
  assert.match(note, /not solved contact or reaction force/i);

  assert.deepEqual(calc.getRange('P5:AD5').values[0], [
    'Distance from bit SI', 'Distance from bit', 'Component', 'Local fraction',
    'Estimated centreline SI', 'Projected centreline', 'Hole high wall', 'Hole low wall',
    'BHA OD high', 'BHA OD low', 'BHA ID high', 'BHA ID low',
    'Projected radial clearance', 'Zero clearance', 'Geometry flag',
  ]);
  assert.match(calc.getRange('T6').formulas[0][0], /SIN\(PI\(\)\*S6\)/);
  assert.match(calc.getRange('U6').formulas[0][0], /'Unit Map'!\$I\$9/);
  assert.match(calc.getRange('V6').formulas[0][0], /Inputs!\$B\$11\/2/);
  assert.match(calc.getRange('X6').formulas[0][0], /Inputs!F6\/2/);
  assert.match(calc.getRange('Z6').formulas[0][0], /Inputs!G6\/2/);
  assert.match(calc.getRange('AB6').formulas[0][0], /Inputs!\$B\$11\/2-ABS\(T6\)-Inputs!F6\/2/);

  const geometryTitles = geometry.charts.items.map((chart) => chart.title?.text ?? chart.title);
  assert.ok(geometryTitles.some((title) => /BHA geometry projection/i.test(title)));
  assert.ok(geometryTitles.some((title) => /Projected radial clearance/i.test(title)));
  assert.ok(geometryTitles.some((title) => /Bending moment versus distance/i.test(title)));
  assert.ok(geometryTitles.some((title) => /Bending stress versus distance/i.test(title)));

  const modeTitles = modes.charts.items.map((chart) => chart.title?.text ?? chart.title);
  assert.ok(modeTitles.some((title) => /Screening Campbell diagram/i.test(title)));
  assert.ok(modeTitles.some((title) => /Component modal frequencies/i.test(title)));
});

test('BHA projection and vibration screening traces export as connected XY series', async () => {
  const { buildBhaWorkbook } = await import('../src/build_bha.mjs');
  const { exportExchangeXlsx } = await import('../src/exchange/export_exchange_xlsx.mjs');
  const zip = await JSZip.loadAsync((await exportExchangeXlsx(buildBhaWorkbook())).data);
  const chartPaths = Object.keys(zip.files).filter((name) => /^xl\/drawings\/charts\/chart\d+\.xml$/.test(name));
  const charts = await Promise.all(chartPaths.map((name) => zip.file(name).async('string')));
  for (const title of ['BHA geometry projection', 'Projected radial clearance', 'Screening Campbell diagram', 'Component modal frequencies']) {
    const chart = charts.find((xml) => xml.includes(title));
    assert.ok(chart, title);
    assert.match(chart, /<c:scatterStyle val="lineMarker"\s*\/>/, `${title} must use connected traces`);
  }
});

test('BHA VBA engine writes projection arrays and labels indication limits explicitly', async () => {
  const source = await readFile(new URL('../VBA/WellForgeBha.bas', import.meta.url), 'utf8');
  const core = await readFile(new URL('../VBA/WellForgeCore.bas', import.meta.url), 'utf8');
  assert.match(source, /Worksheets\("BHA Geometry View"\)/);
  assert.match(source, /geometryData\(1 To 30, 1 To 15\)/);
  assert.match(source, /Sin\(BHA_PI \* localFraction\)/);
  assert.match(source, /wsCalc\.Range\("P6:AD35"\)\.Value2 = geometryData/);
  assert.match(source, /geometric interference indication/i);
  assert.match(source, /not solved contact or reaction force/i);
  assert.match(source, /WF_UnitFactor\("Diameter"\)/);
  assert.doesNotMatch(source, /contactForce\s*=/i);
  assert.match(core, /Case "BHA"[\s\S]*?WF_AssertDashboard "BHA Geometry View", 4/);
  assert.match(core, /WF_AssertSeriesCount "BHA Geometry View", 1, 7/);
  assert.match(core, /WF_AssertSeriesCount "BHA Geometry View", 2, 2/);
  assert.match(core, /not solved contact or reaction force/i);
});

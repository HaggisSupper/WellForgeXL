import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { buildDirectionalWorkbook } from '../src/build_directional.mjs';

const read = (relative) => fs.readFile(new URL(`../${relative}`, import.meta.url), 'utf8');

test('in-memory directional model exposes explicit editable trajectory provenance', () => {
  const workbook = buildDirectionalWorkbook();
  const inputs = workbook.worksheets.getItem('Inputs');
  assert.equal(inputs.getRange('P3').values[0][0], 'Rust trajectory provenance — explicit authoritative identities');
  assert.deepEqual(inputs.getRange('P5:V5').values[0], ['Role', 'UUID', 'URI', 'Object type', 'Content hash', 'Citation name', 'Source system']);
  assert.deepEqual(inputs.getRange('P6:P9').values.flat(), ['Well', 'Wellbore', 'Plan trajectory', 'Survey trajectory']);
  assert.deepEqual(inputs.getRange('P12:P17').values.flat(), ['Analysis UUID', 'MD datum UUID', 'MD datum name', 'MD datum kind', 'Azimuth reference', 'Contract version']);
  for (const uuid of [...inputs.getRange('Q6:Q9').values.flat(), ...inputs.getRange('Q12:Q13').values.flat()]) {
    assert.match(uuid, /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  }
  assert.equal(inputs.getRange('Q15').values[0][0], 'rotary_kelly_bushing');
  assert.equal(inputs.getRange('Q16').values[0][0], 'grid_north');
  assert.equal(inputs.getRange('Q17').values[0][0], '1.0.0');
  assert.equal(inputs.getRange('B12').values[0][0], 'Local grid coordinates; surface origin in B13:B14');
  assert.ok(inputs.getRange('Q6:V9').values.flat().every((value) => value !== null && value !== ''));
});

test('in-memory directional model stores unique UUIDs for every bounded Rust request row', () => {
  const workbook = buildDirectionalWorkbook();
  const calc = workbook.worksheets.getItem('Calc');
  assert.deepEqual(calc.getRange('JA6:JE6').values[0], ['Plan UUID', 'Survey UUID', 'Target UUID', 'Slide UUID', 'Formation UUID']);
  const ranges = [['JA7:JA506', 500], ['JB7:JB506', 500], ['JC7:JC106', 100], ['JD7:JD206', 200], ['JE7:JE106', 100]];
  const all = [];
  for (const [address, count] of ranges) {
    const ids = calc.getRange(address).values.flat();
    assert.equal(ids.length, count);
    assert.ok(ids.every((id) => /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(id)), address);
    all.push(...ids);
  }
  assert.equal(new Set(all).size, all.length);
});

test('in-memory canonical model marks VBA rotations as presentation-only state', () => {
  const workbook = buildDirectionalWorkbook();
  const calc = workbook.worksheets.getItem('Calc');
  const results = workbook.worksheets.getItem('Results');
  assert.deepEqual(calc.getRange('O6:P6').values[0], ['VS State', 'Crossline State']);
  assert.deepEqual(calc.getRange('AH6:AI6').values[0], ['VS State', 'Crossline State']);
  assert.deepEqual(calc.getRange('BJ6:BK6').values[0], ['Plan-at-MD VS State', 'Plan-at-MD Crossline State']);
  assert.deepEqual(results.getRange('H25:I25').values[0], ['VS_State', 'Crossline_State']);
  for (const address of ['O7:P7', 'AH7:AI7', 'BJ7:BK7', 'H26:I26']) {
    assert.deepEqual(workbook.worksheets.getItem(address === 'H26:I26' ? 'Results' : 'Calc').getRange(address).values[0],
      ['PRESENTATION_ONLY_NOT_RUST_RESULT', 'PRESENTATION_ONLY_NOT_RUST_RESULT']);
  }
  assert.equal(calc.getRange('BO7').values[0][0], 'NOT_RUN_RUST_REQUIRED');
  assert.ok(Number.isFinite(workbook.worksheets.getItem('Survey').getRange('R7').values[0][0]));
  assert.ok(Number.isFinite(calc.getRange('DS7').values[0][0]));
});

test('in-memory target helper declares exact Rust fields and typed unavailable legacy slots', () => {
  const workbook = buildDirectionalWorkbook();
  const calc = workbook.worksheets.getItem('Calc');
  assert.deepEqual(calc.getRange('HB6:HT6').values[0], [
    'Target UUID', 'Target MD m', 'Basis', 'Inc State', 'Azi State', 'Dogleg State', 'RF State', 'Position TVD m',
    'Position North m (surface-relative Rust)', 'Position East m (surface-relative Rust)', 'North Difference State',
    'East Difference State', 'Vertical Difference m (Rust)', 'Local Major m', 'Local Minor m',
    'Horizontal Utilization', 'Vertical Utilization', 'Rust Evaluation Status', 'Display Status',
  ]);
  const row = calc.getRange('HB7:HT7').values[0];
  for (const index of [3, 4, 5, 6, 10, 11]) assert.equal(row[index], 'UNAVAILABLE_NOT_IN_RUST_BRIDGE');
  for (const index of [2, 7, 8, 9, 12, 13, 14, 15, 16, 17, 18]) assert.equal(row[index], 'NOT_RUN_RUST_REQUIRED');
});

test('in-memory directional model publishes value-backed Rust engine evidence surfaces', () => {
  const workbook = buildDirectionalWorkbook();
  const results = workbook.worksheets.getItem('Results');
  const checks = workbook.worksheets.getItem('Checks');
  assert.equal(results.getRange('O3').values[0][0], 'Rust trajectory engine evidence');
  assert.deepEqual(results.getRange('O5:O14').values.flat(), ['Execution mode', 'State', 'Request path', 'Result path', 'Diagnostic path', 'Request hash', 'Result hash', 'Engine version', 'Executable SHA-256', 'Accepted UTC']);
  assert.equal(results.getRange('P5').values[0][0], 'RUST REQUIRED — NO VBA FALLBACK');
  assert.equal(results.getRange('P6').values[0][0], 'NOT RUN');
  assert.ok(results.getRange('P5:P14').formulas.flat().every((formula) => !formula));
  assert.equal(checks.getRange('A20').values[0][0], 'Rust executable / hash verification');
  assert.deepEqual(checks.getRange('B20:D20').formulas[0], ['="NOT RUN"', '="INFO"', '="INFO"']);
  assert.deepEqual(checks.getRange('B25:D25').formulas[0], ['="NOT RUN"', '="STOP"', '="STOP"']);
});

test('builder and model declare explicit provenance, identity, and evidence surfaces', async () => {
  const [builder, model] = await Promise.all([read('src/build_directional.mjs'), read('src/directional_workbook_model.mjs')]);
  assert.match(builder, /buildTrajectoryProvenanceSurface/);
  assert.match(builder, /buildTrajectoryIdentitySurface/);
  assert.match(model, /buildTrajectoryEngineEvidence/);
  assert.match(builder, /RUST REQUIRED — NO VBA FALLBACK/);
  assert.match(builder, /createHash\('sha256'\)/);
  assert.match(model, /Rust executable \/ hash verification/);
  assert.doesNotMatch(model, /\['VBA \/ external links', '="None"'/);
});

test('engine manifest declares directional Rust authority and fixed release artifacts', async () => {
  const manifest = JSON.parse(await read('data/wellforge-vba-engine-manifest.json'));
  const directional = manifest.workbooks.find(({ kind }) => kind === 'directional');
  assert.equal(directional.calculationAuthority, 'Rust');
  assert.equal(directional.entryPoint, 'WF_RunTrajectoryRustEngine');
  assert.equal(directional.executable, 'wellforge-trajectory.exe');
  assert.equal(directional.hashManifest, 'wellforge-trajectory.exe.sha256');
});

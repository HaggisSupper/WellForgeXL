import test from 'node:test';
import assert from 'node:assert/strict';

test('shared contract declares SI source dimensions and supported display systems', async () => {
  const { UNIT_SYSTEMS, CUSTOM_UNIT_SYSTEMS, UNIT_ROWS } = await import('../src/common.mjs');
  assert.deepEqual(UNIT_SYSTEMS, ['SI', 'Imperial', 'Mixed', 'Custom']);
  assert.deepEqual(CUSTOM_UNIT_SYSTEMS, ['SI', 'Imperial', 'Mixed']);
  assert.ok(UNIT_ROWS.some((row) => row.domain === 'Length' && row.siUnit === 'm'));
  assert.ok(UNIT_ROWS.some((row) => row.domain === 'Force' && row.siUnit === 'N'));
  assert.ok(UNIT_ROWS.some((row) => row.domain === 'Pressure' && row.siUnit === 'Pa'));
  assert.ok(UNIT_ROWS.some((row) => row.domain === 'Torque' && row.siUnit === 'N-m'));
});

test('unit rows retain stable positions and explicit conversion contracts', async () => {
  const { UNIT_ROWS } = await import('../src/common.mjs');
  const stableDomains = ['Length', 'Diameter', 'Area', 'Volume', 'Flow rate', 'Density', 'Force', 'Pressure', 'Torque', 'Stress', 'Angle', 'Speed'];
  assert.deepEqual(UNIT_ROWS.slice(0, stableDomains.length).map((row) => row.domain), stableDomains);
  for (const row of UNIT_ROWS) {
    assert.equal(row.multiplier, row.imperialMultiplier, `${row.domain} preserves its Imperial compatibility multiplier`);
    assert.equal(typeof row.mixedMultiplier, 'number', `${row.domain} declares a Mixed multiplier`);
    assert.equal(typeof row.offset, 'number', `${row.domain} declares an offset`);
  }
});

test('Angular gradient conversion is appended with the requested display factors', async () => {
  const { UNIT_ROWS } = await import('../src/common.mjs');
  const row = UNIT_ROWS.at(-1);
  assert.deepEqual(row, {
    domain: 'Angular gradient',
    siUnit: 'rad/m',
    imperialUnit: 'deg/100ft',
    mixedUnit: 'deg/30m',
    imperialMultiplier: 1746.37535955875,
    mixedMultiplier: 1718.87338539247,
    multiplier: 1746.37535955875,
    offset: 0,
  });
});

test('Stress declares the correct Pa to MPa Mixed factor', async () => {
  const { UNIT_ROWS } = await import('../src/common.mjs');
  assert.equal(UNIT_ROWS.find((row) => row.domain === 'Stress').mixedMultiplier, 0.000001);
});

test('display conversion remains formula-driven from SI source values', async () => {
  const { displayFormula } = await import('../src/common.mjs');
  assert.equal(
    displayFormula("'Calc'!D12", "$E$7", "$F$7"),
    "='Calc'!D12*'Unit Map'!$E$7+'Unit Map'!$F$7",
  );
});

test('display conversion defaults to a literal zero offset', async () => {
  const { displayFormula } = await import('../src/common.mjs');
  assert.equal(displayFormula('A1', '$I$8'), "=A1*'Unit Map'!$I$8+0");
});

test('calculation helper sheets are hidden from the normal end-user workflow', async () => {
  const { createSuiteWorkbook } = await import('../src/workbook.mjs');
  const { sheets } = createSuiteWorkbook('Visibility contract');
  assert.equal(sheets.Calc.visibility, 'hidden');
});

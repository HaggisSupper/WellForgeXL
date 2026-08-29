import test from 'node:test';
import assert from 'node:assert/strict';

test('hydraulics pressure roadmap distinguishes hydrostatic, friction, and total dynamic pressure', async () => {
  const { buildHydraulicsWorkbook } = await import('../src/build_hydraulics.mjs');
  const workbook = buildHydraulicsWorkbook();
  const profile = workbook.worksheets.getItem('Pressure Profile');
  const dynamicPressure = profile.getRange('G6').formulas[0][0];
  assert.match(dynamicPressure, /Inputs'?!\$B\$9\*9\.80665/,
    'dynamic pressure must include hydrostatic pressure');
  assert.match(dynamicPressure, /Calc'?!H6/,
    'dynamic pressure must include cumulative friction pressure');

  const velocityScreen = workbook.worksheets.getItem('Hydraulics Charts').getRange('C45:C52').formulas.flat();
  assert.ok(velocityScreen.every((formula) => !/NA\(\)/.test(formula)),
    'the visible transport-screen helper must not publish formula errors');
});

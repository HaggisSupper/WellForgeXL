import test from 'node:test';
import assert from 'node:assert/strict';

test('torque drag model derives a depth profile and buckling screen from SI inputs', async () => {
  const { torqueDragFormulaPlan } = await import('../src/build_torque_drag.mjs');
  const plan = torqueDragFormulaPlan();
  assert.match(plan.dogleg, /ACOS/);
  assert.match(plan.buoyedWeight, /1-Inputs!\$B\$8\/7850/);
  assert.match(plan.drag, /\$B\$9/);
  assert.match(plan.helicalFlag, /IF\(/);
});

test('torque drag presents Young modulus in engineering-scale units while calculating in Pa', async () => {
  const { buildTorqueDragWorkbook } = await import('../src/build_torque_drag.mjs');
  const workbook = buildTorqueDragWorkbook();
  const inputs = workbook.worksheets.getItem('Inputs');
  const calc = workbook.worksheets.getItem('Calc');
  assert.equal(inputs.getRange('A12').values[0][0], 'Young modulus');
  assert.equal(inputs.getRange('B12').values[0][0], 206.84271879504);
  assert.equal(inputs.getRange('C12').values[0][0], 'GPa');
  assert.deepEqual(inputs.getRange('C12').dataValidation.rule.values, ['GPa', 'MPa', 'Pa', 'Mpsi', 'psi']);
  assert.match(calc.getRange('L6').formulas[0][0], /Inputs!\$B\$12\*IF\(Inputs!\$C\$12="GPa",1E9/);
});

test('torque drag derives governing limits from the folded API 7G layer', async () => {
  const { buildTorqueDragWorkbook } = await import('../src/build_torque_drag.mjs');
  const workbook = buildTorqueDragWorkbook();
  const limits = workbook.worksheets.getItem('API 7G Limits');
  const inputs = workbook.worksheets.getItem('Inputs');
  assert.match(limits.getRange('B10').formulas[0][0], /Inputs!\$B\$9\^2-Inputs!\$B\$10\^2/);
  assert.match(limits.getRange('B11').formulas[0][0], /SQRT\(3\)/);
  assert.equal(inputs.getRange('B13').formulas[0][0], "='API 7G Limits'!$B$10");
  assert.equal(inputs.getRange('B14').formulas[0][0], "='API 7G Limits'!$B$11");
});

import test from 'node:test';
import assert from 'node:assert/strict';

test('hydraulics model covers every tube section with formula-based pressure loss and nozzle ranking', async () => {
  const { hydraulicsFormulaPlan } = await import('../src/build_hydraulics.mjs');
  const plan = hydraulicsFormulaPlan();
  assert.equal(plan.velocity, '=Inputs!$B$8/C6');
  assert.match(plan.sectionPressureLoss, /F6\*B6\/C6/);
  assert.match(plan.nozzleArea, /PI\(\)\/4/);
  assert.match(plan.candidateScore, /SUM\(/);
});

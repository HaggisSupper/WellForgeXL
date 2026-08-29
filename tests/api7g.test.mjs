import test from 'node:test';
import assert from 'node:assert/strict';

test('API 7G model uses SI tubular-area, buoyancy and utilisation formulas', async () => {
  const { api7gFormulaPlan } = await import('../src/build_api7g.mjs');
  const plan = api7gFormulaPlan();
  assert.equal(plan.metalArea, '=PI()/4*(D6^2-E6^2)');
  assert.equal(plan.buoyancyFactor, '=1-C6/7850');
  assert.match(plan.tensionUtilisation, /F6\/G6/);
  assert.match(plan.torqueUtilisation, /H6\/I6/);
});

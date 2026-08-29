import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { buildMockExchangePayload } from '../src/exchange/build_mock_payload.mjs';
import { quantity } from '../src/exchange/schema_contract.mjs';
import { validateExchangePayload } from '../src/exchange/schema_validator.mjs';
import { directionalReferenceData } from '../src/directional_reference_data.mjs';
import { MOCK_CASE } from '../src/shared_mock_case.mjs';

test('mock payload covers every analysis branch and validates', () => {
  const payload = buildMockExchangePayload();
  assert.deepEqual(Object.keys(payload.analyses).sort(),
    ['api7g', 'bha', 'directional', 'hydraulics', 'torqueDrag']);
  assert.equal(validateExchangePayload(payload).valid, true);
});

test('trajectory and repeated operating values come from the shared case', () => {
  const payload = buildMockExchangePayload();
  assert.equal(payload.trajectory.survey.length, 60);
  assert.deepEqual(payload.fluids[0].density, quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'));
  assert.deepEqual(payload.operatingPoint.wob, quantity(MOCK_CASE.operation.wobN, 'N'));
});

test('hydraulics branch contains every shared flow-path section', () => {
  const payload = buildMockExchangePayload();
  const flowPath = payload.analyses.hydraulics.flowPath;
  assert.equal(Array.isArray(flowPath), true);
  if (Array.isArray(flowPath)) {
    assert.equal(flowPath.length, 8);
    assert.deepEqual(flowPath[3].length, quantity(MOCK_CASE.tubular.drillPipe.lengthM, 'm'));
  }
});

test('analysis branches retain their complete input data while results are not calculated', () => {
  const payload = buildMockExchangePayload();

  assert.equal(payload.analyses.api7g.sections.length, 6);
  assert.deepEqual(payload.analyses.api7g.sections[0].axialLoad, quantity(850000, 'N'));
  assert.deepEqual(payload.analyses.directional.inputs.motorYield, quantity(8, 'deg/100ft'));
  assert.deepEqual(payload.analyses.torqueDrag.inputs.surfaceTorque, quantity(MOCK_CASE.operation.surfaceTorqueNm, 'N*m'));
  for (const analysis of Object.values(payload.analyses)) {
    assert.equal(analysis.calculationState, 'notCalculated');
    assert.deepEqual(analysis.results, []);
  }
});

test('natural source identifiers remain stable and checked-in JSON matches the builder', () => {
  const payload = buildMockExchangePayload();
  assert.deepEqual(payload.trajectory.targets.map(({ id }) => id), directionalReferenceData.targets.map(({ id }) => id));
  assert.deepEqual(payload.trajectory.formationTops.map(({ id }) => id), directionalReferenceData.formationTops.map(({ id }) => id));
  assert.deepEqual(payload.pumpNozzle.nozzles.map(({ id }) => id), MOCK_CASE.pumpNozzle.nozzles.map(({ id }) => id));
  const fixture = JSON.parse(readFileSync(new URL('../data/wellforge-mock-case.json', import.meta.url), 'utf8'));
  assert.deepEqual(fixture, payload);
});

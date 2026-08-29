import test from 'node:test';
import assert from 'node:assert/strict';
import { MOCK_CASE } from '../src/shared_mock_case.mjs';
import { buildApi7gWorkbook } from '../src/build_api7g.mjs';
import { buildHydraulicsWorkbook } from '../src/build_hydraulics.mjs';
import { buildTorqueDragWorkbook } from '../src/build_torque_drag.mjs';
import { buildBhaWorkbook } from '../src/build_bha.mjs';
import { directionalReferenceData } from '../src/directional_reference_data.mjs';
import * as mockCaseModule from '../src/shared_mock_case.mjs';

test('all engineering workbooks use the shared mock operating case for repeated values', () => {
  const api = buildApi7gWorkbook().worksheets.getItem('Inputs');
  const hydraulics = buildHydraulicsWorkbook().worksheets.getItem('Inputs');
  const torqueDrag = buildTorqueDragWorkbook().worksheets.getItem('Inputs');
  const bha = buildBhaWorkbook().worksheets.getItem('Inputs');

  assert.equal(MOCK_CASE.fluid.densityKgM3, 1200);
  assert.equal(api.getRange('C6:C11').values.flat().every((value) => value === MOCK_CASE.fluid.densityKgM3), true);
  assert.equal(hydraulics.getRange('B9').values[0][0], MOCK_CASE.fluid.densityKgM3);
  assert.equal(torqueDrag.getRange('B5').values[0][0], MOCK_CASE.fluid.densityKgM3);
  assert.equal(hydraulics.getRange('B8').values[0][0], MOCK_CASE.hydraulics.flowRateM3S);
  assert.equal(bha.getRange('B6').values[0][0], MOCK_CASE.hydraulics.flowRateM3S);
  assert.equal(bha.getRange('B5').values[0][0], MOCK_CASE.operation.rotarySpeedRpm);
  assert.equal(torqueDrag.getRange('B7').values[0][0], MOCK_CASE.operation.wobN);
  assert.equal(bha.getRange('B10').values[0][0], MOCK_CASE.operation.wobN);
  assert.equal(torqueDrag.getRange('B9').values[0][0], MOCK_CASE.tubular.drillPipe.odM);
  assert.equal(torqueDrag.getRange('B10').values[0][0], MOCK_CASE.tubular.drillPipe.idM);
  assert.equal(hydraulics.getRange('E9:F9').values[0].join(','), `${MOCK_CASE.tubular.drillPipe.lengthM},${MOCK_CASE.tubular.drillPipe.idM}`);
  assert.equal(hydraulics.getRange('B7').values[0][0], MOCK_CASE.rig.pumpEfficiency);
  assert.equal(hydraulics.getRange('B11').values[0][0], MOCK_CASE.pumpNozzle.nozzleCount);
  assert.equal(hydraulics.getRange('E6:H13').values[0][0], MOCK_CASE.hydraulics.flowPath[0].lengthM);
  assert.equal(hydraulics.getRange('E6:H13').values[7][1], MOCK_CASE.hydraulics.flowPath[7].flowIdM);
  assert.equal(api.getRange('F6:H11').values[0][0], MOCK_CASE.api7g.sections[0].axialLoadN);
});

test('torque and drag reuses the directional mock survey in canonical SI', () => {
  const torqueDrag = buildTorqueDragWorkbook().worksheets.getItem('Survey');
  assert.equal(MOCK_CASE.surveyStations.length, directionalReferenceData.survey.length);
  const first = directionalReferenceData.survey[0];
  const last = directionalReferenceData.survey.at(-1);
  assert.deepEqual(torqueDrag.getRange('A6:C6').values[0], [
    first.mdFt * 0.3048,
    first.incDeg * Math.PI / 180,
    first.aziDeg * Math.PI / 180,
  ]);
  assert.deepEqual(torqueDrag.getRange('A65:C65').values[0], [
    last.mdFt * 0.3048,
    last.incDeg * Math.PI / 180,
    last.aziDeg * Math.PI / 180,
  ]);
});

test('hydraulics flow path derives tubular and hole geometry from supplied source records', () => {
  assert.equal(typeof mockCaseModule.buildHydraulicsFlowPath, 'function');
  if (typeof mockCaseModule.buildHydraulicsFlowPath !== 'function') return;

  const tubular = {
    ...MOCK_CASE.tubular,
    drillPipe: { ...MOCK_CASE.tubular.drillPipe, lengthM: 2222, idM: 0.111 },
    drillCollar: { ...MOCK_CASE.tubular.drillCollar, odM: 0.18 },
  };
  const holeSections = [{ ...MOCK_CASE.holeSections[0], topMdM: 10, bottomMdM: 2710, holeIdM: 0.222 }];
  const flowPath = mockCaseModule.buildHydraulicsFlowPath({ tubular, holeSections });

  assert.deepEqual(flowPath.find(({ id }) => id === 'flow-drill-pipe'), {
    id: 'flow-drill-pipe', name: 'Drill pipe', lengthM: 2222, flowIdM: 0.111, hydraulicDiameterM: 0.111, flowType: 'Pipe',
  });
  assert.deepEqual(flowPath.find(({ id }) => id === 'flow-open-hole-annulus'), {
    id: 'flow-open-hole-annulus', name: 'Open-hole annulus', lengthM: 2700, flowIdM: 0.18, hydraulicDiameterM: 0.044, flowType: 'Annulus',
  });
});

test('BHA input table retains each shared component record', () => {
  const bha = buildBhaWorkbook().worksheets.getItem('Inputs');
  assert.deepEqual(
    bha.getRange('D6:H11').values,
    MOCK_CASE.bha.map(({ name, lengthM, odM, idM, supportFactor }) => [name, lengthM, odM, idM, supportFactor]),
  );
});

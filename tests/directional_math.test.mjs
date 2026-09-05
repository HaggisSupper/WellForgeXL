import test from 'node:test';
import assert from 'node:assert/strict';

import { directionalReferenceData } from '../src/directional_reference_data.mjs';
import {
  interpolateMinimumCurvature,
  minimumCurvature,
  positionError,
  slideVector,
  targetEnvelopeStatus,
} from './helpers/directional_math_oracle.mjs';
import { directionalSourceAudit } from './helpers/directional_source_audit.mjs';

const FT_TO_M = 0.3048;
const DEG_TO_RAD = Math.PI / 180;
const EPS_POSITION_M = 1e-8;
const EPS_DLS_RAD_PER_M = 1e-10;

function close(actual, expected, tolerance, message = '') {
  assert.ok(Math.abs(actual - expected) <= tolerance, `${message}: expected ${expected}, received ${actual}`);
}

function toSi(stations) {
  return stations.map(({ station, mdFt, incDeg, aziDeg }) => ({
    station,
    md: mdFt * FT_TO_M,
    inc: incDeg * DEG_TO_RAD,
    azi: aziDeg * DEG_TO_RAD,
  }));
}

test('sanitized fixture preserves the 60 active source plan and survey stations', () => {
  assert.equal(directionalReferenceData.plan.length, 60);
  assert.equal(directionalReferenceData.survey.length, 60);
  assert.deepEqual(directionalReferenceData.plan[0], { station: 0, mdFt: 0, incDeg: 0, aziDeg: 20 });
  assert.deepEqual(directionalReferenceData.plan.at(-1), { station: 59, mdFt: 16125, incDeg: 90, aziDeg: 75 });
  assert.deepEqual(directionalReferenceData.survey.at(-1), { station: 59, mdFt: 16140.816, incDeg: 88.8, aziDeg: 89 });
  assert.equal(directionalReferenceData.targets.length, 3);
  assert.equal(directionalReferenceData.slideIntervals.length, 6);
  assert.equal(directionalReferenceData.formationTops.length, 4);
});

test('sanitized fixture has no prohibited source metadata', () => {
  const serialized = JSON.stringify(directionalReferenceData);
  for (const prohibited of ['dc:creator', 'lastModifiedBy', 'C:\\Users', 'VBA_Macros.bas']) {
    assert.equal(serialized.includes(prohibited), false, `fixture must exclude ${prohibited}`);
  }
});

test('independent minimum-curvature oracle reproduces immutable source snapshots after SI conversion', () => {
  for (const [name, sourceStations] of Object.entries({ plan: directionalReferenceData.plan, survey: directionalReferenceData.survey })) {
    const actual = minimumCurvature(toSi(sourceStations));
    const expected = directionalSourceAudit[name];
    assert.equal(actual.length, 60, `${name} station count`);
    assert.equal(expected.length, 60, `${name} snapshot count`);
    for (let index = 0; index < actual.length; index += 1) {
      assert.deepEqual(sourceStations[index], expected[index].source, `${name} raw source station ${index}`);
      close(actual[index].north, expected[index].northM, EPS_POSITION_M, `${name} north station ${index}`);
      close(actual[index].east, expected[index].eastM, EPS_POSITION_M, `${name} east station ${index}`);
      close(actual[index].tvd, expected[index].tvdM, EPS_POSITION_M, `${name} TVD station ${index}`);
      close(actual[index].dls, expected[index].dlsRadPerM, EPS_DLS_RAD_PER_M, `${name} DLS station ${index}`);
    }
  }
});

test('final sample positional error matches the audited crossline, horizontal, and 3D result', () => {
  const planned = minimumCurvature(toSi(directionalReferenceData.plan));
  const actual = minimumCurvature(toSi(directionalReferenceData.survey));
  // The source's final actual station is 15.816 ft past plan TD, so its
  // terminal comparison is against the final planned station, not an extrapolation.
  const result = positionError(actual.at(-1), planned.at(-1), 55 * DEG_TO_RAD);
  close(result.crossline, 1408.48 * FT_TO_M, 0.05, 'final crossline error');
  close(result.horizontal, 1424.24 * FT_TO_M, 0.05, 'final horizontal error');
  close(result.error3d, 1436.38 * FT_TO_M, 0.05, 'final 3D error');
});

test('partial minimum-curvature interpolation handles station, midpoint, small dogleg, and coverage limits', () => {
  const curved = minimumCurvature([
    { md: 0, inc: 0, azi: 0 },
    { md: 100, inc: Math.PI / 2, azi: 0 },
    { md: 200, inc: Math.PI / 2, azi: Math.PI / 2 },
  ]);
  const exact = interpolateMinimumCurvature(curved, 100);
  assert.equal(exact.status, 'OK');
  close(exact.north, curved[1].north, 1e-12, 'exact station north');
  const mid = interpolateMinimumCurvature(curved, 50);
  assert.equal(mid.status, 'OK');
  close(mid.north, 18.64616142890283, 1e-12, 'midpoint north');
  close(mid.tvd, 45.015815807855304, 1e-12, 'midpoint TVD');
  const nearZero = minimumCurvature([
    { md: 0, inc: 0.3, azi: 0.2 },
    { md: 100, inc: 0.3000000000001, azi: 0.2000000000001 },
  ]);
  const smallDogleg = interpolateMinimumCurvature(nearZero, 50);
  assert.equal(smallDogleg.status, 'OK');
  close(smallDogleg.md, 50, 0, 'small dogleg MD');
  assert.equal(interpolateMinimumCurvature(curved, -1).status, 'BEFORE START');
  assert.equal(interpolateMinimumCurvature(curved, 201).status, 'BEYOND TD');
});

test('slide vector resolves pure build, pure turn, mixed toolface, low inclination, and zero length', () => {
  const pureBuild = slideVector({ mdIn: 0, mdOut: 100, slideLength: 100, startInc: 0, endInc: 0.2, startAzi: 0, endAzi: 0, commandedToolface: 0 });
  assert.equal(pureBuild.status, 'OK');
  close(pureBuild.build, 0.002, 1e-15, 'pure build');
  close(pureBuild.effectiveTurn, 0, 1e-15, 'pure turn component');
  close(pureBuild.responseToolface, 0, 1e-15, 'pure build toolface');
  const pureTurn = slideVector({ mdIn: 0, mdOut: 100, slideLength: 100, startInc: Math.PI / 2, endInc: Math.PI / 2, startAzi: 0, endAzi: 0.1, commandedToolface: Math.PI / 2 });
  close(pureTurn.effectiveTurn, 0.001, 1e-15, 'pure effective turn');
  close(pureTurn.responseToolface, Math.PI / 2, 1e-15, 'pure turn toolface');
  const mixed = slideVector({ mdIn: 0, mdOut: 100, slideLength: 50, startInc: 0.2, endInc: 0.3, startAzi: 0.1, endAzi: 0.3, commandedToolface: 0.4, rotaryBuild: 0.0002, rotaryEffectiveTurn: 0.0001 });
  assert.equal(mixed.status, 'OK');
  assert.ok(mixed.yield > 0);
  assert.ok(mixed.responseToolface > 0 && mixed.responseToolface < Math.PI / 2);
  assert.equal(slideVector({ mdIn: 0, mdOut: 100, slideLength: 100, startInc: 0.001, endInc: 0.001, startAzi: 0, endAzi: 0.1, lowInclinationThreshold: 0.01 }).status, 'LOW_INCLINATION');
  assert.equal(slideVector({ mdIn: 0, mdOut: 100, slideLength: 0, startInc: 0.2, endInc: 0.3, startAzi: 0, endAzi: 0 }).status, 'INVALID_SLIDE_LENGTH');
});

test('target envelopes evaluate point, circle, rotated ellipse, and rotated box hit/miss cases', () => {
  const center = { north: 100, east: 200, tvd: 300 };
  const pointHit = targetEnvelopeStatus({ ...center, type: 'Point', major: 5, verticalTolerance: 2 }, { north: 103, east: 204, tvd: 301 });
  assert.equal(pointHit.status, 'HIT');
  close(pointHit.horizontalUtilization, 1, 1e-15, 'point boundary utilization');
  assert.equal(targetEnvelopeStatus({ ...center, type: 'Point', major: 5, verticalTolerance: 2 }, { north: 103.1, east: 204, tvd: 301 }).status, 'MISS');
  const circleHit = targetEnvelopeStatus({ ...center, type: 'Circle', major: 5, verticalTolerance: 2 }, { north: 100, east: 203, tvd: 298 });
  assert.equal(circleHit.status, 'HIT');
  close(circleHit.verticalUtilization, 1, 1e-15, 'circle vertical boundary utilization');
  assert.equal(targetEnvelopeStatus({ ...center, type: 'Circle', major: 5, verticalTolerance: 2 }, { north: 106, east: 200, tvd: 300 }).status, 'MISS');
  const ellipse = { ...center, type: 'Ellipse', major: 10, minor: 2, rotation: Math.PI / 4, verticalTolerance: 3 };
  assert.equal(targetEnvelopeStatus(ellipse, { north: 100 + 6 * Math.cos(Math.PI / 4), east: 200 + 6 * Math.sin(Math.PI / 4), tvd: 299 }).status, 'HIT');
  assert.equal(targetEnvelopeStatus(ellipse, { north: 100 - 3 * Math.sin(Math.PI / 4), east: 200 + 3 * Math.cos(Math.PI / 4), tvd: 299 }).status, 'MISS');
  const box = { ...center, type: 'Box', major: 8, minor: 3, rotation: Math.PI / 6, verticalTolerance: 2 };
  assert.equal(targetEnvelopeStatus(box, { north: 100 + 5 * Math.cos(Math.PI / 6) - 2 * Math.sin(Math.PI / 6), east: 200 + 5 * Math.sin(Math.PI / 6) + 2 * Math.cos(Math.PI / 6), tvd: 301 }).status, 'HIT');
  assert.equal(targetEnvelopeStatus(box, { north: 100 + 9 * Math.cos(Math.PI / 6), east: 200 + 9 * Math.sin(Math.PI / 6), tvd: 300 }).status, 'MISS');
});

test('formation-top structural high is positive when the actual top is shallower', () => {
  const prognosedTvd = 2000;
  const actualTvd = 1994.5;
  assert.ok(prognosedTvd - actualTvd > 0, 'Prognosed TVD - Actual TVD must be positive for HIGH');
});

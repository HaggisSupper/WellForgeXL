import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  SCHEMA_VERSION,
  UNIT_REGISTRY,
  fromSi,
  quantity,
  toSi,
} from '../src/exchange/schema_contract.mjs';
import { validateExchangePayload } from '../src/exchange/schema_validator.mjs';

function minimalExchangePayload({ wob = quantity(120000, 'N') } = {}) {
  return {
    schemaVersion: '1.0.0',
    caseId: 'wellforge-mock-case',
    createdAt: '2026-08-27T00:00:00.000Z',
    producer: { name: 'WellForge', version: '1.0.0' },
    metadata: { well: 'WF-1', field: 'Example Field', pad: 'P-1', rig: 'Rig 1', datum: 'MSL' },
    unitPreferences: { length: 'm', force: 'N' },
    trajectory: {
      plan: [],
      survey: [{ id: 'survey-000', md: quantity(0, 'm') }],
      targets: [],
      slideIntervals: [],
      formationTops: [],
    },
    holeSections: [],
    tubulars: [],
    bhaComponents: [],
    fluids: [],
    operatingPoint: { wob },
    rigLimits: {},
    pumpNozzle: { pumps: [], nozzles: [] },
    analyses: {},
    provenance: { notes: [] },
    warnings: [],
  };
}

test('registry declares reversible units with dimensions', () => {
  for (const [symbol, unit] of Object.entries(UNIT_REGISTRY)) {
    assert.equal(typeof unit.dimension, 'string', symbol);
    assert.equal(Number.isFinite(unit.toSiMultiplier), true, symbol);
    assert.equal(Number.isFinite(unit.toSiOffset), true, symbol);
  }

  for (const dimension of [
    'unitless', 'length', 'diameter', 'area', 'volume', 'flowRate', 'density',
    'force', 'pressure', 'torque', 'stress', 'angle', 'speed', 'angularGradient',
    'viscosity', 'rheologyConsistency', 'compressibility', 'frequency',
    'rotationalSpeed', 'date', 'temperature',
  ]) {
    assert.equal(Object.values(UNIT_REGISTRY).some((unit) => unit.dimension === dimension), true, dimension);
  }
});

test('converts quantities through canonical SI with dimension checks', () => {
  assert.ok(Math.abs(toSi(quantity(28, 'klbf'), 'force') - 124550.4428) < 1e-9);
  assert.ok(Math.abs(fromSi(124550.4428, 'klbf', 'force') - 28) < 1e-12);
  assert.throws(() => toSi(quantity(1, 'psi'), 'force'), /Unit psi is not valid for force/);
});

test('converts rpm through canonical rad/s rotational speed', () => {
  assert.equal(UNIT_REGISTRY['rad/s'].dimension, 'rotationalSpeed');
  assert.ok(Math.abs(toSi(quantity(60, 'rpm'), 'rotationalSpeed') - (2 * Math.PI)) < 1e-12);
  assert.ok(Math.abs(fromSi(2 * Math.PI, 'rpm', 'rotationalSpeed') - 60) < 1e-12);
});

test('schema accepts the minimum complete exchange payload', () => {
  const result = validateExchangePayload(minimalExchangePayload());
  assert.deepEqual(result, { valid: true, errors: [] });
  assert.equal(SCHEMA_VERSION, '1.0.0');
});

test('schema rejects a quantity without a unit', () => {
  const payload = minimalExchangePayload();
  payload.operatingPoint.wob = { value: 120000 };
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.unit/);
});

test('schema rejects unknown quantity units and duplicate stable identifiers', () => {
  const unknownUnit = minimalExchangePayload({ wob: { value: 120000, unit: 'furlong' } });
  const unknownUnitResult = validateExchangePayload(unknownUnit);
  assert.equal(unknownUnitResult.valid, false);
  assert.match(unknownUnitResult.errors.join('\n'), /operatingPoint\.wob\.unit.*furlong/);

  const duplicateIdentifiers = minimalExchangePayload();
  duplicateIdentifiers.trajectory.survey.push({ id: 'survey-000', md: quantity(10, 'm') });
  const duplicateResult = validateExchangePayload(duplicateIdentifiers);
  assert.equal(duplicateResult.valid, false);
  assert.match(duplicateResult.errors.join('\n'), /trajectory\.survey.*duplicate identifier survey-000/);
});

test('schema rejects malformed semantic versions and missing stable identifiers', () => {
  const payload = minimalExchangePayload();
  payload.schemaVersion = 'version one';
  payload.trajectory.survey[0] = { md: quantity(0, 'm') };
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /schemaVersion.*semantic version/);
  assert.match(result.errors.join('\n'), /trajectory\.survey\[0\]\.id/);

  const leadingZeroPrerelease = minimalExchangePayload();
  leadingZeroPrerelease.schemaVersion = '1.0.0-01';
  const leadingZeroResult = validateExchangePayload(leadingZeroPrerelease);
  assert.equal(leadingZeroResult.valid, false);
  assert.match(leadingZeroResult.errors.join('\n'), /schemaVersion.*semantic version/);
});

test('schema rejects an unsupported schema major version', () => {
  const payload = minimalExchangePayload();
  payload.schemaVersion = '2.0.0';
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /schemaVersion.*unsupported major version/);
});

test('schema requires operating point values to be quantity objects', () => {
  const payload = minimalExchangePayload({ wob: 120000 });
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob must be an object/);
});

test('schema rejects unsupported quantity metadata and invalid metadata types', () => {
  const payload = minimalExchangePayload({
    wob: {
      value: 120000,
      unit: 'N',
      quality: 1,
      source: false,
      timestamp: 0,
      note: [],
      extra: 'not part of the contract',
    },
  });
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.quality must be a string/);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.source must be a string/);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.timestamp must be a string/);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.note must be a string/);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.extra is not allowed/);
});

test('external schema artifact constrains version and quantity contract', () => {
  const schema = JSON.parse(readFileSync(new URL('../schema/wellforge-analysis-exchange.schema.json', import.meta.url), 'utf8'));
  const versionPattern = new RegExp(schema.properties.schemaVersion.pattern);
  const quantitySchema = schema.$defs.quantity;

  assert.equal(schema.$schema, 'https://json-schema.org/draft/2020-12/schema');
  assert.equal(versionPattern.test('1.4.2'), true);
  assert.equal(versionPattern.test('2.0.0'), false);
  assert.equal(versionPattern.test('1.0.0-01'), false);
  assert.equal(versionPattern.test('1.0.0-beta.1'), true);
  assert.deepEqual(quantitySchema.required, ['value', 'unit']);
  assert.equal(quantitySchema.additionalProperties, false);
  assert.equal(quantitySchema.properties.quality.type, 'string');
  assert.equal(quantitySchema.properties.source.type, 'string');
  assert.equal(quantitySchema.properties.timestamp.type, 'string');
  assert.equal(quantitySchema.properties.note.type, 'string');
  assert.ok(quantitySchema.properties.unit.enum.includes('GPa'));
  assert.ok(quantitySchema.properties.unit.enum.includes('Mpsi'));
  assert.deepEqual(schema.properties.operatingPoint.additionalProperties, { $ref: '#/$defs/quantity' });
});

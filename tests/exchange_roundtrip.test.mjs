import test from 'node:test';
import assert from 'node:assert/strict';

import { buildMockExchangePayload } from '../src/exchange/build_mock_payload.mjs';
import {
  exportMappedPayload,
  importMappedPayload,
} from '../src/exchange/exchange_engine.mjs';
import { getPointer, mergeByStableId, setPointer } from '../src/exchange/json_pointer.mjs';
import { mockAdapter } from './helpers/mock_workbook_adapter.mjs';

const wobMap = [{
  pointer: '/operatingPoint/wob', direction: 'Input', sheet: 'Inputs', address: 'B7',
  shape: 'Scalar', unitSource: 'N', dimension: 'force', dataType: 'number', required: true, writable: true,
}];

function payloadWithWob(wob) {
  const payload = buildMockExchangePayload();
  payload.operatingPoint.wob = wob;
  return payload;
}

test('JSON pointers decode escaped tokens and create missing containers', () => {
  const document = { 'a/b': { '~key': 4 } };
  assert.equal(getPointer(document, '/a~1b/~0key'), 4);
  setPointer(document, '/new/0/value', 9);
  assert.deepEqual(document.new, [{ value: 9 }]);
});

test('stable-ID merge recursively retains unknown fields and records', () => {
  const existing = {
    metadata: { well: 'WF-1', extension: { owner: 'operator' } },
    rows: [{ id: 'a', value: 1, vendor: 'kept' }, { id: 'unknown', value: 7 }],
  };
  const merged = mergeByStableId(existing, {
    metadata: { well: 'WF-2' },
    rows: [{ id: 'a', value: 2 }, { id: 'new', value: 3 }],
  });
  assert.deepEqual(merged, {
    metadata: { well: 'WF-2', extension: { owner: 'operator' } },
    rows: [{ id: 'a', value: 2, vendor: 'kept' }, { id: 'unknown', value: 7 }, { id: 'new', value: 3 }],
  });
});

test('unchanged imported quantities retain their original units', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 0 });
  const payload = payloadWithWob({ value: 28, unit: 'klbf' });
  const imported = importMappedPayload(adapter, payload, wobMap, []);
  assert.equal(imported.ok, true);
  assert.ok(Math.abs(adapter.read('Inputs', 'B7') - 124550.4428) < 1e-9);

  const exported = exportMappedPayload(adapter, payload, wobMap, imported.state, {});
  assert.equal(exported.ok, true);
  assert.deepEqual(exported.payload.operatingPoint.wob, { value: 28, unit: 'klbf' });
});

test('changed quantities use the preferred display unit and otherwise canonical SI', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 0 });
  const payload = payloadWithWob({ value: 28, unit: 'klbf' });
  const imported = importMappedPayload(adapter, payload, wobMap, []);
  adapter.write('Inputs', 'B7', 100000);

  const preferred = exportMappedPayload(adapter, payload, wobMap, imported.state, { displayUnits: { force: 'lbf' } });
  assert.ok(Math.abs(preferred.payload.operatingPoint.wob.value - (100000 / 4.4482301)) < 1e-9);
  assert.equal(preferred.payload.operatingPoint.wob.unit, 'lbf');

  const canonical = exportMappedPayload(adapter, payload, wobMap, imported.state, {});
  assert.deepEqual(canonical.payload.operatingPoint.wob, { value: 100000, unit: 'N' });
});

test('failed validation leaves every destination unchanged', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 120000, 'Inputs!B8': 4000 });
  const map = [...wobMap, { ...wobMap[0], pointer: '/operatingPoint/surfaceTorque', address: 'B8', unitSource: 'N*m', dimension: 'torque' }];
  const payload = payloadWithWob({ value: 3000, unit: 'psi' });
  const result = importMappedPayload(adapter, payload, map, []);
  assert.equal(result.ok, false);
  assert.match(result.errors.join('\n'), /not valid for force/);
  assert.equal(adapter.read('Inputs', 'B7'), 120000);
  assert.equal(adapter.read('Inputs', 'B8'), 4000);
  assert.equal(adapter.writes.length, 0);
});

test('write failures restore the complete captured change set', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 120000, 'Inputs!B8': 4000 }, { failOnWrite: 2 });
  const map = [...wobMap, { ...wobMap[0], pointer: '/operatingPoint/surfaceTorque', address: 'B8', unitSource: 'N*m', dimension: 'torque' }];
  const result = importMappedPayload(adapter, buildMockExchangePayload(), map, []);
  assert.equal(result.ok, false);
  assert.match(result.errors.join('\n'), /simulated write failure/);
  assert.equal(adapter.read('Inputs', 'B7'), 120000);
  assert.equal(adapter.read('Inputs', 'B8'), 4000);
});

test('capture failures return diagnostics before any write is attempted', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 120000 }, { failOnCapture: 'Inputs!B7' });
  const result = importMappedPayload(adapter, buildMockExchangePayload(), wobMap, []);
  assert.equal(result.ok, false);
  assert.match(result.errors.join('\n'), /simulated capture failure/);
  assert.equal(adapter.read('Inputs', 'B7'), 120000);
  assert.equal(adapter.writes.length, 0);
});

test('formula destinations are rejected and formula-control text is stored literally', () => {
  const payload = buildMockExchangePayload();
  payload.metadata.well = '=1+1';
  const textMap = [{
    pointer: '/metadata/well', direction: 'Input', sheet: 'Inputs', address: 'B5', shape: 'Scalar',
    unitSource: 'text', dimension: 'text', dataType: 'string', required: true, writable: true,
  }];
  const formulaAdapter = mockAdapter({ 'Inputs!B5': 'old' }, { formulas: ['Inputs!B5'] });
  const rejected = importMappedPayload(formulaAdapter, payload, textMap, []);
  assert.equal(rejected.ok, false);
  assert.match(rejected.errors.join('\n'), /formula destination/i);

  const adapter = mockAdapter({ 'Inputs!B5': 'old' });
  assert.equal(importMappedPayload(adapter, payload, textMap, []).ok, true);
  assert.equal(adapter.read('Inputs', 'B5'), "'=1+1");
});

test('table mappings import and export by stable ID while retaining unknown records', () => {
  const payload = buildMockExchangePayload();
  payload.bhaComponents = [
    { id: 'a', length: { value: 10, unit: 'ft' }, vendor: 'retain-a' },
    { id: 'b', length: { value: 20, unit: 'ft' } },
    { id: 'external', length: { value: 30, unit: 'ft' }, vendor: 'retain-record' },
  ];
  const map = [{
    pointer: '/bhaComponents/*/length', direction: 'Both', sheet: 'Inputs', address: 'B2:B3', shape: 'Table',
    valueColumn: 'B', idColumn: 'A', idPointer: 'id', capacity: 2, unitSource: 'm', dimension: 'length',
    dataType: 'number', required: true, writable: true,
  }];
  const adapter = mockAdapter({ 'Inputs!A2:A3': [['b'], ['a']], 'Inputs!B2:B3': [[0], [0]] });
  const imported = importMappedPayload(adapter, payload, map, []);
  assert.equal(imported.ok, true);
  assert.deepEqual(adapter.read('Inputs', 'B2:B3'), [[6.096], [3.048]]);

  adapter.write('Inputs', 'B2:B3', [[7], [3.048]]);
  const exported = exportMappedPayload(adapter, payload, map, imported.state, { displayUnits: { length: 'ft' } });
  assert.equal(exported.ok, true);
  assert.deepEqual(exported.payload.bhaComponents.map(({ id }) => id), ['a', 'b', 'external']);
  assert.deepEqual(exported.payload.bhaComponents[0], { id: 'a', length: { value: 10, unit: 'ft' }, vendor: 'retain-a' });
  assert.ok(Math.abs(exported.payload.bhaComponents[1].length.value - (7 / 0.3048)) < 1e-12);
  assert.equal(exported.payload.bhaComponents[2].vendor, 'retain-record');
});

test('duplicate table identifiers and matrix dimension mismatches are rejected before writes', () => {
  const payload = buildMockExchangePayload();
  payload.bhaComponents = [{ id: 'a', length: { value: 1, unit: 'm' } }];
  const map = [{
    pointer: '/bhaComponents/*/length', direction: 'Input', sheet: 'Inputs', address: 'B2:B3', shape: 'Table',
    valueColumn: 'B', idColumn: 'A', idPointer: 'id', capacity: 2, unitSource: 'm', dimension: 'length',
    dataType: 'number', required: true, writable: true,
  }];
  const duplicate = mockAdapter({ 'Inputs!A2:A3': [['a'], ['a']], 'Inputs!B2:B3': [[0], [0]] });
  const duplicateResult = importMappedPayload(duplicate, payload, map, []);
  assert.equal(duplicateResult.ok, false);
  assert.match(duplicateResult.errors.join('\n'), /duplicate.*a/i);
  assert.equal(duplicate.writes.length, 0);

  const malformed = mockAdapter({ 'Inputs!A2:A3': [['a']], 'Inputs!B2:B3': [[0], [0]] });
  const malformedResult = importMappedPayload(malformed, payload, map, []);
  assert.equal(malformedResult.ok, false);
  assert.match(malformedResult.errors.join('\n'), /dimension mismatch/i);
  assert.equal(malformed.writes.length, 0);
});

import test from 'node:test';
import assert from 'node:assert/strict';

import { buildApi7gWorkbook } from '../src/build_api7g.mjs';
import { buildBhaWorkbook } from '../src/build_bha.mjs';
import { buildDirectionalWorkbook } from '../src/build_directional.mjs';
import { buildHydraulicsWorkbook } from '../src/build_hydraulics.mjs';
import { buildTorqueDragWorkbook } from '../src/build_torque_drag.mjs';
import { DISPLAY_NUMBER_FORMAT } from '../src/common.mjs';
import { UNIT_ROWS } from '../src/common.mjs';
import { DIRECTIONAL_CAPACITIES } from '../src/directional_contract.mjs';
import { buildMockExchangePayload } from '../src/exchange/build_mock_payload.mjs';
import { fromSi, toSi, UNIT_REGISTRY } from '../src/exchange/schema_contract.mjs';
import { WORKBOOK_MAPS } from '../src/exchange/workbook_maps.mjs';

const EXCHANGE_SHEETS = ['Exchange Map', 'Exchange State', 'Exchange Buffer'];
const DIRECTIONAL_SHEETS = ['Summary', 'Inputs', 'Plan', 'Survey', 'Targets', 'Slide Performance', 'Formation Tops', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEETS];
const EXPECTED_SHEETS = {
  api7g: ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'Tubular Catalog', 'Load Cases', 'Section Detail', 'Strength Charts', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEETS],
  hydraulics: ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'Fluid Model', 'Flow Path', 'Nozzle Cases', 'Pressure Profile', 'Hydraulics Charts', 'Flow Cases', 'Hydraulics Dashboard', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEETS],
  torqueDrag: ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'Wellbore', 'Drillstring', 'Operation Cases', 'ALL', 'PUW', 'SOW', 'BKR', 'SLD', 'ROT', 'DRLG', 'Operation Charts', 'Observed Data', 'Engineering Dashboard', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEETS],
  bha: ['Summary', 'Inputs', 'Survey', 'Results', 'Graphs', 'BHA Assembly', 'Vibration Modes', 'Bending Response', 'BHA Geometry View', 'Tendency Matrix', 'Polar Plot', 'Rust Engine', 'Rust Engine Results', 'Rust Calc', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEETS],
  directional: DIRECTIONAL_SHEETS,
};

const BUILDERS = {
  api7g: buildApi7gWorkbook,
  hydraulics: buildHydraulicsWorkbook,
  torqueDrag: buildTorqueDragWorkbook,
  bha: buildBhaWorkbook,
  directional: buildDirectionalWorkbook,
};

test('every workbook exposes complete exchange infrastructure in the established order', () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    assert.deepEqual(workbook.worksheets.items.map(({ name }) => name), EXPECTED_SHEETS[kind], kind);
    for (const name of EXCHANGE_SHEETS) assert.ok(workbook.worksheets.getItem(name), `${kind}: ${name}`);
  }
});

test('mapping records are immutable, complete, unit-valid, and formula outputs are export-only', () => {
  const requiredKeys = ['pointer', 'direction', 'sheet', 'address', 'shape', 'unitSource', 'dimension', 'dataType', 'required', 'writable'];
  assert.deepEqual(Object.keys(WORKBOOK_MAPS).sort(), Object.keys(BUILDERS).sort());
  assert.ok(Object.isFrozen(WORKBOOK_MAPS));
  for (const [kind, mappings] of Object.entries(WORKBOOK_MAPS)) {
    assert.ok(Object.isFrozen(mappings), kind);
    assert.ok(mappings.some(({ direction }) => direction === 'Input' || direction === 'Both'), `${kind}: no inputs`);
    assert.ok(mappings.some(({ direction }) => direction === 'Output' || direction === 'Both'), `${kind}: no outputs`);
    for (const mapping of mappings) {
      for (const key of requiredKeys) assert.ok(Object.hasOwn(mapping, key), `${kind} ${mapping.pointer}: ${key}`);
      assert.ok(Object.isFrozen(mapping), `${kind} ${mapping.pointer}`);
      assert.match(mapping.pointer, /^\//, `${kind} ${mapping.pointer}`);
      assert.ok(['Input', 'Output', 'Both'].includes(mapping.direction), `${kind} ${mapping.pointer}`);
      if (mapping.shape === 'Scalar') assert.doesNotMatch(mapping.pointer, /\/\*\//, `${kind} ambiguous scalar ${mapping.pointer}`);
      if (mapping.direction === 'Output') assert.equal(mapping.writable, false, `${kind} ${mapping.pointer}`);
      if (mapping.writable) assert.notEqual(mapping.direction, 'Output', `${kind} ${mapping.pointer}`);
      if (mapping.unitSource && !mapping.unitSource.includes('!') && mapping.unitSource !== 'text') {
        const definition = UNIT_REGISTRY[mapping.unitSource];
        assert.ok([definition?.dimension, ...(definition?.dimensions ?? [])].includes(mapping.dimension), `${kind} ${mapping.pointer}`);
      }
    }
  }
});

test('every writable destination is a literal input and every mapped formula destination is non-writable', () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    for (const mapping of WORKBOOK_MAPS[kind]) {
      const formulas = workbook.worksheets.getItem(mapping.sheet).getRange(mapping.address).formulas.flat().filter(Boolean);
      if (mapping.writable) assert.deepEqual(formulas, [], `${kind} ${mapping.sheet}!${mapping.address}`);
      if (formulas.length > 0) assert.equal(mapping.writable, false, `${kind} ${mapping.sheet}!${mapping.address}`);
    }
  }
});

test('the shared inputs and surfaced decisions have declarative destinations', () => {
  const expected = {
    api7g: ['/analyses/api7g/sections/*/length', '/analyses/api7g/controls/surfaceTorque', '/analyses/api7g/results/*/combinedUtilisation', '/analyses/api7g/summary/status'],
    hydraulics: ['/operatingPoint/flowRate', '/analyses/hydraulics/rheology/model', '/analyses/hydraulics/rheology/highShearFlowIndex', '/analyses/hydraulics/flowPath/*/hydraulicDiameter', '/pumpNozzle/nozzles/*/diameter', '/analyses/hydraulics/summary/recommendedNozzleDiameter'],
    torqueDrag: ['/operatingPoint/wob', '/trajectory/survey/*/md', '/analyses/torqueDrag/results/*/rotateTorque', '/analyses/torqueDrag/summary/governingDepth'],
    bha: ['/operatingPoint/rotarySpeed', '/bhaComponents/*/supportFactor', '/analyses/bha/results/*/bendingStress', '/analyses/bha/summary/vibrationScreening'],
    directional: ['/metadata/well', '/trajectory/plan/*/md', '/trajectory/survey/*/azimuth', '/trajectory/targets/*/verticalTolerance', '/trajectory/slideIntervals/*/commandedToolface', '/trajectory/formationTops/*/actualPickMd', '/analyses/directional/summary/decision'],
  };
  for (const [kind, pointers] of Object.entries(expected)) {
    const actual = new Set(WORKBOOK_MAPS[kind].map(({ pointer }) => pointer));
    for (const pointer of pointers) assert.ok(actual.has(pointer), `${kind}: ${pointer}`);
  }
});

test('table mappings declare stable identifiers and explicit bounded capacities', () => {
  for (const [kind, mappings] of Object.entries(WORKBOOK_MAPS)) {
    for (const mapping of mappings.filter(({ shape }) => shape === 'Table')) {
      assert.equal(typeof mapping.idColumn, 'string', `${kind} ${mapping.pointer}`);
      assert.match(mapping.idColumn, /^[A-Z]+$/, `${kind} ${mapping.pointer}: ID column must be a real worksheet column`);
      assert.equal(mapping.idPointer, 'id', `${kind} ${mapping.pointer}`);
      assert.ok(Number.isInteger(mapping.capacity) && mapping.capacity > 0, `${kind} ${mapping.pointer}`);
      assert.equal(typeof mapping.valueColumn, 'string', `${kind} ${mapping.pointer}`);
    }
  }
  const directional = WORKBOOK_MAPS.directional;
  for (const [name, capacity] of Object.entries(DIRECTIONAL_CAPACITIES)) {
    const pointerName = name === 'slideIntervals' ? 'slideIntervals' : name;
    const records = directional.filter(({ pointer, shape }) => shape === 'Table' && pointer.startsWith(`/trajectory/${pointerName}/*/`));
    assert.ok(records.length > 0, name);
    assert.ok(records.every((record) => record.capacity === capacity), name);
  }
});

function pointerValue(root, pointer) {
  return pointer.split('/').slice(1).reduce((value, token) => value?.[token.replaceAll('~1', '/').replaceAll('~0', '~')], root);
}

function tableRows(mapping) {
  const match = mapping.address.match(/^[A-Z]+(\d+):[A-Z]+(\d+)$/);
  assert.ok(match, mapping.address);
  return [Number(match[1]), Number(match[2])];
}

test('required writable mappings resolve against the canonical mock by real stable IDs', () => {
  const payload = buildMockExchangePayload();
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    for (const mapping of WORKBOOK_MAPS[kind].filter(({ writable, required }) => writable && required)) {
      if (mapping.shape === 'Scalar') {
        assert.doesNotMatch(mapping.pointer, /\/\*\//, `${kind} ambiguous scalar ${mapping.pointer}`);
        assert.notEqual(pointerValue(payload, mapping.pointer), undefined, `${kind} ${mapping.pointer}`);
        continue;
      }
      const [arrayPointer, field] = mapping.pointer.split('/*/');
      const records = pointerValue(payload, arrayPointer);
      assert.ok(Array.isArray(records), `${kind} ${arrayPointer}`);
      const [first, last] = tableRows(mapping);
      const ids = workbook.worksheets.getItem(mapping.sheet).getRange(`${mapping.idColumn}${first}:${mapping.idColumn}${last}`).values.flat().filter((value) => value !== null && value !== '');
      assert.deepEqual(ids, records.map(({ id }) => id), `${kind} ${mapping.sheet} ${mapping.pointer}`);
      assert.equal(new Set(ids).size, ids.length, `${kind} duplicate IDs in ${mapping.sheet}`);
      for (const record of records) assert.notEqual(record[field], undefined, `${kind} ${mapping.pointer} ${record.id}`);
    }
  }
});

test('table mapping groups cannot conflict on stable-ID metadata', () => {
  for (const [kind, mappings] of Object.entries(WORKBOOK_MAPS)) {
    const groups = new Map();
    for (const mapping of mappings.filter(({ shape }) => shape === 'Table')) {
      const key = `${mapping.sheet}!${mapping.address.replace(/^[A-Z]+/, '').replace(/:[A-Z]+/, ':')}`;
      const prior = groups.get(key);
      const metadata = [mapping.idColumn, mapping.idPointer, mapping.capacity];
      if (prior) assert.deepEqual(metadata, prior, `${kind} ${key}`);
      else groups.set(key, metadata);
    }
  }
});

test('formula result tables retain the exact canonical record IDs and semantic output IDs stay unique', () => {
  const payload = buildMockExchangePayload();
  const lineages = [
    ['api7g', '/analyses/api7g/sections', 'Inputs', 'I', 6, 'Results', 'I', 6],
    ['torqueDrag', '/trajectory/survey', 'Survey', 'E', 6, 'Results', 'K', 6],
    ['bha', '/bhaComponents', 'Inputs', 'I', 6, 'Results', 'F', 6],
    ['directional', '/trajectory/survey', 'Survey', 'Z', 7, 'Results', 'M', 26],
  ];
  for (const [kind, pointer, sourceSheet, sourceColumn, sourceRow, targetSheet, targetColumn, targetRow] of lineages) {
    const workbook = BUILDERS[kind]();
    const canonicalIds = pointerValue(payload, pointer).map(({ id }) => id);
    const sourceIds = workbook.worksheets.getItem(sourceSheet).getRange(`${sourceColumn}${sourceRow}:${sourceColumn}${sourceRow + canonicalIds.length - 1}`).values.flat();
    assert.deepEqual(sourceIds, canonicalIds, `${kind} source ID drift`);
    const targetFormulas = workbook.worksheets.getItem(targetSheet).getRange(`${targetColumn}${targetRow}:${targetColumn}${targetRow + canonicalIds.length - 1}`).formulas.flat();
    const expectedFormulas = canonicalIds.map((_, index) => kind === 'directional'
      ? `=IF(${sourceSheet}!${sourceColumn}${sourceRow + index}="","",${sourceSheet}!${sourceColumn}${sourceRow + index})`
      : `=${sourceSheet}!${sourceColumn}${sourceRow + index}`);
    assert.deepEqual(targetFormulas, expectedFormulas, `${kind} result ID drift`);
  }

  const bhaIds = BUILDERS.bha().worksheets.getItem('Results').getRange('N6:N17').values.flat();
  assert.equal(new Set(bhaIds).size, bhaIds.length);
  assert.ok(bhaIds.every((id) => /^toolface-\d{3}deg$/.test(id)), bhaIds.join(', '));
  const checkIds = BUILDERS.directional().worksheets.getItem('Checks').getRange('F6:F25').values.flat();
  assert.equal(new Set(checkIds).size, checkIds.length);
  assert.ok(checkIds.every((id) => /^check-(?!\d+$)[a-z0-9-]+$/.test(id)), checkIds.join(', '));
});

test('directional result IDs cover all 500 survey rows without manufacturing IDs for blank rows', () => {
  const workbook = buildDirectionalWorkbook();
  const survey = workbook.worksheets.getItem('Survey');
  const results = workbook.worksheets.getItem('Results');
  const formulas = results.getRange('M26:M525').formulas.flat();
  assert.equal(formulas.length, 500);
  assert.ok(formulas.every(Boolean), 'every mapped result row needs an ID derivation formula');
  assert.match(formulas[60], /^=IF\(Survey!Z67="","",Survey!Z67\)$/);
  assert.match(formulas[499], /^=IF\(Survey!Z506="","",Survey!Z506\)$/);
  assert.equal(new Set(formulas).size, 500, 'each result ID must derive from its own survey row');
  assert.deepEqual(results.getRange('M86:M87').values, [[''], ['']], 'blank capacity rows must not manufacture IDs');

  survey.getRange('Z67').values = [['survey-added-060']];
  survey.getRange('Z68').values = [['survey-added-060']];
  assert.deepEqual(results.getRange('M86:M87').values, [['survey-added-060'], ['survey-added-060']], 'result IDs must preserve added and duplicate source IDs verbatim');
  const populatedIds = survey.getRange('Z7:Z506').values.flat().filter(Boolean);
  const duplicateIds = [...new Set(populatedIds.filter((id, index) => populatedIds.indexOf(id) !== index))];
  assert.deepEqual(duplicateIds, ['survey-added-060'], 'duplicate IDs must remain detectable rather than being regenerated by row');
  assert.match(formulas[61], /Survey!Z68/);
});

test('display-converted outputs take units from the same selector used by their formulas', () => {
  const expected = [
    ['api7g', '/analyses/api7g/results/*/buoyedLoad', "Results!B4", "'Unit Map'!$I$14"],
    ['torqueDrag', '/analyses/torqueDrag/summary/peakPoohHookload', 'Results!B4', 'Results!B'],
    ['torqueDrag', '/analyses/torqueDrag/summary/lowestRihAxialLoad', 'Results!C4', 'Results!C'],
    ['torqueDrag', '/analyses/torqueDrag/summary/governingDepth', 'Results!A4', 'Results!A'],
  ];
  for (const [kind, pointer, unitSource, formulaFragment] of expected) {
    const mapping = WORKBOOK_MAPS[kind].find((candidate) => candidate.pointer === pointer);
    assert.equal(mapping?.unitSource, unitSource, `${kind} ${pointer}`);
    const workbook = BUILDERS[kind]();
    assert.match(workbook.worksheets.getItem(mapping.sheet).getRange(mapping.address).formulas.flat().join('\n'), new RegExp(formulaFragment.replaceAll('$', '\\$')), `${kind} ${pointer}`);
  }
});

test('every display-converted output resolves a populated unit label and never declares a fixed wire unit', () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    for (const mapping of WORKBOOK_MAPS[kind].filter(({ direction }) => direction === 'Output' || direction === 'Both')) {
      const formulaText = workbook.worksheets.getItem(mapping.sheet).getRange(mapping.address).formulas.flat().filter(Boolean).join('\n');
      const displayRows = [...formulaText.matchAll(/'Unit Map'!\$I\$(\d+)/g)].map((match) => match[1]);
      if (mapping.dataType === 'number' && mapping.dimension !== 'unitless' && displayRows.length > 0) assert.match(mapping.unitSource, /!/, `${kind} ${mapping.pointer} hard-codes a unit for display-converted values`);
      if (!mapping.unitSource.includes('!')) continue;
      const separator = mapping.unitSource.indexOf('!');
      const unitSheet = mapping.unitSource.slice(0, separator).replace(/^'|'$/g, '');
      const unitAddress = mapping.unitSource.slice(separator + 1);
      const unitCell = workbook.worksheets.getItem(unitSheet).getRange(unitAddress);
      const unitEvidence = [...unitCell.values.flat(), ...unitCell.formulas.flat()].filter(Boolean).join('\n');
      assert.notEqual(unitEvidence, '', `${kind} ${mapping.pointer} empty unit source ${mapping.unitSource}`);
      const labelRows = [...unitEvidence.matchAll(/'Unit Map'!\$H\$(\d+)/g)].map((match) => match[1]);
      if (displayRows.length > 0 && labelRows.length > 0) {
        assert.deepEqual(new Set(displayRows), new Set(labelRows), `${kind} ${mapping.pointer} display factor/unit-label drift`);
      }
    }
  }
});

test('every Unit Map choice is registered for its engineering dimension', () => {
  const dimension = { Length: 'length', Diameter: 'diameter', Area: 'area', Volume: 'volume', 'Flow rate': 'flowRate', Density: 'density', Force: 'force', Pressure: 'pressure', Torque: 'torque', Stress: 'stress', Angle: 'angle', Speed: 'speed', 'Angular gradient': 'angularGradient' };
  for (const row of UNIT_ROWS) {
    for (const unit of [row.siUnit, row.imperialUnit, row.mixedUnit]) {
      const definition = UNIT_REGISTRY[unit];
      assert.ok(definition, `${row.domain}: ${unit}`);
      assert.ok([definition.dimension, ...(definition.dimensions ?? [])].includes(dimension[row.domain]), `${row.domain}: ${unit}`);
      const original = 123.456;
      const restored = fromSi(toSi({ value: original, unit }, dimension[row.domain]), unit, dimension[row.domain]);
      assert.ok(Math.abs(restored - original) < 1e-9, `${row.domain}: ${unit} is not reversible`);
    }
  }
  for (const unit of ['N-m', 'N*m', 'kN-m', 'kN*m']) {
    assert.equal(fromSi(toSi({ value: 42, unit }, 'torque'), unit, 'torque'), 42, unit);
  }
  for (const dimensionName of ['length', 'diameter']) {
    for (const unit of ['m', 'mm', 'in']) {
      assert.equal(fromSi(toSi({ value: 42, unit }, dimensionName), unit, dimensionName), 42, `${dimensionName}: ${unit}`);
    }
  }
});

test('exchange sheets expose auditable state and bounded buffer contracts', () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    const map = workbook.worksheets.getItem('Exchange Map');
    const state = workbook.worksheets.getItem('Exchange State');
    const buffer = workbook.worksheets.getItem('Exchange Buffer');
    assert.deepEqual(map.getRange('A5:M5').values[0], ['JSON Pointer', 'Direction', 'Sheet', 'Address', 'Shape', 'Value column', 'Stable ID column', 'Row capacity', 'Unit source', 'Dimension', 'Data type', 'Required', 'Writable'], kind);
    assert.equal(map.getRange('A6').values[0][0], WORKBOOK_MAPS[kind][0].pointer, kind);
    assert.equal(map.protection?.protected, true, kind);
    assert.deepEqual(map.protection?.editableRanges, ['A3:M3'], kind);
    assert.deepEqual(state.getRange('A5:F5').values[0], ['Pointer', 'Original value', 'Original unit', 'Canonical value', 'Destination', 'Imported at'], kind);
    assert.equal(state.visibility, 'hidden', kind);
    assert.deepEqual(buffer.getRange('A5:A8').values.flat(), ['Payload', 'Action', 'Status', 'Diagnostics'], kind);
    assert.equal(buffer.getRange('B5').format.wrapText, true, kind);
    assert.ok(buffer.getRange('B:B').format.columnWidth <= 80, kind);
    for (const sheetName of EXCHANGE_SHEETS) {
      assert.equal(workbook.worksheets.getItem(sheetName).getUsedRange().format.numberFormat, DISPLAY_NUMBER_FORMAT, `${kind}: ${sheetName}`);
    }
  }
});

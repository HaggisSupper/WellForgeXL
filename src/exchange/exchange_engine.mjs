import { fromSi, UNIT_REGISTRY, toSi } from './schema_contract.mjs';
import { validateExchangePayload } from './schema_validator.mjs';
import { WORKBOOK_MAPS } from './workbook_maps.mjs';
import { getPointer, mergeByStableId, setPointer } from './json_pointer.mjs';

const RELATIVE_TOLERANCE = 1e-12;
const NUMERIC_TYPES = new Set(['number', 'integer', 'date']);
const STRING_TYPES = new Set(['string', 'status']);
const clone = (value) => value === undefined ? undefined : structuredClone(value);
const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const destination = (mapping) => `${mapping.sheet}!${mapping.address}`;

function failure(errors, extra = {}) {
  return { ok: false, errors: errors.map((error) => error instanceof Error ? error.message : String(error)), ...extra };
}

function resolveMappings(map) {
  if (Array.isArray(map)) return map;
  if (typeof map === 'string' && WORKBOOK_MAPS[map]) return WORKBOOK_MAPS[map];
  throw new Error('map must be a mapping array or known workbook map name');
}

function parsePayload(payload) {
  if (typeof payload !== 'string') return clone(payload);
  return JSON.parse(payload);
}

function splitTablePointer(pointer) {
  const marker = '/*/';
  const index = pointer.indexOf(marker);
  if (index < 0) throw new Error(`Table mapping requires a wildcard pointer: ${pointer}`);
  return { arrayPointer: pointer.slice(0, index), fieldPointer: `/${pointer.slice(index + marker.length)}` };
}

function parseVerticalRange(mapping) {
  const match = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/.exec(mapping.address);
  if (!match || match[1] !== match[3]) throw new Error(`${destination(mapping)} must be a single-column table range`);
  const first = Number(match[2]);
  const last = Number(match[4]);
  const count = last - first + 1;
  if (!Number.isInteger(mapping.capacity) || mapping.capacity !== count) {
    throw new Error(`${destination(mapping)} dimension mismatch: range has ${count} rows but capacity is ${mapping.capacity}`);
  }
  return { first, last, count, idAddress: `${mapping.idColumn}${first}:${mapping.idColumn}${last}` };
}

function asColumn(value, count, label) {
  if (!Array.isArray(value) || value.length !== count || value.some((row) => !Array.isArray(row) || row.length !== 1)) {
    throw new Error(`${label} dimension mismatch: expected ${count}x1 matrix`);
  }
  return value.map(([cell]) => cell);
}

function assertUniqueIds(ids, label) {
  const seen = new Set();
  for (const id of ids.filter((value) => value !== '' && value !== null && value !== undefined)) {
    if (typeof id !== 'string' || id.length === 0) throw new Error(`${label} contains a non-string stable identifier`);
    if (seen.has(id)) throw new Error(`${label} has duplicate identifier ${id}`);
    seen.add(id);
  }
}

function resolveWorkbookUnit(adapter, mapping) {
  if (mapping.unitSource === 'text') return 'text';
  if (!mapping.unitSource?.includes('!')) return mapping.unitSource;
  const separator = mapping.unitSource.indexOf('!');
  const sheet = mapping.unitSource.slice(0, separator).replace(/^'|'$/g, '');
  const address = mapping.unitSource.slice(separator + 1).replaceAll('$', '');
  const value = adapter.read(sheet, address);
  const unit = Array.isArray(value) ? value?.[0]?.[0] : value;
  if (typeof unit !== 'string' || unit.length === 0) throw new Error(`${mapping.pointer} has no workbook unit at ${mapping.unitSource}`);
  return unit;
}

function canonicalUnit(dimension) {
  const entries = Object.entries(UNIT_REGISTRY);
  const matches = ([, definition]) => [definition.dimension, ...(definition.dimensions ?? [])].includes(dimension);
  return entries.find(([, definition]) => definition.dimension === dimension && definition.toSiMultiplier === 1 && definition.toSiOffset === 0)?.[0]
    ?? entries.find((entry) => matches(entry) && entry[1].toSiMultiplier === 1 && entry[1].toSiOffset === 0)?.[0]
    ?? entries.find((entry) => matches(entry))?.[0]
    ?? (() => { throw new Error(`No registered unit for ${dimension}`); })();
}

function literalText(value) {
  return typeof value === 'string' && /^[=+\-@\t\r]/.test(value) ? `'${value}` : value;
}

function importValue(adapter, mapping, value) {
  if (NUMERIC_TYPES.has(mapping.dataType)) {
    if (!isObject(value) || !Number.isFinite(value.value) || typeof value.unit !== 'string') {
      throw new Error(`${mapping.pointer} must be a finite quantity`);
    }
    if (mapping.dataType === 'integer' && !Number.isInteger(value.value)) throw new Error(`${mapping.pointer} must be an integer quantity`);
    const canonicalValue = toSi(value, mapping.dimension);
    if (!Number.isFinite(canonicalValue)) throw new Error(`${mapping.pointer} converts to a non-finite value`);
    const workbookUnit = resolveWorkbookUnit(adapter, mapping);
    const cellValue = fromSi(canonicalValue, workbookUnit, mapping.dimension);
    if (!Number.isFinite(cellValue)) throw new Error(`${mapping.pointer} converts to a non-finite destination value`);
    return { cellValue, canonicalValue, originalQuantity: clone(value) };
  }
  if (STRING_TYPES.has(mapping.dataType)) {
    if (typeof value !== 'string') throw new Error(`${mapping.pointer} must be a string`);
    return { cellValue: literalText(value), canonicalValue: value };
  }
  if (String(mapping.dataType).toLowerCase() === 'boolean') {
    if (typeof value !== 'boolean') throw new Error(`${mapping.pointer} must be a boolean`);
    return { cellValue: value, canonicalValue: value };
  }
  throw new Error(`${mapping.pointer} has unsupported data type ${mapping.dataType}`);
}

function stateEntry(mapping, conversion, recordId = undefined) {
  const quantity = conversion.originalQuantity;
  return {
    pointer: mapping.pointer,
    recordId,
    originalValue: quantity?.value ?? conversion.canonicalValue,
    originalUnit: quantity?.unit ?? '',
    originalQuantity: clone(quantity),
    canonicalValue: conversion.canonicalValue,
    destination: destination(mapping),
    importedAt: new Date().toISOString(),
  };
}

function stateKey(pointer, destinationName, recordId) {
  return `${pointer}\u0000${destinationName}\u0000${recordId ?? ''}`;
}

function stateIndex(state) {
  return new Map((Array.isArray(state) ? state : []).filter(isObject).map((entry) => [stateKey(entry.pointer, entry.destination, entry.recordId), entry]));
}

function planScalarImport(adapter, payload, mapping) {
  const value = getPointer(payload, mapping.pointer);
  if (value === undefined) {
    if (mapping.required) throw new Error(`${mapping.pointer} is required`);
    return null;
  }
  const conversion = importValue(adapter, mapping, value);
  return { mapping, value: conversion.cellValue, states: [stateEntry(mapping, conversion)] };
}

function tableRowsForImport(adapter, payload, mapping) {
  const range = parseVerticalRange(mapping);
  const currentIds = asColumn(adapter.read(mapping.sheet, range.idAddress), range.count, `${mapping.sheet}!${range.idAddress}`);
  assertUniqueIds(currentIds, `${mapping.sheet}!${range.idAddress}`);
  const { arrayPointer, fieldPointer } = splitTablePointer(mapping.pointer);
  const records = getPointer(payload, arrayPointer);
  if (records === undefined && !mapping.required) return null;
  if (!Array.isArray(records)) throw new Error(`${arrayPointer} must be an array`);
  const sourceIds = records.map((record) => record?.[mapping.idPointer ?? 'id']);
  assertUniqueIds(sourceIds, arrayPointer);
  if (sourceIds.some((id) => typeof id !== 'string' || id.length === 0)) throw new Error(`${arrayPointer} records require stable identifiers`);
  const byId = new Map(records.map((record) => [record[mapping.idPointer ?? 'id'], record]));
  const assigned = new Set(currentIds.filter(Boolean));
  const unassigned = records.filter((record) => !assigned.has(record[mapping.idPointer ?? 'id']));
  const rowIds = currentIds.map((id) => id || unassigned.shift()?.[mapping.idPointer ?? 'id'] || '');
  return { range, records, byId, rowIds, fieldPointer };
}

function planTableImport(adapter, payload, mapping) {
  const table = tableRowsForImport(adapter, payload, mapping);
  if (!table) return null;
  const values = [];
  const states = [];
  const existing = asColumn(adapter.read(mapping.sheet, mapping.address), table.range.count, destination(mapping));
  for (let index = 0; index < table.rowIds.length; index += 1) {
    const id = table.rowIds[index];
    const record = table.byId.get(id);
    if (!record) {
      values.push([existing[index] ?? '']);
      continue;
    }
    const value = getPointer(record, table.fieldPointer);
    if (value === undefined) {
      if (mapping.required) throw new Error(`${mapping.pointer} is required for identifier ${id}`);
      values.push([existing[index] ?? '']);
      continue;
    }
    const conversion = importValue(adapter, mapping, value);
    values.push([conversion.cellValue]);
    states.push(stateEntry(mapping, conversion, id));
  }
  return { mapping, value: values, states };
}

function isWritableInput(mapping) {
  return mapping.writable === true && (mapping.direction === 'Input' || mapping.direction === 'Both');
}

export function importMappedPayload(adapter, payloadInput, map, state = []) {
  let payload;
  let mappings;
  try {
    payload = parsePayload(payloadInput);
    mappings = resolveMappings(map);
  } catch (error) {
    return failure([error], { state: clone(state) ?? [] });
  }
  const schema = validateExchangePayload(payload);
  if (!schema.valid) return failure(schema.errors, { state: clone(state) ?? [] });

  const plan = [];
  const errors = [];
  for (const mapping of mappings.filter(isWritableInput)) {
    try {
      if (adapter.isFormula(mapping.sheet, mapping.address)) throw new Error(`${destination(mapping)} is a formula destination`);
      const change = mapping.shape === 'Table'
        ? planTableImport(adapter, payload, mapping)
        : planScalarImport(adapter, payload, mapping);
      if (change) plan.push(change);
    } catch (error) {
      errors.push(error);
    }
  }
  if (errors.length > 0) return failure(errors, { state: clone(state) ?? [] });

  let captured;
  try {
    captured = plan.map(({ mapping }) => adapter.capture(mapping.sheet, mapping.address));
  } catch (error) {
    return failure([error], { state: clone(state) ?? [] });
  }
  try {
    for (const change of plan) adapter.write(change.mapping.sheet, change.mapping.address, change.value);
  } catch (error) {
    try {
      adapter.restore(captured);
    } catch (restoreError) {
      return failure([error, `Rollback failed: ${restoreError.message}`], { state: clone(state) ?? [] });
    }
    return failure([error], { state: clone(state) ?? [] });
  }

  const prior = stateIndex(state);
  for (const change of plan) {
    for (const entry of change.states) prior.set(stateKey(entry.pointer, entry.destination, entry.recordId), entry);
  }
  return { ok: true, errors: [], state: [...prior.values()], changes: plan.length };
}

function nearlyEqual(left, right) {
  if (Object.is(left, right)) return true;
  if (!Number.isFinite(left) || !Number.isFinite(right)) return false;
  return Math.abs(left - right) <= RELATIVE_TOLERANCE * Math.max(1, Math.abs(left), Math.abs(right));
}

function exportValue(adapter, mapping, cellValue, prior) {
  if (NUMERIC_TYPES.has(mapping.dataType)) {
    if (!Number.isFinite(cellValue)) throw new Error(`${mapping.pointer} contains a non-finite workbook value`);
    const workbookUnit = resolveWorkbookUnit(adapter, mapping);
    const canonicalValue = toSi({ value: cellValue, unit: workbookUnit }, mapping.dimension);
    if (prior?.originalQuantity && nearlyEqual(canonicalValue, prior.canonicalValue)) return clone(prior.originalQuantity);
    return { canonicalValue };
  }
  if (STRING_TYPES.has(mapping.dataType)) {
    if (typeof cellValue !== 'string') throw new Error(`${mapping.pointer} workbook value must be a string`);
    return cellValue.startsWith("'") && /^[=+\-@\t\r]/.test(cellValue.slice(1)) ? cellValue.slice(1) : cellValue;
  }
  if (String(mapping.dataType).toLowerCase() === 'boolean') {
    if (typeof cellValue !== 'boolean') throw new Error(`${mapping.pointer} workbook value must be a boolean`);
    return cellValue;
  }
  throw new Error(`${mapping.pointer} has unsupported data type ${mapping.dataType}`);
}

function quantityForExport(intermediate, mapping, options) {
  if (!isObject(intermediate) || !Object.hasOwn(intermediate, 'canonicalValue')) return intermediate;
  const preferred = options.displayUnits?.[mapping.dimension];
  const unit = preferred ?? canonicalUnit(mapping.dimension);
  const value = fromSi(intermediate.canonicalValue, unit, mapping.dimension);
  if (!Number.isFinite(value)) throw new Error(`${mapping.pointer} converts to a non-finite export value`);
  return { value, unit };
}

function shouldExport(mapping, options) {
  if (mapping.direction === 'Output' && options.includeResults === false) return false;
  return ['Input', 'Output', 'Both'].includes(mapping.direction);
}

function exportScalar(adapter, payload, mapping, states, options) {
  const cellValue = adapter.read(mapping.sheet, mapping.address);
  if (cellValue === undefined || cellValue === null || cellValue === '') {
    if (mapping.required) throw new Error(`${destination(mapping)} is required`);
    return payload;
  }
  const prior = states.get(stateKey(mapping.pointer, destination(mapping), undefined));
  const value = quantityForExport(exportValue(adapter, mapping, cellValue, prior), mapping, options);
  setPointer(payload, mapping.pointer, value);
  return payload;
}

function exportTable(adapter, payload, mapping, states, options) {
  const range = parseVerticalRange(mapping);
  const ids = asColumn(adapter.read(mapping.sheet, range.idAddress), range.count, `${mapping.sheet}!${range.idAddress}`);
  const values = asColumn(adapter.read(mapping.sheet, mapping.address), range.count, destination(mapping));
  assertUniqueIds(ids, `${mapping.sheet}!${range.idAddress}`);
  const { arrayPointer, fieldPointer } = splitTablePointer(mapping.pointer);
  const updates = [];
  for (let index = 0; index < ids.length; index += 1) {
    const id = ids[index];
    const cellValue = values[index];
    if (id === '' || id === null || id === undefined) continue;
    if (cellValue === '' || cellValue === null || cellValue === undefined) {
      if (mapping.required) throw new Error(`${mapping.pointer} is required for identifier ${id}`);
      continue;
    }
    const prior = states.get(stateKey(mapping.pointer, destination(mapping), id));
    const value = quantityForExport(exportValue(adapter, mapping, cellValue, prior), mapping, options);
    const record = { [mapping.idPointer ?? 'id']: id };
    setPointer(record, fieldPointer, value);
    updates.push(record);
  }
  const existing = getPointer(payload, arrayPointer);
  setPointer(payload, arrayPointer, mergeByStableId(Array.isArray(existing) ? existing : [], updates));
  return payload;
}

export function exportMappedPayload(adapter, existingPayloadInput, map, state = [], options = {}) {
  let payload;
  let mappings;
  try {
    payload = parsePayload(existingPayloadInput ?? {});
    if (!isObject(payload)) throw new Error('existing payload must be an object');
    mappings = resolveMappings(map);
  } catch (error) {
    return failure([error], { payload: clone(existingPayloadInput), state: clone(state) ?? [] });
  }
  const states = stateIndex(state);
  const errors = [];
  for (const mapping of mappings.filter((candidate) => shouldExport(candidate, options))) {
    try {
      if (mapping.shape === 'Table') exportTable(adapter, payload, mapping, states, options);
      else exportScalar(adapter, payload, mapping, states, options);
    } catch (error) {
      errors.push(error);
    }
  }
  if (errors.length > 0) return failure(errors, { payload: clone(existingPayloadInput), state: clone(state) ?? [] });
  return { ok: true, errors: [], payload, state: clone(state) ?? [] };
}

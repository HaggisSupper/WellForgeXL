import { SCHEMA_VERSION, UNIT_REGISTRY } from './schema_contract.mjs';

const SEMANTIC_VERSION = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:(?:0|[1-9]\d*)|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

const REQUIRED_TOP_LEVEL = [
  'schemaVersion', 'caseId', 'createdAt', 'producer', 'metadata', 'unitPreferences',
  'trajectory', 'holeSections', 'tubulars', 'bhaComponents', 'fluids', 'operatingPoint',
  'rigLimits', 'pumpNozzle', 'analyses', 'provenance', 'warnings',
];

const IDENTIFIED_ARRAY_PATHS = new Set([
  'trajectory.plan', 'trajectory.survey', 'trajectory.targets', 'trajectory.slideIntervals',
  'trajectory.formationTops', 'holeSections', 'tubulars', 'bhaComponents', 'fluids',
  'pumpNozzle.pumps', 'pumpNozzle.nozzles',
]);

const QUANTITY_KEYS = new Set(['value', 'unit', 'quality', 'source', 'timestamp', 'note']);
const RFC_3339_DATE_TIME = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/;

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function pathAt(path, key) {
  return path ? `${path}.${key}` : key;
}

function validateQuantity(value, path, errors) {
  if (!isObject(value)) {
    errors.push(`${path} must be an object`);
    return;
  }
  if (!Object.hasOwn(value, 'value')) errors.push(`${path}.value is required`);
  else if (!Number.isFinite(value.value)) errors.push(`${path}.value must be a finite number`);
  if (!Object.hasOwn(value, 'unit')) errors.push(`${path}.unit is required`);
  else if (typeof value.unit !== 'string') errors.push(`${path}.unit must be a string`);
  else if (!Object.hasOwn(UNIT_REGISTRY, value.unit)) errors.push(`${path}.unit has unknown unit ${value.unit}`);
  for (const key of Object.keys(value)) {
    if (!QUANTITY_KEYS.has(key)) errors.push(`${path}.${key} is not allowed`);
  }
  for (const key of ['quality', 'source', 'note']) {
    if (Object.hasOwn(value, key) && typeof value[key] !== 'string') errors.push(`${path}.${key} must be a string`);
  }
  if (Object.hasOwn(value, 'timestamp')) {
    if (typeof value.timestamp !== 'string') errors.push(`${path}.timestamp must be a string`);
    else if (!RFC_3339_DATE_TIME.test(value.timestamp)) errors.push(`${path}.timestamp must be an RFC 3339 date-time`);
  }
}

function validateQuantityObjects(value, path, errors) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => validateQuantityObjects(item, `${path}[${index}]`, errors));
    return;
  }
  if (!isObject(value)) return;
  if (Object.hasOwn(value, 'value') || Object.hasOwn(value, 'unit')) {
    validateQuantity(value, path, errors);
    return;
  }
  for (const [key, item] of Object.entries(value)) {
    validateQuantityObjects(item, pathAt(path, key), errors);
  }
}

function validateStableIdentifiers(payload, errors) {
  for (const dotPath of IDENTIFIED_ARRAY_PATHS) {
    const value = dotPath.split('.').reduce((current, key) => current?.[key], payload);
    if (!Array.isArray(value)) continue;
    const seen = new Set();
    value.forEach((record, index) => {
      const recordPath = `${dotPath}[${index}]`;
      if (!isObject(record)) {
        errors.push(`${recordPath} must be an object with an id`);
        return;
      }
      if (typeof record.id !== 'string' || record.id.length === 0) {
        errors.push(`${recordPath}.id is required`);
        return;
      }
      if (seen.has(record.id)) errors.push(`${dotPath} has duplicate identifier ${record.id}`);
      seen.add(record.id);
    });
  }
}

function validateDeclaredQuantities(payload, errors) {
  if (!isObject(payload.operatingPoint)) return;
  for (const [key, value] of Object.entries(payload.operatingPoint)) {
    validateQuantity(value, `operatingPoint.${key}`, errors);
  }
}

export function validateExchangePayload(payload) {
  const errors = [];
  if (!isObject(payload)) return { valid: false, errors: ['payload must be an object'] };

  for (const key of REQUIRED_TOP_LEVEL) {
    if (!Object.hasOwn(payload, key)) errors.push(`${key} is required`);
  }

  if (typeof payload.schemaVersion !== 'string') errors.push('schemaVersion must be a string');
  else if (!SEMANTIC_VERSION.test(payload.schemaVersion)) errors.push('schemaVersion must be a semantic version');
  else if (payload.schemaVersion.split('.')[0] !== SCHEMA_VERSION.split('.')[0]) errors.push('schemaVersion has unsupported major version');
  if (typeof payload.caseId !== 'string' || payload.caseId.length === 0) errors.push('caseId must be a non-empty string');
  if (typeof payload.createdAt !== 'string') errors.push('createdAt must be a string');

  for (const key of ['producer', 'metadata', 'unitPreferences', 'trajectory', 'operatingPoint', 'rigLimits', 'pumpNozzle', 'analyses', 'provenance']) {
    if (!isObject(payload[key])) errors.push(`${key} must be an object`);
  }
  for (const key of ['holeSections', 'tubulars', 'bhaComponents', 'fluids', 'warnings']) {
    if (!Array.isArray(payload[key])) errors.push(`${key} must be an array`);
  }

  if (isObject(payload.producer)) {
    if (typeof payload.producer.name !== 'string' || payload.producer.name.length === 0) errors.push('producer.name must be a non-empty string');
    if (typeof payload.producer.version !== 'string' || payload.producer.version.length === 0) errors.push('producer.version must be a non-empty string');
  }
  if (isObject(payload.trajectory)) {
    for (const key of ['plan', 'survey', 'targets', 'slideIntervals', 'formationTops']) {
      if (!Array.isArray(payload.trajectory[key])) errors.push(`trajectory.${key} must be an array`);
    }
  }
  if (isObject(payload.pumpNozzle)) {
    for (const key of ['pumps', 'nozzles']) {
      if (!Array.isArray(payload.pumpNozzle[key])) errors.push(`pumpNozzle.${key} must be an array`);
    }
  }
  if (isObject(payload.provenance) && !Array.isArray(payload.provenance.notes)) errors.push('provenance.notes must be an array');
  if (isObject(payload.unitPreferences)) {
    for (const [dimension, unit] of Object.entries(payload.unitPreferences)) {
      if (typeof unit !== 'string') errors.push(`unitPreferences.${dimension} must be a string`);
      else if (!Object.hasOwn(UNIT_REGISTRY, unit)) errors.push(`unitPreferences.${dimension} has unknown unit ${unit}`);
    }
  }

  validateDeclaredQuantities(payload, errors);
  validateStableIdentifiers(payload, errors);
  validateQuantityObjects(payload, '', errors);
  return { valid: errors.length === 0, errors };
}

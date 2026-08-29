import unitRegistry from '../../data/wellforge-unit-registry.json' with { type: 'json' };

export const SCHEMA_VERSION = '1.0.0';
export const UNIT_REGISTRY = Object.freeze(
  Object.fromEntries(Object.entries(unitRegistry).map(([symbol, definition]) => [
    symbol,
    Object.freeze({ ...definition }),
  ])),
);

export function quantity(value, unit, metadata = undefined) {
  return metadata === undefined ? { value, unit } : { value, unit, ...metadata };
}

export function toSi(quantity, expectedDimension) {
  const definition = UNIT_REGISTRY[quantity.unit];
  if (!definition || ![definition.dimension, ...(definition.dimensions ?? [])].includes(expectedDimension)) {
    throw new Error(`Unit ${quantity.unit} is not valid for ${expectedDimension}`);
  }
  return quantity.value * definition.toSiMultiplier + definition.toSiOffset;
}

export function fromSi(siValue, unit, expectedDimension) {
  const definition = UNIT_REGISTRY[unit];
  if (!definition || ![definition.dimension, ...(definition.dimensions ?? [])].includes(expectedDimension)) {
    throw new Error(`Unit ${unit} is not valid for ${expectedDimension}`);
  }
  return (siValue - definition.toSiOffset) / definition.toSiMultiplier;
}

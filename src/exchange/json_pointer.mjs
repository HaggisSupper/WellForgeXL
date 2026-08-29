const FORBIDDEN_TOKENS = new Set(['__proto__', 'prototype', 'constructor']);

function tokens(pointer) {
  if (pointer === '') return [];
  if (typeof pointer !== 'string' || !pointer.startsWith('/')) throw new Error(`Invalid JSON pointer: ${pointer}`);
  return pointer.slice(1).split('/').map((token) => {
    if (/~(?:[^01]|$)/.test(token)) throw new Error(`Invalid JSON pointer escape in ${pointer}`);
    const decoded = token.replaceAll('~1', '/').replaceAll('~0', '~');
    if (FORBIDDEN_TOKENS.has(decoded)) throw new Error(`Unsafe JSON pointer token: ${decoded}`);
    return decoded;
  });
}

const clone = (value) => value === undefined ? undefined : structuredClone(value);
const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const isArrayIndex = (token) => /^(?:0|[1-9]\d*)$/.test(token);

export function getPointer(document, pointer) {
  return tokens(pointer).reduce((value, token) => value?.[token], document);
}

export function setPointer(document, pointer, value) {
  const path = tokens(pointer);
  if (path.length === 0) return value;
  if (document === null || typeof document !== 'object') throw new Error('JSON pointer document must be an object or array');
  let target = document;
  for (let index = 0; index < path.length - 1; index += 1) {
    const token = path[index];
    const nextToken = path[index + 1];
    if (target[token] === undefined) target[token] = isArrayIndex(nextToken) ? [] : {};
    if (target[token] === null || typeof target[token] !== 'object') {
      throw new Error(`Cannot traverse non-container at /${path.slice(0, index + 1).join('/')}`);
    }
    target = target[token];
  }
  target[path.at(-1)] = clone(value);
  return document;
}

export function mergeByStableId(existing, updates) {
  if (updates === undefined) return clone(existing);
  if (Array.isArray(updates)) {
    if (!Array.isArray(existing)) return clone(updates);
    const identified = updates.every((record) => isObject(record) && typeof record.id === 'string' && record.id.length > 0);
    if (!identified) return clone(updates);
    const updateById = new Map(updates.map((record) => [record.id, record]));
    const merged = existing.map((record) => {
      if (!isObject(record) || typeof record.id !== 'string' || !updateById.has(record.id)) return clone(record);
      const update = updateById.get(record.id);
      updateById.delete(record.id);
      return mergeByStableId(record, update);
    });
    for (const update of updates) {
      if (updateById.has(update.id)) {
        merged.push(clone(update));
        updateById.delete(update.id);
      }
    }
    return merged;
  }
  if (isObject(updates)) {
    const merged = isObject(existing) ? clone(existing) : {};
    for (const [key, value] of Object.entries(updates)) merged[key] = mergeByStableId(merged[key], value);
    return merged;
  }
  return clone(updates);
}

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';

const sourcePath = new URL('../OfficeScripts/WellForgeJsonExchange.ts', import.meta.url);

test('Office Script exposes all actions without dynamic JSON execution', async () => {
  const source = await fs.readFile(sourcePath, 'utf8');

  assert.match(source, /action:\s*"Import"\s*\|\s*"Export"\s*\|\s*"Validate"/);
  assert.match(source, /JSON\.parse/);
  assert.doesNotMatch(source, /\beval\s*\(|new Function/);
  assert.doesNotMatch(source, /fetch\s*\(|XMLHttpRequest|require\s*\(|import\s+/);
});

test('Office Script reads declarative maps and protects unit state, stable IDs, and writes', async () => {
  const source = await fs.readFile(sourcePath, 'utf8');

  for (const required of ['Exchange Map', 'Exchange State', 'Exchange Buffer', 'JSON.parse', 'stable identifier', 'rollback', 'calculate(ExcelScript.CalculationType.full)']) {
    assert.match(source, new RegExp(required.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'), required);
  }
  assert.match(source, /toSi|fromSi/);
  assert.match(source, /getFormulas/);
  assert.match(source, /writeDiagnostics/);
});

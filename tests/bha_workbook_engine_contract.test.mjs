import test from 'node:test';
import assert from 'node:assert/strict';
import { buildBhaWorkbook } from '../src/build_bha.mjs';

test('BHA workbook exposes value-backed Rust result surfaces', async () => {
  const workbook = buildBhaWorkbook();
  const engine = workbook.worksheets.getItem('Rust Engine');
  const results = workbook.worksheets.getItem('Rust Engine Results');
  const calc = workbook.worksheets.getItem('Rust Calc');
  assert.equal(engine.getRange('B7').values[0][0], 'RUST REQUIRED — NO VBA FALLBACK');
  assert.equal(results.getRange('A5').values[0][0], 'Decision metric');
  assert.equal(calc.getRange('A5').values[0][0], 'MD m');
  assert.ok(Number(calc.getRange('A6').values[0][0]) >= 0);
  assert.equal(calc.getRange('A40').values[0][0], 'Frequency Hz');
  assert.equal(calc.getRange('E40').values[0][0], 'Order');
  assert.equal(calc.visibility, 'hidden');
  assert.ok(results.charts.items.length >= 3);
});

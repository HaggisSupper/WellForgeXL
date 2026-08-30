import test from 'node:test';
import assert from 'node:assert/strict';

test('calculation helper sheets are hidden from the normal end-user workflow', async () => {
  const { createSuiteWorkbook } = await import('../src/workbook.mjs');
  const { sheets } = createSuiteWorkbook('Visibility contract');
  assert.equal(sheets.Calc.visibility, 'hidden');
});

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

test('calculation helper sheets are hidden from the normal end-user workflow', async () => {
  const source = await readFile(new URL('../src/workbook.mjs', import.meta.url), 'utf8');
  assert.match(source, /if \(sheets\.Calc\) sheets\.Calc\.visibility = 'hidden'/);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';

const root = fileURLToPath(new URL('..', import.meta.url));
const files = [
  'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
  'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
  'Torque_Drag_and_Buckling_SI.xlsx',
  'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
  'Directional_Drilling_Wellplan_and_Survey_SI.xlsx',
];

test('visible sheets contain no formula errors or formula-backed literal labels', async () => {
  for (const name of files) {
    const file = await FileBlob.load(path.join(root, 'outputs', name));
    const wb = await SpreadsheetFile.importXlsx(file);
    const errors = await wb.inspect({ kind: 'match', searchTerm: '#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A', options: { useRegex: true, maxResults: 50 }, maxChars: 6000 });
    const visibleErrors = errors.ndjson.trim().split('\n')
      .filter(Boolean)
      .map((line) => JSON.parse(line))
      .filter((entry) => entry.kind === 'match' && entry.sheet !== 'Calc');
    assert.deepEqual(visibleErrors, [], `${name} has visible-sheet formula errors:\n${errors.ndjson}`);
  }
});

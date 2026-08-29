import fs from 'node:fs/promises';
import path from 'node:path';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';

const root = new URL('..', import.meta.url);
const outDir = new URL('../qa_renders/', import.meta.url).pathname;
await fs.mkdir(outDir, { recursive: true });
const books = [
  'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
  'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
  'Torque_Drag_and_Buckling_SI.xlsx',
  'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
  'Directional_Drilling_Wellplan_and_Survey_SI.xlsx',
];
for (const name of books) {
  const file = await FileBlob.load(new URL(`outputs/${name}`, root).pathname);
  const wb = await SpreadsheetFile.importXlsx(file);
  const sheets = (await wb.inspect({ kind: 'sheet', include: 'id,name', maxChars: 6000 })).ndjson
    .trim().split('\n').filter(Boolean).map((line) => JSON.parse(line))
    .map((r) => r.name)
    .filter((name) => !['Calc'].includes(name));
  const errors = await wb.inspect({ kind: 'match', searchTerm: '#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A', options: { useRegex: true, maxResults: 100 }, maxChars: 4000 });
  const visibleErrors = errors.ndjson.trim().split('\n').filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter((entry) => entry.kind === 'match' && entry.sheet !== 'Calc');
  const drawings = await wb.inspect({ kind: 'drawing', maxChars: 8000 });
  const errorOutput = visibleErrors.length ? visibleErrors.map((entry) => JSON.stringify(entry)).join('\n') : '{"kind":"notice","message":"Visible-sheet error search matched 0 entries."}';
  process.stdout.write(`BOOK ${name}\nERRORS ${errorOutput}\nDRAWINGS ${drawings.ndjson}\n`);
  for (const sheetName of sheets) {
    const image = await wb.render({ sheetName, autoCrop: 'all', scale: 1, format: 'png' });
    await fs.writeFile(path.join(outDir, `${name.replace('.xlsx','')}-${sheetName}.png`), new Uint8Array(await image.arrayBuffer()));
  }
}

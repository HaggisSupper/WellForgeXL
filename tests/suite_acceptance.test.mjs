import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import JSZip from 'jszip';

const root = new URL('..', import.meta.url);
const outputs = [
  'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
  'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
  'Torque_Drag_and_Buckling_SI.xlsx',
  'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
  'Directional_Drilling_Wellplan_and_Survey_SI.xlsx',
];

test('suite includes five VBA-free source workbooks and the refresh script', async () => {
  for (const name of outputs) {
    const bytes = await fs.readFile(new URL(`outputs/${name}`, root));
    assert.ok(bytes.length > 10_000, `${name} is unexpectedly small`);
    assert.equal(bytes.includes(Buffer.from('vbaProject.bin')), false, `${name} contains a VBA payload`);
  }
  await fs.access(new URL('OfficeScripts/WellForgeWorkbookRefresh.ts', root));
});

test('suite includes the complete JSON exchange contract and both automation hosts', async () => {
  for (const relativePath of [
    'schema/wellforge-analysis-exchange.schema.json',
    'data/wellforge-unit-registry.json',
    'data/wellforge-mock-case.json',
    'OfficeScripts/WellForgeJsonExchange.ts',
    'VBA/WellForgeJsonExchange.bas',
    'tools/Install-WellForgeJsonMacro.ps1',
    'tools/Test-WellForgeJsonMacro.ps1',
    'docs/JSON_EXCHANGE_GUIDE.md',
  ]) await fs.access(new URL(relativePath, root));
});

test('every checked-in source workbook contains all OOXML parts declared by its manifest', async () => {
  for (const name of outputs) {
    const zip = await JSZip.loadAsync(await fs.readFile(new URL(`outputs/${name}`, root)));
    const contentTypes = await zip.file('[Content_Types].xml').async('string');
    const declaredParts = [...contentTypes.matchAll(/<Override\b[^>]*PartName="([^"]+)"[^>]*\/>/g)]
      .map((match) => match[1].replace(/^\//, ''));
    const missingParts = declaredParts.filter((partName) => !zip.file(partName));
    assert.deepEqual(missingParts, [], name);
  }
});

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';

const read = async (relativePath) => fs.readFile(new URL(`../${relativePath}`, import.meta.url), 'utf8');

test('fresh-checkout VBA build uses checked-in source workbooks without Node authoring dependencies', async () => {
  const source = await read('tools/Build-WellForgeVbaSuite.ps1');
  assert.match(source, /workbooks\\source/);
  assert.doesNotMatch(source, /src\\build_suite\.mjs|& node /);
  assert.match(source, /Assert-XlsxPackageIntegrity -Path \$sourcePath/);
  assert.match(source, /Get-FileHash -Algorithm SHA256/);
  assert.match(source, /source-workbooks\.sha256/);
  assert.match(source, /GZipStream/);
});

test('Office Script allocates unseen stable IDs to blank rows transactionally', async () => {
  const source = await read('OfficeScripts/WellForgeJsonExchange.ts');
  assert.match(source, /blankRows/);
  assert.match(source, /rowById\[id\]\s*=\s*row/);
  assert.match(source, /idRange\.getCell\(row,\s*0\)/);
  assert.match(source, /pending\.push\(\{\s*range:\s*idCell/);
  assert.match(source, /No blank worksheet row is available/);
});

test('workbook events pass CountLarge without narrowing to a VBA Long', async () => {
  const [events, core] = await Promise.all([
    read('VBA/ThisWorkbookEvents.txt'),
    read('VBA/WellForgeCore.bas'),
  ]);
  assert.doesNotMatch(events, /CLng\(Target\.CountLarge\)/);
  assert.match(events, /CDbl\(Target\.CountLarge\)/);
  assert.match(core, /WF_HandleSheetChange\(ByVal SheetName As String, ByVal ChangedCells As Double\)/);
});

test('BHA release smoke test verifies the colocated executable hash manifest', async () => {
  const source = await read('tools/Test-WellForgeBhaEngine.ps1');
  assert.match(source, /\.sha256/);
  assert.match(source, /Get-FileHash[\s\S]*SHA256/);
  assert.match(source, /BHA executable hash mismatch/);
});

test('BHA bridge is fully validated before accepted worksheet values are cleared', async () => {
  const source = await read('VBA/WellForgeBhaEngine.bas');
  const validation = source.indexOf('WF_ValidateBhaBridge');
  const clearing = source.indexOf('.ClearContents', source.indexOf('Private Sub WF_WriteBhaBridge'));
  assert.ok(validation >= 0, 'bridge needs a complete validation pass');
  assert.ok(validation < clearing, 'bridge validation must finish before accepted values are cleared');
});

test('BHA bridge completeness requires value-backed FRF and Campbell records', async () => {
  const source = await read('VBA/WellForgeBhaEngine.bas');
  const validator = source.slice(source.indexOf('Private Sub WF_ValidateBhaBridge'));
  assert.match(validator, /frfCount\s*<\s*1/);
  assert.match(validator, /campbellCount\s*<\s*1/);
});

test('BHA bridge parser rejects malformed mode and mode-shape record numbers before numeric coercion', async () => {
  const source = await read('VBA/WellForgeBhaEngine.bas');
  const validator = source.slice(source.indexOf('Private Sub WF_ValidateBhaBridge'));
  assert.match(validator, /If\s+Not\s+IsNumeric\(fields\(1\)\)\s+Then\s+Err\.Raise/);
  assert.match(validator, /INVALID MODE RECORD NUMBER/);
  assert.match(validator, /INVALID MODE SHAPE RECORD NUMBER/);
});

test('refresh companion describes value-only VBA and Rust calculation authority accurately', async () => {
  const source = await read('OfficeScripts/WellForgeWorkbookRefresh.ts');
  assert.doesNotMatch(source, /engineering results[^\n]*remain Excel formulas/i);
  assert.match(source, /VBA\/Rust calculation authority/i);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';

const INSTALLER = new URL('../tools/Install-WellForgeJsonMacro.ps1', import.meta.url);
const SMOKE_TEST = new URL('../tools/Test-WellForgeJsonMacro.ps1', import.meta.url);

test('installer creates macro-enabled copies without overwriting source xlsx workbooks', async () => {
  const source = await fs.readFile(INSTALLER, 'utf8');

  assert.match(source, /outputs[\\/]macro-enabled/);
  assert.match(source, /xlOpenXMLWorkbookMacroEnabled\s*=\s*52/);
  assert.match(source, /\.Open\([^\n]*\$true/);
  assert.match(source, /VBProject\.VBComponents\.Import/);
  assert.match(source, /Enable Trust access to the VBA project object model, then rerun this installer\./);
  assert.doesNotMatch(source, /Remove-Item\s+.*\.xlsx|SaveAs\([^\n]*\.xlsx/i);
});

test('installer removes an existing module before import and releases Excel COM objects', async () => {
  const source = await fs.readFile(INSTALLER, 'utf8');

  assert.match(source, /VBComponents\.Remove/);
  assert.match(source, /finally/);
  assert.match(source, /\[System\.Runtime\.InteropServices\.Marshal\]::ReleaseComObject/);
  assert.match(source, /\.Quit\(\)/);
});

test('smoke test verifies the installed module, validation macro, and Checks sheet', async () => {
  const source = await fs.readFile(SMOKE_TEST, 'utf8');

  assert.match(source, /WellForgeJsonExchange/);
  assert.match(source, /WellForge_ValidateExchange/);
  assert.match(source, /Worksheets\.Item\('Checks'\)/);
  assert.match(source, /blocking/i);
  assert.match(source, /\.Close\(\$false\)/);
  assert.match(source, /\[System\.Runtime\.InteropServices\.Marshal\]::ReleaseComObject/);
});

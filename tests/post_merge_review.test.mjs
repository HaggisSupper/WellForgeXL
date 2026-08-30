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

test('source verifier materializes immutable workbooks before running consumers', async () => {
  const source = await read('tools/verify-node.mjs');
  assert.match(source, /source-workbooks\.sha256/);
  assert.match(source, /createHash\(['"]sha256['"]\)/);
  assert.match(source, /gunzipSync/);
  assert.match(source, /path\.basename\(name\)\s*!==\s*name/);
  assert.match(source, /Invalid source workbook name/);
  assert.match(source, /materializeSourceWorkbooks[\s\S]*runNodeTests/);
});

test('source verifier runs the complete Node suite and VBA lint with bounded child processes', async () => {
  const source = await read('tools/verify-node.mjs');
  assert.match(source, /--test/);
  assert.match(source, /--test-concurrency=1/);
  assert.match(source, /for \(const testFile of tests\)/);
  assert.match(source, /unit_workbook_contract\.test\.mjs/);
  assert.match(source, /tests\/.*\.test\.mjs/);
  assert.match(source, /path\.join\(['"]tools['"],\s*['"]lint_vba\.mjs['"]\)/);
  assert.match(source, /timeout:/);
  assert.match(source, /process\.exitCode\s*=\s*1/);
});

test('Linux CI pins toolchains and enforces source, Rust, and dependency-policy gates', async () => {
  const workflow = await read('.github/workflows/source-verification.yml');
  assert.match(workflow, /permissions:\s*\n\s*contents:\s*read/);
  assert.match(workflow, /node-version:\s*['"]24['"]/);
  assert.match(workflow, /dtolnay\/rust-toolchain@6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772/);
  assert.doesNotMatch(workflow, /dtolnay\/rust-toolchain@master/);
  assert.match(workflow, /toolchain:\s*1\.98\.0/);
  assert.match(workflow, /cargo fmt --all -- --check/);
  assert.match(workflow, /cargo clippy --workspace --all-targets --all-features --locked -- -D warnings/);
  assert.match(workflow, /cargo test --workspace --all-features --locked/);
  assert.match(workflow, /cargo-deny[^\n]*0\.20\.2/);
  assert.match(workflow, /cargo deny check licenses bans sources/);
});

test('release CI uses only publicly provisioned Node dependencies', async () => {
  const [workflow, packageManifest, packageLock] = await Promise.all([
    read('.github/workflows/source-verification.yml'),
    read('package.json'),
    read('package-lock.json'),
  ]);
  assert.match(workflow, /npm ci/);
  assert.match(workflow, /verify-node\.mjs --release/);
  assert.doesNotMatch(packageManifest, /@oai\/artifact-tool/);
  const packageJson = JSON.parse(packageManifest);
  assert.equal(packageJson.dependencies.jszip, '3.10.1');
  const lockJson = JSON.parse(packageLock);
  assert.equal(lockJson.packages['node_modules/jszip'].version, '3.10.1');
  assert.doesNotMatch(packageLock, /codex-primary-runtime/);
});

test('Windows release workflow targets a qualified Excel runner and always retains evidence', async () => {
  const [workflow, builder] = await Promise.all([
    read('.github/workflows/windows-release-verification.yml'),
    read('tools/Build-WellForgeVbaSuite.ps1'),
  ]);
  assert.match(workflow, /workflow_dispatch/);
  assert.match(workflow, /runs-on:\s*\[self-hosted,\s*Windows,\s*wellforgexl-excel\]/);
  assert.match(workflow, /timeout-minutes:/);
  assert.match(workflow, /Build-WellForgeVbaSuite\.ps1/);
  assert.match(workflow, /Write-WellForgeReleaseEvidence\.ps1/);
  assert.match(workflow, /if:\s*always\(\)/);
  assert.match(workflow, /actions\/upload-artifact@v4/);
  assert.match(workflow, /if-no-files-found:\s*warn/);
  assert.match(builder, /\$sourcePaths\[\$name\]\s*=\s*\$sourcePath/);
  assert.match(builder, /\$sourcePath\s*=\s*\$sourcePaths\[\$name\]/);
});

test('Windows evidence writer records every native release surface without claiming success from missing files', async () => {
  const source = await read('tools/Write-WellForgeReleaseEvidence.ps1');
  for (const field of ['rust_toolchain', 'bha_executable', 'trajectory_executable', 'workbooks', 'logs', 'overall_status']) {
    assert.match(source, new RegExp(field));
  }
  assert.match(source, /Get-FileHash[\s\S]*SHA256/);
  assert.match(source, /Test-Path/);
  assert.match(source, /ConvertTo-Json/);
});

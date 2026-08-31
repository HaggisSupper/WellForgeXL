import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

const engine = fs.readFileSync(new URL('../VBA/WellForgeBhaEngine.bas', import.meta.url), 'utf8');
const core = fs.readFileSync(new URL('../VBA/WellForgeCore.bas', import.meta.url), 'utf8');
const builder = fs.readFileSync(new URL('../tools/Build-WellForgeVbaSuite.ps1', import.meta.url), 'utf8');

test('BHA dispatch requires the colocated hash-verified Rust executable', () => {
  assert.match(core, /Case "BHA": WF_RunBhaRustEngine/);
  assert.doesNotMatch(core, /Case "BHA": WF_CalcBHA/);
  assert.match(engine, /ThisWorkbook\.Path[\s\S]*"wellforge-bha\.exe"/);
  assert.match(engine, /WF_FileSha256\(executablePath\)/);
  assert.match(engine, /ENGINE HASH MISMATCH/);
  assert.match(engine, /RUST REQUIRED — NO VBA FALLBACK/);
});

test('BHA process boundary is bounded and validates request/result evidence', () => {
  assert.match(engine, /WF_ExecBounded[\s\S]*timeoutSeconds/);
  assert.match(engine, /validate --input/);
  assert.match(engine, /run --input[\s\S]*--output/);
  assert.match(engine, /verify-result --input[\s\S]*--request-hash/);
  assert.match(engine, /bridge --input[\s\S]*--output[\s\S]*--request-hash/);
  assert.match(engine, /WF_WriteBhaBridge/);
  assert.doesNotMatch(engine, /JsonParse\(/);
  assert.match(engine, /LAST ACCEPTED VALUES PRESERVED/);
  assert.doesNotMatch(engine, /cmd\.exe/i);
});

test('BHA bridge commits are transactional and expose a runtime rollback fault test', () => {
  assert.match(engine, /WF_BhaSnapshot/);
  assert.match(engine, /WF_BhaRestoreSnapshots/);
  assert.match(engine, /WF_BhaSnapshotsMatch/);
  assert.match(engine, /On Error GoTo Rollback/);
  assert.match(engine, /Rollback:[\s\S]*WF_BhaRestoreSnapshots/);
  assert.match(engine, /WF_BHA_INJECT_COMMIT_FAILURE/);
  assert.match(engine, /Public Sub WellForge_BhaRollbackSelfTest/);
  assert.match(engine, /WF_BHA_LAST_ROLLBACK_VERIFIED/);
});

test('Windows suite builder compiles and hashes the Rust engine before Excel', () => {
  const rustBuild = builder.indexOf('Build-WellForgeBhaEngine.ps1');
  const excelStart = builder.indexOf('New-Object -ComObject Excel.Application');
  assert.ok(rustBuild >= 0 && rustBuild < excelStart);
  assert.match(builder, /WellForgeBhaEngine\.bas/);
});

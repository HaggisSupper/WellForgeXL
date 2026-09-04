import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import JSZip from 'jszip';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tool = path.join(repositoryRoot, 'tools', 'release-package.mjs');
const gitSha = '0123456789abcdef0123456789abcdef01234567';
const requiredGates = [
  'native_binaries',
  'vba_compilation_excel_com',
  'unit_switching',
  'chart_rendering',
  'rollback_runtime',
  'package_acceptance',
];
const packageFiles = [
  'LICENSE',
  'LICENSE-APACHE',
  'API_7G_Drill_String_Strength_and_Torque_SI.xlsm',
  'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsm',
  'Directional_Drilling_Wellplan_and_Survey_SI.xlsm',
  'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsm',
  'Torque_Drag_and_Buckling_SI.xlsm',
  'wellforge-bha.exe',
  'wellforge-bha.exe.sha256',
  'wellforge-trajectory.exe',
  'wellforge-trajectory.exe.sha256',
  'wellforge-torque-drag.exe',
  'wellforge-torque-drag.exe.sha256',
  'wellforge-hydraulics.exe',
  'wellforge-hydraulics.exe.sha256',
];

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function run(args) {
  return spawnSync(process.execPath, [tool, ...args], {
    cwd: repositoryRoot,
    encoding: 'utf8',
  });
}

async function arrangePackage() {
  const root = await mkdtemp(path.join(os.tmpdir(), 'wellforgexl-release-package-'));
  const packageDirectory = path.join(root, 'package');
  await mkdir(packageDirectory);
  for (const name of packageFiles.filter((entry) => !entry.endsWith('.sha256'))) {
    await writeFile(path.join(packageDirectory, name), `release-bytes:${name}`);
  }
  for (const executable of ['wellforge-bha.exe', 'wellforge-trajectory.exe', 'wellforge-torque-drag.exe', 'wellforge-hydraulics.exe']) {
    const bytes = await readFile(path.join(packageDirectory, executable));
    await writeFile(path.join(packageDirectory, `${executable}.sha256`), sha256(bytes));
  }
  return {
    root,
    packageDirectory,
    archivePath: path.join(root, `wellforgexl-windows-${gitSha}.zip`),
  };
}

test('create and verify produce an exact, deterministic, hash-bound release archive', async () => {
  const arranged = await arrangePackage();
  const first = run(['create', '--package-dir', arranged.packageDirectory, '--archive', arranged.archivePath, '--git-sha', gitSha]);
  assert.equal(first.status, 0, first.stderr);
  const firstArchive = await readFile(arranged.archivePath);

  const secondArchivePath = path.join(arranged.root, 'second.zip');
  const second = run(['create', '--package-dir', arranged.packageDirectory, '--archive', secondArchivePath, '--git-sha', gitSha]);
  assert.equal(second.status, 0, second.stderr);
  assert.deepEqual(await readFile(secondArchivePath), firstArchive);

  const extractDirectory = path.join(arranged.root, 'extracted');
  const verified = run(['verify', '--archive', arranged.archivePath, '--extract-dir', extractDirectory, '--git-sha', gitSha]);
  assert.equal(verified.status, 0, verified.stderr);
  assert.deepEqual((await readdir(extractDirectory)).sort(), [...packageFiles, 'release-manifest.json'].sort());

  const manifest = JSON.parse(await readFile(path.join(extractDirectory, 'release-manifest.json'), 'utf8'));
  assert.equal(manifest.git_sha, gitSha);
  assert.deepEqual(manifest.files.map(({ path: filePath }) => filePath), packageFiles);
});

test('create rejects stale or unapproved files in the release directory', async () => {
  const arranged = await arrangePackage();
  await writeFile(path.join(arranged.packageDirectory, 'stale.xlsm.bak'), 'stale');
  const result = run(['create', '--package-dir', arranged.packageDirectory, '--archive', arranged.archivePath, '--git-sha', gitSha]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /unexpected package entry/i);
});

test('verify rejects archive bytes that no longer match the release manifest', async () => {
  const arranged = await arrangePackage();
  const created = run(['create', '--package-dir', arranged.packageDirectory, '--archive', arranged.archivePath, '--git-sha', gitSha]);
  assert.equal(created.status, 0, created.stderr);

  const zip = await JSZip.loadAsync(await readFile(arranged.archivePath));
  zip.file('wellforge-bha.exe', 'tampered');
  await writeFile(arranged.archivePath, await zip.generateAsync({ type: 'nodebuffer' }));
  const result = run(['verify', '--archive', arranged.archivePath, '--extract-dir', path.join(arranged.root, 'extract'), '--git-sha', gitSha]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /(?:size|hash) mismatch/i);
});

test('verify rejects duplicate manifest entries and falsified package roles', async () => {
  const arranged = await arrangePackage();
  const created = run(['create', '--package-dir', arranged.packageDirectory, '--archive', arranged.archivePath, '--git-sha', gitSha]);
  assert.equal(created.status, 0, created.stderr);

  const zip = await JSZip.loadAsync(await readFile(arranged.archivePath));
  const manifest = JSON.parse(await zip.file('release-manifest.json').async('string'));
  manifest.files[0].role = 'native-engine';
  manifest.files.push({ ...manifest.files[0] });
  zip.file('release-manifest.json', `${JSON.stringify(manifest, null, 2)}\n`);
  await writeFile(arranged.archivePath, await zip.generateAsync({ type: 'nodebuffer' }));
  const result = run(['verify', '--archive', arranged.archivePath, '--extract-dir', path.join(arranged.root, 'extract'), '--git-sha', gitSha]);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /duplicate manifest entry|role mismatch/i);
});

test('evidence passes only when every required gate is passed for the same run and git SHA', async () => {
  const arranged = await arrangePackage();
  const created = run(['create', '--package-dir', arranged.packageDirectory, '--archive', arranged.archivePath, '--git-sha', gitSha]);
  assert.equal(created.status, 0, created.stderr);
  const gateResultsPath = path.join(arranged.root, 'gate-results.json');
  const evidencePath = path.join(arranged.root, 'release-evidence.json');
  const logDirectory = path.join(arranged.root, 'logs');
  const renderDirectory = path.join(arranged.root, 'chart-renders');
  await mkdir(logDirectory);
  await mkdir(renderDirectory);
  await Promise.all([
    writeFile(path.join(logDirectory, 'bha-native-smoke.log'), 'bha smoke passed\n'),
    writeFile(path.join(logDirectory, 'trajectory-native-smoke.log'), 'trajectory smoke passed\n'),
    writeFile(path.join(logDirectory, 'windows-acceptance.stdout.log'), 'acceptance passed\n'),
    writeFile(path.join(logDirectory, 'build.jsonl'), '{"status":"passed"}\n'),
    writeFile(path.join(renderDirectory, 'chart.png'), Buffer.alloc(600, 1)),
  ]);
  const runId = 'run-42-attempt-1';
  const gates = Object.fromEntries(requiredGates.map((name) => [name, { status: 'passed' }]));
  await writeFile(gateResultsPath, `\ufeff${JSON.stringify({ schema_version: '1.0.0', run_id: runId, git_sha: gitSha, gates })}`);

  const passed = run([
    'evidence', '--gate-results', gateResultsPath, '--archive', arranged.archivePath,
    '--output', evidencePath, '--support-root', arranged.root, '--git-sha', gitSha, '--run-id', runId,
  ]);
  assert.equal(passed.status, 0, passed.stderr);
  const evidence = JSON.parse(await readFile(evidencePath, 'utf8'));
  assert.equal(evidence.overall_status, 'passed');
  assert.deepEqual(Object.keys(evidence.gates), requiredGates);
  assert.ok(evidence.supporting_artifacts.every((entry) => /^[0-9a-f]{64}$/u.test(entry.sha256)));

  delete gates.rollback_runtime;
  await writeFile(gateResultsPath, JSON.stringify({ schema_version: '1.0.0', run_id: runId, git_sha: gitSha, gates }));
  const failed = run([
    'evidence', '--gate-results', gateResultsPath, '--archive', arranged.archivePath,
    '--output', evidencePath, '--support-root', arranged.root, '--git-sha', gitSha, '--run-id', runId,
  ]);
  assert.notEqual(failed.status, 0);
  const failedEvidence = JSON.parse(await readFile(evidencePath, 'utf8'));
  assert.equal(failedEvidence.overall_status, 'failed');
  assert.equal(failedEvidence.gates.rollback_runtime.status, 'missing');
});

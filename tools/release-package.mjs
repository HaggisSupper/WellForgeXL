import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

import JSZip from 'jszip';

export const packageFiles = [
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

export const requiredGates = [
  'native_binaries',
  'vba_compilation_excel_com',
  'unit_switching',
  'chart_rendering',
  'rollback_runtime',
  'package_acceptance',
];

const manifestName = 'release-manifest.json';
const deterministicZipDate = new Date('1980-01-01T00:00:00.000Z');

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function assertGitSha(gitSha) {
  if (!/^[0-9a-f]{40}$/u.test(gitSha)) throw new Error(`Invalid git SHA: ${gitSha}`);
}

function parseArgs(values) {
  const result = new Map();
  for (let index = 0; index < values.length; index += 2) {
    const name = values[index];
    const value = values[index + 1];
    if (!name?.startsWith('--') || value === undefined) throw new Error(`Invalid argument list near ${name ?? '<end>'}`);
    result.set(name.slice(2), value);
  }
  return result;
}

function requireArg(args, name) {
  const value = args.get(name);
  if (!value) throw new Error(`Missing --${name}`);
  return path.resolve(value);
}

function requireTextArg(args, name) {
  const value = args.get(name);
  if (!value) throw new Error(`Missing --${name}`);
  return value;
}

function compareExactEntries(actual, expected, label) {
  const duplicates = actual.filter((entry, index) => actual.indexOf(entry) !== index);
  if (duplicates.length > 0) throw new Error(`Duplicate ${label} entry: ${[...new Set(duplicates)].join(', ')}`);
  const actualSorted = [...actual].sort();
  const expectedSorted = [...expected].sort();
  const unexpected = actualSorted.filter((entry) => !expectedSorted.includes(entry));
  const missing = expectedSorted.filter((entry) => !actualSorted.includes(entry));
  if (unexpected.length > 0) throw new Error(`Unexpected ${label} entry: ${unexpected.join(', ')}`);
  if (missing.length > 0) throw new Error(`Missing ${label} entry: ${missing.join(', ')}`);
}

function roleFor(name) {
  if (name.startsWith('LICENSE')) return 'license';
  if (name.endsWith('.xlsm')) return 'workbook';
  if (name.endsWith('.exe')) return 'native-engine';
  return 'sha256-manifest';
}

async function packagePayload(packageDirectory) {
  const entries = await readdir(packageDirectory, { withFileTypes: true });
  if (entries.some((entry) => !entry.isFile())) {
    const entry = entries.find((item) => !item.isFile());
    throw new Error(`Unexpected package entry: ${entry.name}`);
  }
  compareExactEntries(
    entries.map(({ name }) => name).filter((name) => name !== manifestName),
    packageFiles,
    'package',
  );
  return new Map(await Promise.all(packageFiles.map(async (name) => [name, await readFile(path.join(packageDirectory, name))])));
}

function validateEngineSidecars(files) {
  for (const executable of ['wellforge-bha.exe', 'wellforge-trajectory.exe', 'wellforge-torque-drag.exe', 'wellforge-hydraulics.exe']) {
    const expected = files.get(`${executable}.sha256`).toString('utf8').trim().toLowerCase();
    if (!/^[0-9a-f]{64}$/u.test(expected)) throw new Error(`Invalid hash manifest: ${executable}.sha256`);
    if (expected !== sha256(files.get(executable))) throw new Error(`Executable hash mismatch: ${executable}`);
  }
}

export async function createReleaseArchive({ packageDirectory, archivePath, gitSha }) {
  assertGitSha(gitSha);
  const files = await packagePayload(packageDirectory);
  validateEngineSidecars(files);
  const manifest = {
    schema_version: '1.0.0',
    git_sha: gitSha,
    files: packageFiles.map((filePath) => ({
      path: filePath,
      role: roleFor(filePath),
      sha256: sha256(files.get(filePath)),
      size_bytes: files.get(filePath).length,
    })),
  };
  const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  await writeFile(path.join(packageDirectory, manifestName), manifestBytes);

  const zip = new JSZip();
  for (const name of packageFiles) zip.file(name, files.get(name), { date: deterministicZipDate });
  zip.file(manifestName, manifestBytes, { date: deterministicZipDate });
  const archive = await zip.generateAsync({
    type: 'nodebuffer',
    compression: 'DEFLATE',
    compressionOptions: { level: 9 },
    platform: 'DOS',
  });
  await mkdir(path.dirname(archivePath), { recursive: true });
  await writeFile(archivePath, archive);
  return { manifest, archive_sha256: sha256(archive) };
}

async function loadVerifiedArchive(archivePath, gitSha) {
  assertGitSha(gitSha);
  const archive = await readFile(archivePath);
  const zip = await JSZip.loadAsync(archive, { checkCRC32: true });
  const entries = Object.values(zip.files);
  if (entries.some((entry) => entry.dir || entry.name.includes('/') || entry.name.includes('\\'))) {
    const entry = entries.find((item) => item.dir || item.name.includes('/') || item.name.includes('\\'));
    throw new Error(`Unexpected archive entry: ${entry.name}`);
  }
  compareExactEntries(entries.map(({ name }) => name), [...packageFiles, manifestName], 'archive');
  const manifestBytes = await zip.file(manifestName).async('nodebuffer');
  const manifest = JSON.parse(manifestBytes.toString('utf8'));
  if (manifest.schema_version !== '1.0.0') throw new Error(`Unsupported release manifest: ${manifest.schema_version}`);
  if (manifest.git_sha !== gitSha) throw new Error(`Release manifest git SHA mismatch: ${manifest.git_sha}`);
  if (!Array.isArray(manifest.files)) throw new Error('Release manifest files must be an array');
  compareExactEntries(manifest.files.map(({ path: filePath }) => filePath), packageFiles, 'manifest');

  const files = new Map();
  for (const expected of manifest.files) {
    if (expected.role !== roleFor(expected.path)) throw new Error(`Role mismatch: ${expected.path}`);
    const bytes = await zip.file(expected.path).async('nodebuffer');
    if (expected.size_bytes !== bytes.length) throw new Error(`Size mismatch: ${expected.path}`);
    if (expected.sha256 !== sha256(bytes)) throw new Error(`Hash mismatch: ${expected.path}`);
    files.set(expected.path, bytes);
  }
  validateEngineSidecars(files);
  return { archive, manifest, manifestBytes, files };
}

async function inventorySupportingEvidence(supportRoot, gateResultsPath) {
  const records = [];
  const gateBytes = await readFile(gateResultsPath);
  records.push({ path: 'gate-results.json', sha256: sha256(gateBytes), size_bytes: gateBytes.length });
  for (const directoryName of ['logs', 'chart-renders']) {
    const directoryPath = path.join(supportRoot, directoryName);
    const entries = await readdir(directoryPath, { withFileTypes: true });
    for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
      if (!entry.isFile()) throw new Error(`Unexpected supporting evidence entry: ${directoryName}/${entry.name}`);
      const bytes = await readFile(path.join(directoryPath, entry.name));
      records.push({
        path: `${directoryName}/${entry.name}`,
        sha256: sha256(bytes),
        size_bytes: bytes.length,
      });
    }
  }
  const paths = new Set(records.map(({ path: filePath }) => filePath));
  for (const requiredLog of ['bha-native-smoke.log', 'trajectory-native-smoke.log', 'windows-acceptance.stdout.log']) {
    const record = records.find(({ path: filePath }) => filePath === `logs/${requiredLog}`);
    if (!record || record.size_bytes === 0) throw new Error(`Required supporting log is missing or empty: ${requiredLog}`);
  }
  if (![...paths].some((filePath) => filePath.startsWith('logs/') && filePath.endsWith('.jsonl'))) {
    throw new Error('No JSONL build log was retained');
  }
  const renders = records.filter(({ path: filePath }) => filePath.startsWith('chart-renders/') && filePath.endsWith('.png'));
  if (renders.length === 0 || renders.some(({ size_bytes: sizeBytes }) => sizeBytes <= 512)) {
    throw new Error('Chart rendering evidence is missing or empty');
  }
  return records;
}

export async function verifyAndExtractReleaseArchive({ archivePath, extractDirectory, gitSha }) {
  const verified = await loadVerifiedArchive(archivePath, gitSha);
  await mkdir(extractDirectory, { recursive: true });
  const existing = await readdir(extractDirectory);
  if (existing.length > 0) throw new Error(`Extraction directory is not empty: ${extractDirectory}`);
  for (const [name, bytes] of verified.files) await writeFile(path.join(extractDirectory, name), bytes);
  await writeFile(path.join(extractDirectory, manifestName), verified.manifestBytes);
  return verified.manifest;
}

export async function writeReleaseEvidence({ gateResultsPath, archivePath, outputPath, supportRoot, gitSha, runId }) {
  const issues = [];
  let gateDocument;
  let verifiedArchive;
  let supportingArtifacts = [];
  try {
    const gateText = await readFile(gateResultsPath, 'utf8');
    gateDocument = JSON.parse(gateText.replace(/^\uFEFF/u, ''));
  } catch (error) {
    issues.push(`Gate results unavailable: ${error.message}`);
    gateDocument = { gates: {} };
  }
  if (gateDocument.schema_version !== '1.0.0') issues.push(`Unsupported gate-results schema: ${gateDocument.schema_version ?? '<missing>'}`);
  if (gateDocument.git_sha !== gitSha) issues.push(`Gate-results git SHA mismatch: ${gateDocument.git_sha ?? '<missing>'}`);
  if (gateDocument.run_id !== runId) issues.push(`Gate-results run ID mismatch: ${gateDocument.run_id ?? '<missing>'}`);

  const gates = Object.fromEntries(requiredGates.map((name) => {
    const gate = gateDocument.gates?.[name];
    if (!gate) return [name, { status: 'missing' }];
    if (gate.status !== 'passed') issues.push(`Gate did not pass: ${name} (${gate.status ?? '<missing>'})`);
    return [name, gate];
  }));
  for (const [name, gate] of Object.entries(gates)) {
    if (gate.status === 'missing') issues.push(`Required gate is missing: ${name}`);
  }

  try {
    verifiedArchive = await loadVerifiedArchive(archivePath, gitSha);
  } catch (error) {
    issues.push(`Release archive invalid: ${error.message}`);
  }
  try {
    supportingArtifacts = await inventorySupportingEvidence(supportRoot, gateResultsPath);
    const renderedCount = supportingArtifacts.filter(({ path: filePath }) => filePath.startsWith('chart-renders/')).length;
    const reportedCount = gateDocument.gates?.chart_rendering?.details?.exported_png_files;
    if (Number.isInteger(reportedCount) && reportedCount !== renderedCount) {
      issues.push(`Chart evidence count mismatch: gate reported ${reportedCount}, retained ${renderedCount}`);
    }
  } catch (error) {
    issues.push(`Supporting evidence invalid: ${error.message}`);
  }

  const evidence = {
    schema_version: '2.0.0',
    generated_at_utc: new Date().toISOString(),
    run_id: runId,
    git_sha: gitSha,
    gates,
    supporting_artifacts: supportingArtifacts,
    package: verifiedArchive ? {
      archive_path: archivePath,
      archive_sha256: sha256(verifiedArchive.archive),
      manifest_sha256: sha256(verifiedArchive.manifestBytes),
      files: verifiedArchive.manifest.files,
    } : null,
    issues,
    overall_status: issues.length === 0 ? 'passed' : 'failed',
  };
  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
  return evidence;
}

async function main() {
  const [command, ...rawArgs] = process.argv.slice(2);
  const args = parseArgs(rawArgs);
  if (command === 'create') {
    const result = await createReleaseArchive({
      packageDirectory: requireArg(args, 'package-dir'),
      archivePath: requireArg(args, 'archive'),
      gitSha: requireTextArg(args, 'git-sha'),
    });
    process.stdout.write(`${JSON.stringify(result)}\n`);
    return;
  }
  if (command === 'verify') {
    const manifest = await verifyAndExtractReleaseArchive({
      archivePath: requireArg(args, 'archive'),
      extractDirectory: requireArg(args, 'extract-dir'),
      gitSha: requireTextArg(args, 'git-sha'),
    });
    process.stdout.write(`${JSON.stringify({ git_sha: manifest.git_sha, files: manifest.files.length })}\n`);
    return;
  }
  if (command === 'evidence') {
    const evidence = await writeReleaseEvidence({
      gateResultsPath: requireArg(args, 'gate-results'),
      archivePath: requireArg(args, 'archive'),
      outputPath: requireArg(args, 'output'),
      supportRoot: requireArg(args, 'support-root'),
      gitSha: requireTextArg(args, 'git-sha'),
      runId: requireTextArg(args, 'run-id'),
    });
    if (evidence.overall_status !== 'passed') process.exitCode = 1;
    return;
  }
  throw new Error(`Unknown command: ${command ?? '<missing>'}`);
}

main().catch((error) => {
  process.stderr.write(`${error.stack ?? error}\n`);
  process.exitCode = 1;
});

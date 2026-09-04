import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';
import { spawn } from 'node:child_process';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourceDirectory = path.join(repositoryRoot, 'workbooks', 'source');
const outputDirectory = path.join(repositoryRoot, 'outputs');
const sourceManifest = path.join(sourceDirectory, 'source-workbooks.sha256');
const childTimeoutMs = 15 * 60 * 1000;
const testPattern = 'tests/*.test.mjs';
const releaseTests = [
  'bha_rust_engine_contract.test.mjs',
  'depth_chart_contract.test.mjs',
  'directional_math.test.mjs',
  'exchange_mock_payload.test.mjs',
  'exchange_roundtrip.test.mjs',
  'exchange_schema.test.mjs',
  'html_ui_contract.test.mjs',
  'office_script_exchange.test.mjs',
  'post_merge_review.test.mjs',
  'release_package.test.mjs',
  'suite_acceptance.test.mjs',
  'trajectory_rust_engine_contract.test.mjs',
  'unit_contract.test.mjs',
  'unit_workbook_contract.test.mjs',
  'vba_engine_contract.test.mjs',
  'vba_exchange_contract.test.mjs',
  'vba_installer_contract.test.mjs',
];
const authoringTests = [
  'api7g.test.mjs',
  'bha.test.mjs',
  'bha_geometry_visualization_contract.test.mjs',
  'bha_workbook_engine_contract.test.mjs',
  'directional.test.mjs',
  'directional_structure.test.mjs',
  'directional_workbook_values.test.mjs',
  'exchange_mapping.test.mjs',
  'exchange_ooxml.test.mjs',
  'hydraulics.test.mjs',
  'hydraulics_chart_data.test.mjs',
  'industry_visualization_contract.test.mjs',
  'render_contract.test.mjs',
  'shared_mock_case.test.mjs',
  'torque_drag.test.mjs',
  'trajectory_workbook_engine_contract.test.mjs',
  'unit_display_contract.test.mjs',
];

function sha256(filePath) {
  return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

export function materializeSourceWorkbooks() {
  const entries = readFileSync(sourceManifest, 'utf8')
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((line) => {
      const match = /^([0-9a-f]{64})\s+\*?(.+)$/iu.exec(line);
      if (!match) throw new Error(`Invalid source workbook manifest line: ${line}`);
      const name = match[2];
      if (path.basename(name) !== name || path.extname(name).toLowerCase() !== '.xlsx') {
        throw new Error(`Invalid source workbook name: ${name}`);
      }
      return { expectedHash: match[1].toLowerCase(), name };
    });

  mkdirSync(outputDirectory, { recursive: true });
  for (const { expectedHash, name } of entries) {
    const plainSource = path.join(sourceDirectory, name);
    const compressedSource = `${plainSource}.gz`;
    const destination = path.join(outputDirectory, name);
    if (existsSync(plainSource)) {
      writeFileSync(destination, readFileSync(plainSource));
    } else if (existsSync(compressedSource)) {
      writeFileSync(destination, gunzipSync(readFileSync(compressedSource)));
    } else {
      throw new Error(`Source workbook is missing: ${name}`);
    }
    const actualHash = sha256(destination);
    if (actualHash !== expectedHash) {
      throw new Error(`Source workbook hash mismatch for ${name}: ${actualHash}`);
    }
  }
  return entries.length;
}

function run(command, args) {
  return new Promise((resolve, reject) => {
    const detached = process.platform !== 'win32';
    const child = spawn(command, args, { cwd: repositoryRoot, stdio: 'inherit', detached });
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      try {
        if (detached) process.kill(-child.pid, 'SIGKILL');
        else child.kill('SIGKILL');
      } catch {
        child.kill('SIGKILL');
      }
    }, childTimeoutMs);
    child.once('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once('close', (code, signal) => {
      clearTimeout(timer);
      if (timedOut) reject(new Error(`${command} ${args.join(' ')} exceeded ${childTimeoutMs} ms; process tree terminated`));
      else if (code !== 0) reject(new Error(`${command} ${args.join(' ')} exited ${code ?? signal}`));
      else resolve();
    });
  });
}

export async function runNodeTests() {
  const discoveredTests = readdirSync(path.join(repositoryRoot, 'tests'))
    .filter((name) => name.endsWith('.test.mjs'))
    .sort();
  const classifiedTests = [...releaseTests, ...authoringTests].sort();
  if (new Set(classifiedTests).size !== classifiedTests.length) {
    throw new Error('A Node test is classified as both release and authoring-only.');
  }
  if (JSON.stringify(classifiedTests) !== JSON.stringify(discoveredTests)) {
    const unclassified = discoveredTests.filter((name) => !classifiedTests.includes(name));
    const missing = classifiedTests.filter((name) => !discoveredTests.includes(name));
    throw new Error(`Node test classification mismatch. Unclassified: ${unclassified.join(', ') || '<none>'}; missing: ${missing.join(', ') || '<none>'}`);
  }
  const selectedTests = process.argv.includes('--release') ? releaseTests : discoveredTests;
  const tests = selectedTests.map((name) => path.join('tests', name));
  if (tests.length === 0) throw new Error(`No tests matched ${testPattern}`);
  for (const testFile of tests) {
    await run(process.execPath, ['--test', '--test-concurrency=1', '--test-force-exit', testFile]);
  }
}

export async function runVbaLint() {
  await run(process.execPath, [path.join('tools', 'lint_vba.mjs')]);
}

try {
  const workbookCount = materializeSourceWorkbooks();
  process.stdout.write(`Verified and materialized ${workbookCount} source workbooks.\n`);
  if (!process.argv.includes('--materialize-only')) {
    await runNodeTests();
    await runVbaLint();
  }
} catch (error) {
  process.stderr.write(`${error.stack ?? error}\n`);
  process.exitCode = 1;
}

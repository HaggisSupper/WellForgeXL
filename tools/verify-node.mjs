import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';
import { spawnSync } from 'node:child_process';

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
  'office_script_exchange.test.mjs',
  'post_merge_review.test.mjs',
  'suite_acceptance.test.mjs',
  'trajectory_rust_engine_contract.test.mjs',
  'unit_contract.test.mjs',
  'unit_workbook_contract.test.mjs',
  'vba_engine_contract.test.mjs',
  'vba_exchange_contract.test.mjs',
  'vba_installer_contract.test.mjs',
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
  const result = spawnSync(command, args, {
    cwd: repositoryRoot,
    encoding: 'utf8',
    stdio: 'inherit',
    timeout: childTimeoutMs,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} ${args.join(' ')} exited ${result.status}`);
}

export function runNodeTests() {
  const discoveredTests = readdirSync(path.join(repositoryRoot, 'tests'))
    .filter((name) => name.endsWith('.test.mjs'))
    .sort();
  const selectedTests = process.argv.includes('--release') ? releaseTests : discoveredTests;
  const tests = selectedTests.map((name) => path.join('tests', name));
  if (tests.length === 0) throw new Error(`No tests matched ${testPattern}`);
  for (const testFile of tests) {
    run(process.execPath, ['--test', '--test-concurrency=1', '--test-force-exit', testFile]);
  }
}

export function runVbaLint() {
  run(process.execPath, [path.join('tools', 'lint_vba.mjs')]);
}

try {
  const workbookCount = materializeSourceWorkbooks();
  process.stdout.write(`Verified and materialized ${workbookCount} source workbooks.\n`);
  if (!process.argv.includes('--materialize-only')) {
    runNodeTests();
    runVbaLint();
  }
} catch (error) {
  process.stderr.write(`${error.stack ?? error}\n`);
  process.exitCode = 1;
}

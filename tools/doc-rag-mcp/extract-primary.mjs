import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { loadJsonl, writeJsonl, writeJsonFile, ensureServerRunning, ensureModelReady } from './pipeline.mjs';
import { extractWithDocling } from './extractors.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const manifestPath = process.argv[3] ?? path.resolve('outputs/doc-rag/manifest.jsonl');
const outDir = process.argv[4] ?? path.resolve('outputs/doc-rag');
const limit = Number(process.argv[5] ?? '100');

const config = JSON.parse(await readFile(configPath, 'utf8'));

console.error('[extract-primary] Ensuring LM Studio server is running...');
await ensureServerRunning();

console.error('[extract-primary] Ensuring model is loaded...');
const modelReady = await ensureModelReady();
if (!modelReady) {
  console.error('[extract-primary] ERROR: Model failed to load');
  process.exit(1);
}
console.error(`[extract-primary] Model ready: ${modelReady}`);

const rows = await loadJsonl(manifestPath);
const primaryRows = rows.filter((r) => r.heuristicStage === 'primary').slice(0, limit);

const extractedPath = path.join(outDir, 'extracted-primary.jsonl');
const summaryPath = path.join(outDir, 'extraction-summary.json');

console.error(`Extracting ${primaryRows.length} primary files (limit=${limit})`);
console.error(`Using concurrent pool size: 3 (tuned for 6GB VRAM + q5_k_m model)`);

// Semaphore for concurrency control
class Semaphore {
  constructor(max) {
    this.max = max;
    this.current = 0;
    this.queue = [];
  }
  async acquire() {
    if (this.current < this.max) {
      this.current += 1;
      return;
    }
    return new Promise((resolve) => {
      this.queue.push(resolve);
    });
  }
  release() {
    this.current -= 1;
    const resolve = this.queue.shift();
    if (resolve) {
      this.current += 1;
      resolve();
    }
  }
}

const sem = new Semaphore(3); // 3 concurrent extraction tasks
const extracted = [];
let completed = 0;

const extractTask = async (row) => {
  await sem.acquire();
  try {
    const result = await extractWithDocling(row.path, config);
    const entry = {
      path: row.path,
      rel: row.rel,
      families: row.families,
      text: result.text,
      pages: result.pages,
      tables: result.tables,
      backend: result.backend,
      warnings: result.warnings,
      textLength: result.text?.length ?? 0,
    };
    extracted.push(entry);
    completed += 1;
    if (completed % 10 === 0) {
      console.error(`  ${completed}/${primaryRows.length}...`);
    }
  } catch (err) {
    const entry = {
      path: row.path,
      rel: row.rel,
      families: row.families,
      text: '',
      pages: [],
      tables: [],
      backend: 'error',
      warnings: [err.message],
      textLength: 0,
    };
    extracted.push(entry);
    completed += 1;
    if (completed % 10 === 0) {
      console.error(`  ${completed}/${primaryRows.length}...`);
    }
  } finally {
    sem.release();
  }
};

await Promise.all(primaryRows.map((row) => extractTask(row)));

await writeJsonl(extractedPath, extracted);

const summary = {
  ok: true,
  count: extracted.length,
  withText: extracted.filter((r) => r.textLength > 0).length,
  withTables: extracted.filter((r) => r.tables?.length > 0).length,
  withWarnings: extracted.filter((r) => r.warnings?.length > 0).length,
  totalTextChars: extracted.reduce((sum, r) => sum + r.textLength, 0),
  paths: { extracted: extractedPath },
};

await writeJsonFile(summaryPath, summary);
console.log(JSON.stringify(summary, null, 2));

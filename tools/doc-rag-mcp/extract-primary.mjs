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

const extracted = [];
for (let i = 0; i < primaryRows.length; i += 1) {
  const row = primaryRows[i];
  if ((i + 1) % 10 === 0) {
    console.error(`  ${i + 1}/${primaryRows.length}...`);
  }
  try {
    const result = await extractWithDocling(row.path, config);
    extracted.push({
      path: row.path,
      rel: row.rel,
      families: row.families,
      text: result.text,
      pages: result.pages,
      tables: result.tables,
      backend: result.backend,
      warnings: result.warnings,
      textLength: result.text?.length ?? 0,
    });
  } catch (err) {
    extracted.push({
      path: row.path,
      rel: row.rel,
      families: row.families,
      text: '',
      pages: [],
      tables: [],
      backend: 'error',
      warnings: [err.message],
      textLength: 0,
    });
  }
}

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

import { readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import {
  walkDocuments,
  classifyPath,
  heuristicStage,
  isNoiseByPath,
  topFolder,
  buildClassificationPrompt,
  callLmStudioChat,
  parseClassification,
  classifyBatch,
  loadJsonl,
  writeJsonl,
  writeJsonFile,
} from './pipeline.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const rootDir = process.argv[3] ?? path.resolve('G:\\My Drive\\Drilling Background');
const outDir = process.argv[4] ?? path.resolve('outputs/doc-rag');
const mlLimit = Number(process.argv[5] ?? '1000');
const batchSize = Number(process.argv[6] ?? '10');

const config = JSON.parse(await readFile(configPath, 'utf8'));

const manifestPath = path.join(outDir, 'manifest.jsonl');
const mlManifestPath = path.join(outDir, 'manifest-ml.jsonl');
const summaryPath = path.join(outDir, 'inventory-summary.json');

async function fileStats(filePath) {
  try {
    const s = await stat(filePath);
    return { size: s.size, mtime: s.mtimeMs };
  } catch {
    return { size: 0, mtime: 0 };
  }
}


async function main() {
  console.error(`Building inventory for ${rootDir}`);
  const existingManifest = await loadJsonl(manifestPath);
  const existingMl = await loadJsonl(mlManifestPath);
  const cache = new Map(existingManifest.map((r) => [r.path, r]));
  const mlCache = new Map(existingMl.map((r) => [r.path, r]));

  const docs = await walkDocuments(rootDir);
  console.error(`Found ${docs.length} candidate files`);

  const rows = [];
  let mlCount = 0;
  let primaryHeuristic = 0;
  let secondaryHeuristic = 0;
  let noiseHeuristic = 0;

  for (let i = 0; i < docs.length; i += 1) {
    const filePath = docs[i];
    if ((i + 1) % 1000 === 0) {
      console.error(`  ${i + 1}/${docs.length}...`);
    }

    const rel = path.relative(rootDir, filePath);
    const ext = path.extname(filePath).toLowerCase();
    const { size, mtime } = await fileStats(filePath);
    const families = classifyPath(filePath);
    const heuristic = heuristicStage(filePath, families, rel);

    if (heuristic.stage === 'primary') primaryHeuristic += 1;
    else if (heuristic.stage === 'secondary') secondaryHeuristic += 1;
    else noiseHeuristic += 1;

    const cached = cache.get(filePath);
    let ml = null;
    if (cached && cached.mtime === mtime && cached.size === size && cached.ml) {
      ml = cached.ml;
    } else if (mlCache.has(filePath)) {
      ml = mlCache.get(filePath).ml;
    }

    rows.push({
      path: filePath,
      rel,
      ext,
      size,
      mtime,
      topFolder: topFolder(rel),
      families,
      heuristicStage: heuristic.stage,
      heuristicKeep: heuristic.keep,
      heuristicReason: heuristic.reason,
      ml,
      cachedAt: new Date().toISOString(),
    });
  }

  console.error(`Heuristic routing: primary=${primaryHeuristic}, secondary=${secondaryHeuristic}, noise=${noiseHeuristic}`);

  const mlCandidates = rows
    .filter((r) => !r.ml && (r.heuristicStage === 'primary' || r.heuristicStage === 'secondary'))
    .slice(0, mlLimit);

  console.error(`ML classification candidates: ${mlCandidates.length} (limit=${mlLimit}, batch=${batchSize})`);

  for (let i = 0; i < mlCandidates.length; i += batchSize) {
    const batch = mlCandidates.slice(i, i + batchSize);
    console.error(`  ML batch ${i + 1}-${Math.min(i + batchSize, mlCandidates.length)}/${mlCandidates.length}...`);

    try {
      const results = await classifyBatch(batch, config);
      for (const row of batch) {
        row.ml = results.get(row.path);
        mlCount += 1;
      }
    } catch (err) {
      console.error(`    batch failed: ${err.message}; falling back to individual calls`);
      for (const row of batch) {
        try {
          const raw = await callLmStudioChat({
            baseUrl: config.lm_studio.base_url,
            model: config.lm_studio.model,
            messages: buildClassificationPrompt({ filePath: row.path, stats: { size: row.size, ext: row.ext, families: row.families } }),
            maxTokens: 512,
          });
          row.ml = parseClassification(raw);
          mlCount += 1;
        } catch (innerErr) {
          row.ml = { stage: null, keep: false, family: '', confidence: 0, reason: `lm-studio error: ${innerErr.message}`, signals: [], raw: '' };
        }
      }
    }
  }

  const mlRows = rows.filter((r) => r.ml);
  await writeJsonl(manifestPath, rows);
  await writeJsonl(mlManifestPath, mlRows);

  const summary = {
    ok: true,
    rootDir,
    outDir,
    total: rows.length,
    heuristic: {
      primary: primaryHeuristic,
      secondary: secondaryHeuristic,
      noise: noiseHeuristic,
    },
    ml: {
      classified: mlRows.length,
      thisRun: mlCount,
      primary: mlRows.filter((r) => r.ml.stage === 'primary').length,
      secondary: mlRows.filter((r) => r.ml.stage === 'secondary').length,
      noise: mlRows.filter((r) => r.ml.stage === 'noise').length,
      unparseable: mlRows.filter((r) => r.ml.stage === null).length,
    },
    paths: {
      manifest: manifestPath,
      mlManifest: mlManifestPath,
    },
  };

  await writeJsonFile(summaryPath, summary);
  console.log(JSON.stringify(summary, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

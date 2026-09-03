import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { loadJsonl, writeJsonFile, classifyBatch, parseClassification, callLmStudioChat } from './pipeline.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const manifestPath = process.argv[3] ?? path.resolve('outputs/doc-rag/manifest.jsonl');
const outDir = process.argv[4] ?? path.resolve('outputs/doc-rag');
const samplesPerStage = Number(process.argv[5] ?? '20');
const batchSize = Number(process.argv[6] ?? '5');

const config = JSON.parse(await readFile(configPath, 'utf8'));
const rows = await loadJsonl(manifestPath);

function sampleRows(stage, n) {
  const pool = rows.filter((r) => r.heuristicStage === stage);
  const shuffled = pool.sort(() => Math.random() - 0.5);
  return shuffled.slice(0, n);
}

function buildClassificationPrompt({ filePath, stats }) {
  return [
    {
      role: 'system',
      content: [
        'Classify drilling-engineering documents for a RAG ingest pipeline.',
        'Return JSON only with exactly these fields: stage (string), keep (boolean), family (string), confidence (number 0-1), reason (string), signals (string[]).',
        'stage MUST be one of: "primary", "secondary", or "noise".',
        'primary = validation/oracle cases, saved runs, validation workbooks, direct theory papers, mechanism references, core manuals, calculator specs, trajectory listings.',
        'secondary = supporting guides, training docs with worked examples, related catalogs, legacy simulation artifacts that expose equations or outputs.',
        'noise = marketing, cover pages only, blank scans, generic admin, duplicates without substance, superficial indexes, icon/image assets.',
        'Be strict: prefer noise unless there is concrete engineering value.',
      ].join(' '),
    },
    {
      role: 'user',
      content: JSON.stringify({ filePath, stats }),
    },
  ];
}

async function main() {
  const candidates = [
    ...sampleRows('primary', samplesPerStage),
    ...sampleRows('secondary', samplesPerStage),
    ...sampleRows('noise', samplesPerStage),
  ];

  console.error(`Validating ${candidates.length} files (${samplesPerStage} per heuristic stage)`);

  for (let i = 0; i < candidates.length; i += batchSize) {
    const batch = candidates.slice(i, i + batchSize);
    console.error(`  batch ${i + 1}-${Math.min(i + batchSize, candidates.length)}/${candidates.length}`);
    try {
      const results = await classifyBatch(batch, config);
      for (const row of batch) {
        row.ml = results.get(row.path);
      }
    } catch (err) {
      console.error(`    batch failed: ${err.message}; falling back`);
      for (const row of batch) {
        try {
          const raw = await callLmStudioChat({
            baseUrl: config.lm_studio.base_url,
            model: config.lm_studio.model,
            messages: buildClassificationPrompt({ filePath: row.path, stats: { size: row.size, ext: row.ext, families: row.families } }),
            maxTokens: 512,
          });
          row.ml = parseClassification(raw);
        } catch {
          row.ml = { stage: null, keep: false, family: '', confidence: 0, reason: 'lm-studio error', signals: [], raw: '' };
        }
      }
    }
  }

  const matrix = {};
  for (const row of candidates) {
    const h = row.heuristicStage;
    const m = row.ml?.stage ?? 'unparseable';
    matrix[h] = matrix[h] ?? {};
    matrix[h][m] = (matrix[h][m] ?? 0) + 1;
  }

  const summary = {
    ok: true,
    samplesPerStage,
    total: candidates.length,
    matrix,
    agreement: {
      primary: { heuristic: samplesPerStage, mlPrimary: candidates.filter((r) => r.heuristicStage === 'primary' && r.ml?.stage === 'primary').length },
      secondary: { heuristic: samplesPerStage, mlSecondary: candidates.filter((r) => r.heuristicStage === 'secondary' && r.ml?.stage === 'secondary').length },
      noise: { heuristic: samplesPerStage, mlNoise: candidates.filter((r) => r.heuristicStage === 'noise' && r.ml?.stage === 'noise').length },
    },
  };

  await writeJsonFile(path.join(outDir, 'validation-summary.json'), summary);
  await writeJsonFile(path.join(outDir, 'validation-samples.json'), candidates);
  console.log(JSON.stringify(summary, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

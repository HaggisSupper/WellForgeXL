import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { walkDocuments, writeJsonFile, buildClassificationPrompt, callLmStudioChat, classifyPath } from './pipeline.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const rootDir = process.argv[3] ?? path.resolve('G:\\My Drive\\Drilling Background');
const outPath = process.argv[4] ?? path.resolve('outputs/doc-rag/classification-index.json');
const config = JSON.parse(await readFile(configPath, 'utf8'));
const docs = await walkDocuments(rootDir);

const highValue = [];
const noise = [];

for (const filePath of docs.slice(0, 200)) {
  const families = classifyPath(filePath);
  const prompt = buildClassificationPrompt({ filePath, text: filePath });
  const ml = await callLmStudioChat({
    baseUrl: config.lm_studio.base_url,
    model: config.lm_studio.model,
    messages: prompt,
    maxTokens: 256,
  });
  let parsed = null;
  try {
    parsed = JSON.parse(ml?.choices?.[0]?.message?.content ?? ml?.content ?? '{}');
  } catch {
    parsed = null;
  }
  const keep = Boolean(parsed?.keep);
  const row = { filePath, families, ml, parsed };
  if (keep) highValue.push(row); else noise.push(row);
}

await writeJsonFile(outPath, { ok: true, rootDir, total: docs.length, sampled: 200, highValue, noise, config });
console.log(JSON.stringify({ ok: true, outPath, total: docs.length, sampled: 200, highValue: highValue.length, noise: noise.length }, null, 2));

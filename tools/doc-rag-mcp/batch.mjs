import path from 'node:path';
import { readFile } from 'node:fs/promises';
import { walkDocuments, classifyPath, extractDocumentStub, buildExtractionPrompt, callLmStudioChat, normalizeResult, writeJsonFile } from './pipeline.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const rootDir = process.argv[3] ?? path.resolve('G:\\My Drive\\Drilling Background');
const outDir = process.argv[4] ?? path.resolve('outputs/doc-rag');

const config = JSON.parse(await readFile(configPath, 'utf8'));
const docs = await walkDocuments(rootDir);
const results = [];

for (const filePath of docs.slice(0, 20)) {
  const families = classifyPath(filePath);
  const extracted = await extractDocumentStub(filePath);
  const prompt = buildExtractionPrompt({ filePath, family: families[0], mode: 'batch', text: extracted.text });
  const reply = await callLmStudioChat({
    baseUrl: config.lm_studio.base_url,
    model: config.lm_studio.model,
    messages: prompt,
  });
  const normalized = normalizeResult({ filePath, families, extracted, lmStudio: reply });
  results.push(normalized);
  const safe = filePath.replace(/[<>:"/\\|?*]+/g, '_');
  await writeJsonFile(path.join(outDir, `${safe}.json`), normalized);
}

await writeJsonFile(path.join(outDir, 'index.json'), { ok: true, rootDir, count: results.length, results });
console.log(JSON.stringify({ ok: true, rootDir, count: results.length, outDir }, null, 2));

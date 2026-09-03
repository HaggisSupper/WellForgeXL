import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { classifyPath, buildExtractionPrompt, callLmStudioChat, normalizeResult, buildClassificationPrompt, buildOkfPrompt, mergeOkfResult, parseClassification } from './pipeline.mjs';
import { extractWithDocling, extractWithOcr, extractWithVlm, chooseExtractionBackend } from './extractors.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const raw = await readFile(configPath, 'utf8');
const config = JSON.parse(raw);
const filePath = process.argv[3];

if (!filePath) {
  throw new Error('Usage: node tools/doc-rag-mcp/runner.mjs <config> <document-path>');
}

const families = classifyPath(filePath);
const preferred = config.docling ?? {};
const classification = await callLmStudioChat({
  baseUrl: config.lm_studio.base_url,
  model: config.lm_studio.model,
  messages: buildClassificationPrompt({ filePath }),
  maxTokens: 512,
});
const classificationJson = parseClassification(classification);
const keep = classificationJson.keep;
const backend = chooseExtractionBackend({ keep, confidence: classificationJson.confidence, doclingConfig: preferred });
if (backend === 'skip') {
  console.log(JSON.stringify({ filePath, families, keep: false, skipped: true, classification: classificationJson }, null, 2));
  process.exit(0);
}
const extracted = backend === 'ocr'
  ? await extractWithOcr(filePath, config)
  : backend === 'vlm'
    ? await extractWithVlm(filePath, config)
    : await extractWithDocling(filePath, config);
const prompt = buildExtractionPrompt({ filePath, family: families[0], mode: 'runner', text: extracted.text });
const result = await callLmStudioChat({
  baseUrl: config.lm_studio.base_url,
  model: config.lm_studio.model,
  messages: prompt,
});

const okfReply = await callLmStudioChat({
  baseUrl: config.lm_studio.base_url,
  model: config.lm_studio.model,
  messages: buildOkfPrompt({ filePath, family: families[0], text: extracted.text }),
  maxTokens: 1024,
});

console.log(JSON.stringify(normalizeResult({ filePath, families, extracted: mergeOkfResult({ extracted, okf: okfReply }), lmStudio: result }), null, 2));

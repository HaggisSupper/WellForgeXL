import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { classifyPath, readTextFallback, buildClassificationPrompt, callLmStudioChat, writeJsonFile } from './pipeline.mjs';

const configPath = process.argv[2] ?? path.resolve('data/doc-rag-mcp.config.json');
const targetPath = process.argv[3];
if (!targetPath) throw new Error('Usage: node tools/doc-rag-mcp/classify-front-end.mjs <config> <path>');

const config = JSON.parse(await readFile(configPath, 'utf8'));
const text = await readTextFallback(targetPath);
const heuristics = classifyPath(targetPath);
const prompt = buildClassificationPrompt({ filePath: targetPath, text });
const reply = await callLmStudioChat({
  baseUrl: config.lm_studio.base_url,
  model: config.lm_studio.model,
  messages: prompt,
  maxTokens: 512,
});

const result = {
  filePath: targetPath,
  heuristics,
  ml: reply,
};

console.log(JSON.stringify(result, null, 2));
if (process.argv.includes('--write')) {
  await writeJsonFile(`${targetPath}.classification.json`, result);
}

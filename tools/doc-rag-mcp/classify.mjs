import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { classifyPath, writeJsonFile } from './pipeline.mjs';

const input = process.argv[2] ?? process.argv[3];
if (!input) throw new Error('Usage: node tools/doc-rag-mcp/classify.mjs <path>');

const families = classifyPath(input);
const out = {
  path: input,
  basename: path.basename(input),
  families,
};

console.log(JSON.stringify(out, null, 2));

if (process.argv.includes('--write')) {
  await writeJsonFile(`${input}.classification.json`, out);
}

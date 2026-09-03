import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { classifyPath, extractDocumentStub, buildExtractionPrompt, buildOkfPrompt, callLmStudioChat, normalizeResult, mergeOkfResult } from './pipeline.mjs';

const server = new McpServer({
  name: 'wellforge-doc-rag',
  version: '0.2.0',
});

const config = {
  lmStudio: {
    baseUrl: process.env.LM_STUDIO_BASE_URL ?? 'http://127.0.0.1:1234/v1',
    model: process.env.LM_STUDIO_MODEL ?? 'local-model',
  },
};

server.tool('classify_document', {
  path: 'string',
}, async ({ path }) => ({
  content: [{ type: 'text', text: JSON.stringify({ ok: true, path, families: classifyPath(path) }, null, 2) }],
}));

server.tool('extract_document_text', {
  path: 'string',
  family: { type: 'string', optional: true },
}, async ({ path, family }) => {
  const extracted = await extractDocumentStub(path);
  const prompt = buildExtractionPrompt({ filePath: path, family: family ?? classifyPath(path)[0], mode: 'text', text: extracted.text });
  const reply = await callLmStudioChat({ baseUrl: config.lmStudio.baseUrl, model: config.lmStudio.model, messages: prompt });
  return {
    content: [{ type: 'text', text: JSON.stringify(normalizeResult({ filePath: path, families: classifyPath(path), extracted, lmStudio: reply }), null, 2) }],
  };
});

server.tool('okf_enrich_text', {
  path: 'string',
  text: 'string',
  family: { type: 'string', optional: true },
}, async ({ path, text, family }) => {
  const reply = await callLmStudioChat({
    baseUrl: config.lmStudio.baseUrl,
    model: config.lmStudio.model,
    messages: buildOkfPrompt({ filePath: path, family: family ?? classifyPath(path)[0], text }),
    maxTokens: 1024,
  });
  return {
    content: [{ type: 'text', text: JSON.stringify(mergeOkfResult({ extracted: { text, pages: [], tables: [], images: [], coordinates: [], warnings: [] }, okf: reply }), null, 2) }],
  };
});

await server.connect(new StdioServerTransport());

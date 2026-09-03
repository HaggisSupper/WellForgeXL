# Document RAG MCP

MCP-backed document ingestion scaffold for the WellForge document-first corpus pipeline.

## Entry points

- `npm run mcp:doc` — MCP server over stdio (classify, extract, OKF enrich)
- `npm run doc:run -- <config> <file>` — single-document runner
- `npm run doc:inventory -- <config> <corpus-root> <out-dir> <ml-limit> <batch-size>` — Phase 1 corpus inventory
- `npm run doc:validate -- <config> <manifest.jsonl> <out-dir> <samples-per-stage> <batch-size>` — routing validation

## Config

`data/doc-rag-mcp.config.json` sets the document policy and LM Studio endpoint:

- Docling-enabled extraction (backends stubbed)
- OCR / VLM fallback toggles
- LM Studio OpenAI-compatible endpoint and model name

## Phase 1: Corpus inventory and routing

The inventory script walks the corpus, applies heuristic family and stage routing, and optionally runs an LM Studio classifier in batches.

```bash
npm run doc:inventory
```

Output:

- `outputs/doc-rag/manifest.jsonl` — one row per file with heuristic routing
- `outputs/doc-rag/manifest-ml.jsonl` — rows that received an ML classification
- `outputs/doc-rag/inventory-summary.json` — counts and stats

Heuristic routing (current corpus):

- `primary`: 3,261 files — validation/oracle cases, core engineering docs, deep-pocket folders
- `secondary`: 5,716 files — supporting guides, training, related catalogs
- `noise`: 5,174 files — admin, marketing, icons, duplicates, unknown

### Validation

Run a stratified sample through the LM Studio classifier and compare with heuristic routing:

```bash
npm run doc:validate -- data/doc-rag-mcp.config.json outputs/doc-rag/manifest.jsonl outputs/doc-rag 10 3
```

Latest validation (30 files, 10 per heuristic stage):

- Noise rejection: 100% agreement
- Primary detection: 80% heuristic → ML primary
- Secondary bucket: mixed (some primary, some noise), consistent with expected catch-all behavior

## Current status

- Phase 1 inventory and routing is complete.
- ML classifier is validated and works in batched mode (batch size 3 recommended for current local model).
- Docling/OCR/VLM extraction backends remain stubs; Phase 2 will implement real extraction.

# Document RAG MCP

This repo now includes a small MCP-backed document ingestion scaffold for a document-first corpus pipeline.

## Entry points

- `npm run mcp:doc` — MCP server over stdio
- `npm run doc:run -- <config>` — local runner that loads the ingestion config

## Config

`data/doc-rag-mcp.config.json` sets the document policy:

- Docling-enabled extraction
- OCR enabled
- coordinate preservation
- image preservation
- VLM fallback toggle
- LM Studio OpenAI-compatible endpoint

## Current status

The server and runner are scaffolded and will be expanded into real Docling/OCR/VLM integration next.

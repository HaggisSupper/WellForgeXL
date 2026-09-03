import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import fg from 'fast-glob';
import { request } from 'undici';

const DOC_EXTENSIONS = new Set(['.pdf', '.docx', '.pptx', '.xlsx', '.xlsm', '.xlsb', '.txt', '.html', '.htm', '.png', '.jpg', '.jpeg', '.tif', '.tiff']);

export function classifyPath(filePath) {
  const lower = filePath.toLowerCase();
  const families = [];
  const add = (name, predicate) => { if (predicate) families.push(name); };
  add('tnd', /torque and drag|tnd|hook load|tripping|stiff-string|load vs torque|torque calculator|make up torque/.test(lower));
  add('bha', /bha|bottom hole assembly|stabilizer|jar|motor|critical speed|vibration|drillstring design|bending strength/.test(lower));
  add('hydraulics', /hydraul|mud|flow|ecd|pressure loss|nozzle|split-flow|well control|rheology|sheargenius|microflux/.test(lower));
  add('trajectory', /survey|wellplan|wellz|hawkeye|kellydown|anti-collision|trajectory|directional|min curv|dogleg/.test(lower));
  add('reference', /catalog|handbook|reference|standard|spec|api|iso|norsok|dnv|calculator/.test(lower));
  add('training', /manual|guide|tutorial|how to|training|instructor|participants|quick start|work instructions/.test(lower));
  add('validation', /validation|test case|case|sample projects|results|trx|lvr|oracle|benchmark/.test(lower));
  return families.length ? [...new Set(families)] : ['unknown'];
}

export function isOfficeOrPdf(filePath) {
  return DOC_EXTENSIONS.has(path.extname(filePath).toLowerCase());
}

export async function readTextFallback(filePath) {
  try {
    return await readFile(filePath, 'utf8');
  } catch {
    return '';
  }
}

export async function walkDocuments(rootDir, patterns = ['**/*.{pdf,docx,pptx,xlsx,xlsm,xlsb,txt,html,htm,png,jpg,jpeg,tif,tiff}']) {
  return fg(patterns, {
    cwd: rootDir,
    onlyFiles: true,
    unique: true,
    dot: false,
    absolute: true,
    suppressErrors: true,
  });
}

export async function writeJsonFile(filePath, value) {
  await mkdir(path.dirname(filePath), { recursive: true });
  await writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

export async function extractDocumentStub(filePath) {
  const text = await readTextFallback(filePath);
  return {
    text,
    pages: [],
    tables: [],
    images: [],
    coordinates: [],
    warnings: text ? [] : ['binary-or-non-text input needs Docling/OCR'],
  };
}

export async function callLmStudioChat({ baseUrl, model, messages, temperature = 0.1, maxTokens = 1024 }) {
  const url = new URL('/chat/completions', baseUrl).toString();
  const body = { model, messages, temperature, max_tokens: maxTokens };
  const { body: resBody, statusCode } = await request(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  const text = await resBody.text();
  if (statusCode >= 400) throw new Error(`LM Studio error ${statusCode}: ${text}`);
  return JSON.parse(text);
}

export function buildExtractionPrompt({ filePath, family, mode, text }) {
  return [
    { role: 'system', content: 'You extract structured information from engineering documents. Return concise JSON only.' },
    { role: 'user', content: JSON.stringify({ filePath, family, mode, textSample: text.slice(0, 16000) }) },
  ];
}

export function normalizeResult({ filePath, families, extracted, lmStudio }) {
  return {
    filePath,
    families,
    text: extracted.text ?? '',
    pages: extracted.pages ?? [],
    tables: extracted.tables ?? [],
    images: extracted.images ?? [],
    coordinates: extracted.coordinates ?? [],
    warnings: extracted.warnings ?? [],
    lmStudio,
  };
}

export function buildClassificationPrompt({ filePath, text = '' }) {
  return [
    {
      role: 'system',
      content: [
        'Classify drilling-engineering documents for RAG value.',
        'Return JSON only with fields: keep (boolean), family (string), confidence (0-1), reason (string), signals (string[]), stage (string).',
        'Prefer keep=true only when the file is likely a primary or secondary ingest target.',
        'Primary ingest targets: validation/oracle cases, saved runs, validation workbooks, direct theory papers, mechanism references, core manuals, calculator specs, trajectory listings.',
        'Secondary ingest targets: supporting guides, training docs with worked examples, related catalogs, and legacy simulation artifacts that expose equations or outputs.',
        'Reject noise: marketing, cover pages, blank scans, generic admin, duplicates without substance, and superficial indexes.',
      ].join(' '),
    },
    {
      role: 'user',
      content: JSON.stringify({ filePath, textSample: text.slice(0, 8000) }),
    },
  ];
}

export function buildOkfPrompt({ filePath, family, text }) {
  return [
    {
      role: 'system',
      content: [
        'You perform a secondary OKF enrichment pass on extracted engineering text.',
        'Return JSON only with fields: normalized_text, keywords, entities, formulas, tables, confidence, notes, preserve_sections.',
        'Preserve worked examples, table semantics, formulas, units, and section ordering.',
        'Normalize text without flattening important structure.',
      ].join(' '),
    },
    {
      role: 'user',
      content: JSON.stringify({ filePath, family, textSample: text.slice(0, 16000) }),
    },
  ];
}

export function mergeOkfResult({ extracted, okf }) {
  return {
    ...extracted,
    okf,
    text: okf?.normalized_text || extracted.text || '',
    keywords: okf?.keywords ?? [],
    entities: okf?.entities ?? [],
    formulas: okf?.formulas ?? [],
    okfConfidence: okf?.confidence ?? null,
  };
}

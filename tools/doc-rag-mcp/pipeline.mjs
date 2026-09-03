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

const NOISE_PATTERNS = [
  /\\\$recycle\.bin/i,
  /\\thumbs\.db$/i,
  /\\desktop\.ini$/i,
  /\\__pycache__\\/i,
  /\\\.old\\/i,
  /\\backup\\/i,
  /\\copy of /i,
  /\\\(\d+\)\./,
  /\\icon/i,
  /\\\.tmp$/i,
  /\\\.temp$/i,
  /\\~\$/,
];

const NOISE_EXTS = new Set(['.ico', '.lnk', '.db', '.ini', '.ds_store']);

export function isNoiseByPath(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  if (NOISE_EXTS.has(ext)) return true;
  const lower = filePath.toLowerCase();
  return NOISE_PATTERNS.some((p) => p.test(lower));
}

export function topFolder(relPath) {
  const normalized = relPath.replace(/^[\\/]+/, '');
  const slash = normalized.indexOf('\\') >= 0 ? normalized.indexOf('\\') : normalized.indexOf('/');
  return slash >= 0 ? normalized.slice(0, slash) : normalized;
}

const PRIMARY_FOLDERS = new Set([
  'drilling calcs 2009',
  'bha analysis',
  'drilling engineering',
  'comanchecalcs',
  'referencecalculators',
  'wellz',
  'ddsim',
  'torque and drag',
  'hydraulics models',
  'pipe handbooks',
  'vibration primer',
  'vibration documents',
]);

export function heuristicStage(filePath, families = [], relPath = '') {
  const lower = filePath.toLowerCase();
  if (isNoiseByPath(filePath)) {
    return { stage: 'noise', keep: false, reason: 'heuristic noise pattern or extension' };
  }

  const topFolderName = topFolder(relPath || lower).toLowerCase();
  const inPrimaryFolder = PRIMARY_FOLDERS.has(topFolderName);

  if (/agenda|minutes|managers meeting|meeting update|cover page|flyer|brochure|advertisement|press release/.test(lower)) {
    return { stage: 'noise', keep: false, reason: 'admin/marketing/cover material' };
  }

  const isValidation = /validation|test case|saved|oracle|benchmark|results|trx|lvr|sample projects/.test(lower);
  const isCoreDoc = /handbook|companion|paper|theory|mechanism|equation|spec|datasheet|calc|workbook|model|simulation|index/.test(lower);
  const isGuide = /manual|guide|tutorial|how to|training|instructor|participants|quick start|work instructions/.test(lower);
  const isActiveFamily = families.some((f) => ['tnd', 'bha', 'hydraulics', 'trajectory', 'reference'].includes(f));

  if (families.includes('validation') || isValidation) {
    return { stage: 'primary', keep: true, reason: 'validation/oracle/case material' };
  }
  if (isActiveFamily && inPrimaryFolder) {
    return { stage: 'primary', keep: true, reason: 'document in known high-value folder' };
  }
  if ((families.includes('tnd') || families.includes('bha') || families.includes('hydraulics') || families.includes('trajectory')) && isCoreDoc) {
    return { stage: 'primary', keep: true, reason: 'core engineering document for active family' };
  }
  if (families.includes('reference') && isCoreDoc) {
    return { stage: 'primary', keep: true, reason: 'reference spec/datasheet/handbook with structural data' };
  }
  if (isGuide && isActiveFamily) {
    return { stage: 'secondary', keep: true, reason: 'supporting guide/manual for active family' };
  }
  if (families.includes('training')) {
    return { stage: 'secondary', keep: true, reason: 'training material' };
  }
  if (families.includes('unknown')) {
    return { stage: 'noise', keep: false, reason: 'unknown family with no strong signals' };
  }
  return { stage: 'secondary', keep: true, reason: 'related material of unclear priority' };
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

export async function loadJsonl(filePath) {
  try {
    const text = await readFile(filePath, 'utf8');
    return text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        try {
          return JSON.parse(line);
        } catch {
          return null;
        }
      })
      .filter(Boolean);
  } catch {
    return [];
  }
}

export async function writeJsonl(filePath, rows) {
  await mkdir(path.dirname(filePath), { recursive: true });
  const lines = rows.map((r) => JSON.stringify(r));
  await writeFile(filePath, lines.join('\n') + (lines.length ? '\n' : ''), 'utf8');
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
  const url = new URL('chat/completions', baseUrl.replace(/\/$/, '') + '/').toString();
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

export function buildClassificationPrompt({ filePath, text = '', stats = {} }) {
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
      content: JSON.stringify({ filePath, stats, textSample: text.slice(0, 8000) }),
    },
  ];
}

export function parseClassification(mlResponse) {
  const raw = mlResponse?.choices?.[0]?.message?.content ?? mlResponse?.content ?? '{}';
  let cleaned = raw.trim();
  const fenceMatch = cleaned.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (fenceMatch) cleaned = fenceMatch[1].trim();
  if (!cleaned.startsWith('{')) {
    const jsonStart = cleaned.indexOf('{');
    if (jsonStart >= 0) cleaned = cleaned.slice(jsonStart);
  }
  try {
    const parsed = JSON.parse(cleaned);
    const stage = ['primary', 'secondary', 'noise'].includes(parsed?.stage) ? parsed.stage : null;
    return {
      stage,
      keep: stage === 'primary' || stage === 'secondary',
      family: String(parsed?.family ?? ''),
      confidence: Math.min(1, Math.max(0, Number(parsed?.confidence ?? 0))),
      reason: String(parsed?.reason ?? ''),
      signals: Array.isArray(parsed?.signals) ? parsed.signals.map(String) : [],
      raw,
    };
  } catch {
    return { stage: null, keep: false, family: '', confidence: 0, reason: 'unparseable response', signals: [], raw };
  }
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

export function buildBatchClassificationPrompt(items) {
  return [
    {
      role: 'system',
      content: [
        'Classify a batch of drilling-engineering files for a RAG ingest pipeline.',
        'Return JSON only: an object keyed by the exact file path, where each value has exactly these fields: stage (string), keep (boolean), family (string), confidence (number 0-1), reason (string), signals (string[]).',
        'stage MUST be one of: "primary", "secondary", or "noise".',
        'primary = validation/oracle cases, saved runs, validation workbooks, direct theory papers, mechanism references, core manuals, calculator specs, trajectory listings.',
        'secondary = supporting guides, training docs with worked examples, related catalogs, legacy simulation artifacts that expose equations or outputs.',
        'noise = marketing, cover pages only, blank scans, generic admin, duplicates without substance, superficial indexes, icon/image assets.',
        'Be strict: prefer noise unless there is concrete engineering value. Include every key from the input.',
      ].join(' '),
    },
    {
      role: 'user',
      content: JSON.stringify({ files: items.map((item) => ({ path: item.path, ext: item.ext, size: item.size, families: item.families, heuristicStage: item.heuristicStage })) }),
    },
  ];
}

export function parseBatchClassification(rawText, paths) {
  let cleaned = rawText.trim();
  const fenceMatch = cleaned.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (fenceMatch) cleaned = fenceMatch[1].trim();
  if (!cleaned.startsWith('{')) {
    const jsonStart = cleaned.indexOf('{');
    if (jsonStart >= 0) cleaned = cleaned.slice(jsonStart);
  }
  let parsed;
  try {
    parsed = JSON.parse(cleaned);
  } catch {
    const fallback = new Map();
    for (const p of paths) {
      fallback.set(p, { stage: null, keep: false, family: '', confidence: 0, reason: 'unparseable batch response', signals: [], raw: rawText });
    }
    return fallback;
  }
  const map = new Map();
  for (const p of paths) {
    const entry = parsed?.[p];
    const stage = ['primary', 'secondary', 'noise'].includes(entry?.stage) ? entry.stage : null;
    map.set(p, {
      stage,
      keep: stage === 'primary' || stage === 'secondary',
      family: String(entry?.family ?? ''),
      confidence: Math.min(1, Math.max(0, Number(entry?.confidence ?? 0))),
      reason: String(entry?.reason ?? ''),
      signals: Array.isArray(entry?.signals) ? entry.signals.map(String) : [],
      raw: JSON.stringify(entry ?? {}),
    });
  }
  return map;
}

export async function classifyBatch(items, config, { maxTokens = 1536 } = {}) {
  const prompt = buildBatchClassificationPrompt(items);
  const raw = await callLmStudioChat({
    baseUrl: config.lm_studio.base_url,
    model: config.lm_studio.model,
    messages: prompt,
    maxTokens,
  });
  const rawText = raw?.choices?.[0]?.message?.content ?? raw?.content ?? '{}';
  return parseBatchClassification(rawText, items.map((i) => i.path));
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

export async function extractWithDocling(filePath, config) {
  return {
    backend: 'docling',
    filePath,
    text: '',
    pages: [],
    tables: [],
    images: [],
    coordinates: [],
    warnings: ['docling backend not yet installed in this scaffold'],
    config,
  };
}

export async function extractWithOcr(filePath, config) {
  return {
    backend: 'tesseract',
    filePath,
    text: '',
    pages: [],
    tables: [],
    images: [],
    coordinates: [],
    warnings: ['ocr backend not yet installed in this scaffold'],
    config,
  };
}

export async function extractWithVlm(filePath, config) {
  return {
    backend: 'vlm',
    filePath,
    text: '',
    pages: [],
    tables: [],
    images: [],
    coordinates: [],
    warnings: ['vlm fallback not yet installed in this scaffold'],
    config,
  };
}

export function chooseExtractionBackend({ keep, confidence, doclingConfig }) {
  if (!keep) return 'skip';
  if (confidence >= 0.8 && doclingConfig?.enable_ocr) return 'ocr';
  if (confidence >= 0.5 && doclingConfig?.enable_tables) return 'docling';
  return doclingConfig?.fallback_vlm ? 'vlm' : 'docling';
}

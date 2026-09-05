import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function runPythonExtractor(filePath) {
  return new Promise((resolve, reject) => {
    const script = path.join(__dirname, 'extract.py');
    const child = spawn('python', [script, filePath], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => { stdout += chunk; });
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`extract.py exited ${code}: ${stderr || stdout}`));
        return;
      }
      try {
        const parsed = JSON.parse(stdout);
        resolve(parsed);
      } catch (err) {
        reject(new Error(`invalid JSON from extract.py: ${err.message}\n${stdout}`));
      }
    });
  });
}

export async function extractWithDocling(filePath, config) {
  const extracted = await runPythonExtractor(filePath);
  return { ...extracted, backend: 'python-docling' };
}

export async function extractWithOcr(filePath, config) {
  const extracted = await runPythonExtractor(filePath);
  if (!extracted.text && /\.(png|jpg|jpeg|tif|tiff)$/i.test(filePath)) {
    return { ...extracted, backend: 'python-ocr', warnings: [...extracted.warnings, 'ocr result empty'] };
  }
  return { ...extracted, backend: 'python-ocr' };
}

export async function extractWithVlm(filePath, config) {
  const extracted = await runPythonExtractor(filePath);
  return { ...extracted, backend: 'python-vlm' };
}

export function chooseExtractionBackend({ keep, confidence, doclingConfig }) {
  if (!keep) return 'skip';
  if (confidence >= 0.8 && doclingConfig?.enable_ocr) return 'ocr';
  if (confidence >= 0.5 && doclingConfig?.enable_tables) return 'docling';
  return doclingConfig?.fallback_vlm ? 'vlm' : 'docling';
}

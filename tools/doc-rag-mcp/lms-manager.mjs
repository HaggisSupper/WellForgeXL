#!/usr/bin/env node
/**
 * LM Studio model manager for headless operation.
 * Manages model loading/unloading based on VRAM constraints.
 * RTX 4050: 6GB VRAM → use q4_k_m (2.78GB) or q5_k_m (3.16GB)
 */

import { spawn, execSync } from 'node:child_process';
import { request } from 'undici';

const SERVER_URL = 'http://127.0.0.1:1234/v1';
const VRAM_MB = 6144; // 6GB for RTX 4050
const RESERVE_MB = 512; // Reserve 512MB for buffer

const MODELS = {
  'qwen3.8-4b-distill@q4_k_m': { size_mb: 2785, quality: 'low-med' },
  'qwen3.8-4b-distill@q5_k_m': { size_mb: 3160, quality: 'medium' },
  'qwen3.8-4b-distill@q8_0': { size_mb: 4608, quality: 'high' },
  'google/gemma-4-e2b': { size_mb: 4410, quality: 'high' },
  'prism-ml/bonsai-27b': { size_mb: 4730, quality: 'high' },
};

const PREFERRED_ORDER = ['qwen3.8-4b-distill@q5_k_m', 'qwen3.8-4b-distill@q4_k_m'];

async function checkServer() {
  try {
    const { body: resBody, statusCode } = await request(`${SERVER_URL}/models`, { method: 'GET', connect_timeout: 1000 });
    if (statusCode === 200) return true;
  } catch {}
  return false;
}

async function getLoadedModels() {
  try {
    const { body: resBody, statusCode } = await request(`${SERVER_URL}/models`, { method: 'GET' });
    const text = await resBody.text();
    const data = JSON.parse(text);
    return data.data?.map((m) => m.id) || [];
  } catch (e) {
    console.error('Failed to fetch models:', e.message);
    return [];
  }
}

function startServer() {
  console.error('[lms-manager] Starting LM Studio server headlessly...');
  const proc = spawn('lms', ['server', 'start', '--port', '1234'], {
    stdio: 'ignore',
    detached: true,
  });
  proc.unref();
  return new Promise((resolve) => {
    const interval = setInterval(async () => {
      if (await checkServer()) {
        clearInterval(interval);
        console.error('[lms-manager] Server is ready');
        resolve();
      }
    }, 500);
    setTimeout(() => {
      clearInterval(interval);
      resolve();
    }, 5000);
  });
}

function selectBestModel(vramAvailable = VRAM_MB - RESERVE_MB) {
  for (const modelId of PREFERRED_ORDER) {
    if (MODELS[modelId] && MODELS[modelId].size_mb <= vramAvailable) {
      return modelId;
    }
  }
  return PREFERRED_ORDER[0];
}

async function loadModel(modelId) {
  console.error(`[lms-manager] Loading ${modelId}...`);
  const loaded = await getLoadedModels();
  if (loaded.includes(modelId)) {
    console.error(`[lms-manager] ${modelId} already loaded`);
    return true;
  }

  return new Promise((resolve) => {
    const proc = spawn('lms', ['load', modelId, '-y'], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';
    proc.stdout?.on('data', (chunk) => { stdout += chunk; });
    proc.stderr?.on('data', (chunk) => { stderr += chunk; });
    proc.on('close', (code) => {
      if (code === 0) {
        console.error(`[lms-manager] ${modelId} loaded successfully`);
        resolve(true);
      } else {
        console.error(`[lms-manager] Load failed for ${modelId}`);
        resolve(false);
      }
    });
    setTimeout(() => {
      proc.kill();
      resolve(false);
    }, 30000); // 30s timeout
  });
}

async function unloadModel(modelId) {
  console.error(`[lms-manager] Unloading ${modelId}...`);
  return new Promise((resolve) => {
    const proc = spawn('lms', ['unload', modelId, '-y'], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    proc.on('close', (code) => {
      if (code === 0) {
        console.error(`[lms-manager] ${modelId} unloaded`);
        resolve(true);
      } else {
        resolve(false);
      }
    });
  });
}

async function ensureModelLoaded(modelId = null) {
  if (!modelId) {
    modelId = selectBestModel();
  }

  const loaded = await getLoadedModels();
  const isLoaded = loaded.includes(modelId);

  if (isLoaded) {
    console.error(`[lms-manager] ${modelId} is ready`);
    return modelId;
  }

  console.error(`[lms-manager] Unloading all models...`);
  for (const id of loaded) {
    if (MODELS[id]) {
      await unloadModel(id);
    }
  }

  if (!(await loadModel(modelId))) {
    console.error(`[lms-manager] Failed to load ${modelId}`);
    return null;
  }

  return modelId;
}

// CLI
async function main() {
  const command = process.argv[2];

  if (command === 'start') {
    await startServer();
  } else if (command === 'ensure-loaded') {
    const modelId = process.argv[3];
    const loaded = await ensureModelLoaded(modelId);
    console.log(loaded || 'null');
  } else if (command === 'get-loaded') {
    const loaded = await getLoadedModels();
    console.log(JSON.stringify(loaded));
  } else if (command === 'select-best') {
    const best = selectBestModel();
    console.log(best);
  } else {
    console.error('Usage: lms-manager.mjs <start|ensure-loaded|get-loaded|select-best>');
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

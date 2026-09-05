import { readFile } from 'node:fs/promises';
import path from 'node:path';

export async function loadManifest(manifestPath) {
  const raw = await readFile(manifestPath, 'utf8');
  const lines = raw.trim().split(/\r?\n/);
  const header = lines.shift();
  const columns = header.split(',');
  return lines.map((line) => {
    const values = line.split(',');
    const row = {};
    for (let i = 0; i < columns.length; i += 1) row[columns[i]] = values[i];
    return row;
  });
}

export function filterDocumentRows(rows, families) {
  return rows.filter((row) => families.some((family) => row.path.includes(family)));
}

export function pathFromManifestRow(rootDir, row) {
  return path.join(rootDir, row.path.replace(/^\^?/, ''));
}

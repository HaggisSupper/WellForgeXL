import fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const directory = path.join(root, 'VBA');
const files = (await fs.readdir(directory)).filter((name) => name.endsWith('.bas')).sort();
const failures = [];

for (const file of files) {
  const text = await fs.readFile(path.join(directory, file), 'utf8');
  const lines = text.replaceAll('\r\n', '\n').split('\n');
  if (!lines.some((line) => /^Option Explicit\s*$/i.test(line.trim()))) failures.push(`${file}: missing Option Explicit`);
  let procedure = null;
  lines.forEach((raw, index) => {
    const line = raw.trim();
    if (raw.length > 1023) failures.push(`${file}:${index + 1}: line exceeds VBA 1023-character limit`);
    const start = line.match(/^(?:(?:Public|Private|Friend)\s+)?(?:Static\s+)?(Sub|Function|Property\s+(?:Get|Let|Set))\s+([A-Za-z_][A-Za-z0-9_]*)\b/i);
    const end = line.match(/^End\s+(Sub|Function|Property)\s*$/i);
    if (start) {
      if (procedure) failures.push(`${file}:${index + 1}: nested procedure ${start[2]} inside ${procedure.name}`);
      procedure = { kind: start[1].toLowerCase().startsWith('property') ? 'property' : start[1].toLowerCase(), name: start[2], line: index + 1 };
    }
    if (end) {
      if (!procedure) failures.push(`${file}:${index + 1}: ${line} without procedure`);
      else {
        const kind = end[1].toLowerCase();
        if (kind !== procedure.kind) failures.push(`${file}:${index + 1}: ${line} closes ${procedure.kind} ${procedure.name}`);
        procedure = null;
      }
    }
  });
  if (procedure) failures.push(`${file}:${procedure.line}: unclosed ${procedure.kind} ${procedure.name}`);
}

if (failures.length) {
  process.stderr.write(`${failures.join('\n')}\n`);
  process.exitCode = 1;
} else {
  process.stdout.write(`VBA structural lint passed for ${files.length} modules.\n`);
}

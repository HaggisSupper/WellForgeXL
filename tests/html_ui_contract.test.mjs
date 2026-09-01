import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const uiRoot = path.join(root, 'HTML UI');

test('HTML UI is a portable multi-file website with the required engine views', async () => {
  const required = ['index.html', 'styles.css', 'app.js', 'README.md', path.join('data', 'chart-method.json')];
  for (const file of required) await fs.access(path.join(uiRoot, file));
  const html = await fs.readFile(path.join(uiRoot, 'index.html'), 'utf8');
  for (const view of ['overview', 'trajectory', 'bha', 'hydraulics', 'torque', 'data']) {
    assert.match(html, new RegExp(`data-view="${view}"`), `${view} tab is missing`);
    assert.match(html, new RegExp(`data-panel="${view}"`), `${view} panel is missing`);
  }
  assert.match(html, /role="tablist"/);
  assert.match(html, /type="module" src="app\.js"/);
});

test('HTML UI contains no prohibited vendor references', async () => {
  const files = ['index.html', 'styles.css', 'app.js', 'README.md', path.join('data', 'chart-method.json')];
  const source = (await Promise.all(files.map((file) => fs.readFile(path.join(uiRoot, file), 'utf8')))).join('\n');
  assert.doesNotMatch(source, /weatherford|\bwft\b|\bk1\b|\bk2\b/i);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const uiRoot = path.join(root, 'HTML UI');

test('HTML UI is a portable multi-file website with the required engine views', async () => {
  const required = ['index.html', 'tokens.css', 'styles.css', 'app.js', 'README.md', 'Launch-WellForgeUI.bat', path.join('data', 'chart-method.json')];
  for (const file of required) await fs.access(path.join(uiRoot, file));
  const html = await fs.readFile(path.join(uiRoot, 'index.html'), 'utf8');
  for (const view of ['overview', 'trajectory', 'bha', 'hydraulics', 'torque', 'data']) {
    assert.match(html, new RegExp(`data-view="${view}"`), `${view} tab is missing`);
    assert.match(html, new RegExp(`data-panel="${view}"`), `${view} panel is missing`);
  }
  assert.match(html, /role="tablist"/);
  assert.match(html, /type="module" src="app\.js"/);
  assert.match(html, /id="app-launcher"/);
  assert.match(html, /id="launcher-search"/);
  assert.match(html, /tokens\.css/);
  await fs.access(path.join(uiRoot, 'vendor', 'tabulator.min.js'));
  await fs.access(path.join(uiRoot, 'vendor', 'tabulator_midnight.min.css'));
  assert.match(html, /vendor\/tabulator\.min\.js/);
  const app = await fs.readFile(path.join(uiRoot, 'app.js'), 'utf8');
  const tokens = await fs.readFile(path.join(uiRoot, 'tokens.css'), 'utf8');
  const styles = await fs.readFile(path.join(uiRoot, 'styles.css'), 'utf8');
  assert.match(tokens, /--color-surface-canvas:\s*#111315/);
  assert.match(tokens, /--space-1:\s*4px/);
  assert.match(styles, /body\s*\{\s*background:\s*var\(--color-surface-canvas\)/);
  assert.match(app, /new window\.Tabulator/);
  assert.match(app, /mountGrid\("engine-table"/);
  assert.match(app, /mountGrid\("survey-table"/);
  assert.match(app, /mountGrid\("nozzle-table"/);
  assert.match(app, /openLauncher/);
  assert.match(app, /event\.key\.toLowerCase\(\) === "k"/);
  assert.match(app, /launcherReturnFocus/);
  assert.match(app, /event\.key === "Tab"/);
});

test('HTML UI contains no prohibited vendor references', async () => {
  const files = ['index.html', 'tokens.css', 'styles.css', 'app.js', 'README.md', 'Launch-WellForgeUI.bat', path.join('data', 'chart-method.json')];
  const source = (await Promise.all(files.map((file) => fs.readFile(path.join(uiRoot, file), 'utf8')))).join('\n');
  assert.doesNotMatch(source, /weatherford|\bwft\b|\bk1\b|\bk2\b/i);
});

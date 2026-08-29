import test from 'node:test';
import assert from 'node:assert/strict';
import JSZip from 'jszip';

test('BHA model produces formula-based bending and polar coordinates for each WOB case', async () => {
  const { bhaFormulaPlan } = await import('../src/build_bha.mjs');
  const plan = bhaFormulaPlan();
  assert.match(plan.firstFrequency, /SQRT/);
  assert.match(plan.bendingStress, /M6\*D6/);
  assert.match(plan.polarX, /COS/);
  assert.match(plan.polarY, /SIN/);
});

test('BHA presents Young modulus in engineering-scale units while calculating in Pa', async () => {
  const { buildBhaWorkbook } = await import('../src/build_bha.mjs');
  const workbook = buildBhaWorkbook();
  const inputs = workbook.worksheets.getItem('Inputs');
  const calc = workbook.worksheets.getItem('Calc');
  assert.equal(inputs.getRange('A7').values[0][0], 'Young modulus');
  assert.equal(inputs.getRange('B7').values[0][0], 206.84271879504);
  assert.equal(inputs.getRange('C7').values[0][0], 'GPa');
  assert.deepEqual(inputs.getRange('C7').dataValidation.rule.values, ['GPa', 'MPa', 'Pa', 'Mpsi', 'psi']);
  assert.match(calc.getRange('H6').formulas[0][0], /Inputs!\$B\$7\*IF\(Inputs!\$C\$7="GPa",1E9/);
});

test('BHA exports a PolarPlotter-style radar and XY scatter combination with independent axes', async () => {
  const { buildBhaWorkbook } = await import('../src/build_bha.mjs');
  const { exportExchangeXlsx } = await import('../src/exchange/export_exchange_xlsx.mjs');
  const zip = await JSZip.loadAsync((await exportExchangeXlsx(buildBhaWorkbook())).data);
  const chartPaths = Object.keys(zip.files).filter((name) => /^xl\/drawings\/charts\/chart\d+\.xml$/.test(name));
  const charts = await Promise.all(chartPaths.map((name) => zip.file(name).async('string')));
  const polar = charts.find((xml) => xml.includes('WOB/toolface polar response'));
  const rose = charts.find((xml) => xml.includes('Toolface / WOB rose plot'));
  assert.ok(polar, 'polar response chart');
  assert.ok(rose, 'decision-sheet rose plot');
  assert.match(rose, /<c:scatterStyle val="lineMarker"\s*\/>/,
    'the superimposed WOB traces must connect into rose curves');
  assert.ok((rose.match(/<a:alpha val="65000"\s*\/>/g) ?? []).length >= 2,
    'each WOB rose trace must retain 35% transparency when superimposed');
  assert.match(polar, /<c:radarChart>/);
  assert.match(polar, /<c:scatterChart>/);
  const radarIds = [...polar.matchAll(/<c:radarChart>[\s\S]*?<\/c:radarChart>/g)]
    .flatMap(([xml]) => [...xml.matchAll(/<c:axId val="([^"]+)"/g)].map((match) => match[1]));
  const scatterIds = [...polar.matchAll(/<c:scatterChart>[\s\S]*?<\/c:scatterChart>/g)]
    .flatMap(([xml]) => [...xml.matchAll(/<c:axId val="([^"]+)"/g)].map((match) => match[1]));
  assert.equal(new Set([...radarIds, ...scatterIds]).size, 4);
});

test('BHA export contains every OOXML part declared by its content-type manifest', async () => {
  const { buildBhaWorkbook } = await import('../src/build_bha.mjs');
  const { exportExchangeXlsx } = await import('../src/exchange/export_exchange_xlsx.mjs');
  const zip = await JSZip.loadAsync((await exportExchangeXlsx(buildBhaWorkbook())).data);
  const contentTypes = await zip.file('[Content_Types].xml').async('string');
  const declaredParts = [...contentTypes.matchAll(/<Override\b[^>]*PartName="([^"]+)"[^>]*\/>/g)]
    .map((match) => match[1].replace(/^\//, ''));
  const missingParts = declaredParts.filter((partName) => !zip.file(partName));
  assert.deepEqual(missingParts, []);
});

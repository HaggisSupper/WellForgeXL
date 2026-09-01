import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import JSZip from 'jszip';

const root = fileURLToPath(new URL('..', import.meta.url));

function chartTitle(xml) {
  return [...xml.matchAll(/<a:t>([^<]*)<\/a:t>/g)].map((match) => match[1]).join(' ');
}

async function chartsIn(workbookName) {
  const bytes = await fs.readFile(path.join(root, 'outputs', workbookName));
  const zip = await JSZip.loadAsync(bytes);
  const chartPaths = Object.keys(zip.files)
    .filter((name) => /^xl\/drawings\/charts\/chart\d+\.xml$/.test(name))
    .sort((left, right) => left.localeCompare(right, undefined, { numeric: true }));
  return Promise.all(chartPaths.map(async (name) => {
    const xml = await zip.file(name).async('string');
    return { name, title: chartTitle(xml), xml };
  }));
}

function assertDepthRoadmap(chart, label) {
  assert.match(chart.xml, /<c:scatterChart>/, `${label} must be a true XY scatter chart`);
  assert.match(chart.xml, /<c:scatterStyle val="lineMarker"\s*\/>/, `${label} must connect the depth profile rather than show isolated markers`);
  assert.match(chart.xml, /<c:xVal>/, `${label} must put the calculated response on X`);
  assert.match(chart.xml, /<c:yVal>/, `${label} must put measured depth on Y`);
  assert.match(chart.xml, /<c:orientation val="maxMin"\s*\/>/, `${label} must show zero MD at the top`);
  const axes = [...chart.xml.matchAll(/<c:valAx>([\s\S]*?)<\/c:valAx>/g)].map((match) => match[1]);
  assert.ok(
    axes.some((axis) => /<c:orientation val="maxMin"\s*\/>/.test(axis) && /<c:axPos val="l"\s*\/>/.test(axis)),
    `${label} must keep the reversed depth axis vertical on the left`,
  );
  assert.ok(
    axes.some((axis) => /<c:orientation val="minMax"\s*\/>/.test(axis) && /<c:axPos val="t"\s*\/>/.test(axis)),
    `${label} must place the calculated-response axis horizontally at the top`,
  );
}

test('every torque-drag chart is a response-X / reversed-depth-Y roadmap', async () => {
  const charts = await chartsIn('Torque_Drag_and_Buckling_SI.xlsx');
  assert.equal(charts.length, 31, 'torque-drag roadmap inventory changed');
  for (const chart of charts) assertDepthRoadmap(chart, `T&D ${chart.title || chart.name}`);
});

test('hydraulics publishes pressure, ECD, and velocity roadmaps against reversed MD', async () => {
  const charts = await chartsIn('Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx');
  const depthCharts = charts.filter(({ title }) => /\bvs MD\b/i.test(title));
  assert.equal(depthCharts.length, 6, 'hydraulics must expose three base and three sensitivity depth roadmaps');
  for (const chart of depthCharts) assertDepthRoadmap(chart, `Hydraulics ${chart.title}`);

  const pressure = depthCharts.find(({ title }) => /pressure/i.test(title));
  const ecd = depthCharts.find(({ title }) => /ECD/i.test(title));
  const velocity = depthCharts.find(({ title }) => /velocity/i.test(title));
  assert.ok(pressure && ecd && velocity, 'pressure, ECD, and velocity depth roadmaps are required');
  assert.ok((pressure.xml.match(/<c:ser>/g) ?? []).length >= 3, 'pressure roadmap must compare multiple pressure components');
  assert.ok((ecd.xml.match(/<c:ser>/g) ?? []).length >= 3, 'ECD roadmap must compare static, dynamic, and limit series');
  assert.ok((velocity.xml.match(/<c:ser>/g) ?? []).length >= 2, 'velocity roadmap must include its operating threshold');

  const nozzle = charts.find(({ title }) => /Nozzle pressure envelope/i.test(title));
  assert.ok(nozzle, 'nozzle pressure envelope is required');
  assert.ok((nozzle.xml.match(/<c:ser>/g) ?? []).length >= 3, 'nozzle envelope must compare SPP, surface limit, and bit pressure drop');
  assert.match(nozzle.xml, /<c:xVal>[\s\S]*\$A\$64:\$A\$68[\s\S]*<\/c:xVal>/,
    'nozzle diameter must be the true numeric X coordinate, not the point index');
});

test('directional MD plots follow the same response-X / depth-Y convention', async () => {
  const charts = await chartsIn('Directional_Drilling_Wellplan_and_Survey_SI.xlsx');
  const mdCharts = charts.filter(({ title }) => /\bvs MD\b/i.test(title));
  assert.equal(mdCharts.length, 4, 'directional workbook must expose four MD-indexed roadmaps');
  for (const chart of mdCharts) assertDepthRoadmap(chart, `Directional ${chart.title}`);

  const verticalSection = charts.find(({ title }) => /Vertical Section/i.test(title));
  assert.ok(verticalSection, 'vertical-section chart is required');
  assertDepthRoadmap(verticalSection, 'Directional vertical section');
});

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';

const root = fileURLToPath(new URL('..', import.meta.url));

async function loadWorkbook(name) {
  const file = await FileBlob.load(path.join(root, 'outputs', name));
  return SpreadsheetFile.importXlsx(file);
}

function chartByTitle(sheet, pattern) {
  return sheet.charts.items.find((chart) => pattern.test(chart.title?.text ?? chart.titleText ?? ''));
}

function seriesNames(chart) {
  return chart.series.items.map((series) => series.name);
}

test('torque-drag exposes a synchronized engineering dashboard with observed data and persisted settings', async () => {
  const workbook = await loadWorkbook('Torque_Drag_and_Buckling_SI.xlsx');
  const names = workbook.worksheets.items.map((sheet) => sheet.name);
  for (const required of ['Engineering Dashboard', 'Observed Data', 'Chart Settings']) {
    assert.ok(names.includes(required), `T&D is missing ${required}`);
  }

  const dashboard = workbook.worksheets.getItem('Engineering Dashboard');
  assert.equal(dashboard.getRange('A5').values[0][0], 'Selected MD');
  assert.match(String(dashboard.getRange('A8').values[0][0]), /Nearest station/i);
  assert.ok(dashboard.charts.items.length >= 4, 'T&D dashboard needs axial, torque, inclination, and friction-sensitivity roadmaps');

  const axial = chartByTitle(dashboard, /axial.*model.*actual.*limit/i);
  assert.ok(axial, 'T&D dashboard is missing the model/actual/limit axial roadmap');
  const axialNames = seriesNames(axial);
  for (const expected of ['PUW', 'SOW', 'BKR', 'SLD', 'ROT', 'DRLG', 'Observed hookload', 'Tension rating', 'Sinusoidal buckling', 'Helical buckling']) {
    assert.ok(axialNames.includes(expected), `T&D axial roadmap is missing ${expected}`);
  }

  const torque = chartByTitle(dashboard, /torque.*model.*actual.*limit/i);
  assert.ok(torque, 'T&D dashboard is missing the model/actual/limit torque roadmap');
  const torqueNames = seriesNames(torque);
  for (const expected of ['BKR', 'ROT', 'DRLG', 'Observed torque', 'Torsional rating']) {
    assert.ok(torqueNames.includes(expected), `T&D torque roadmap is missing ${expected}`);
  }
  const sensitivity = chartByTitle(dashboard, /friction sensitivity.*PUW/i);
  assert.ok(sensitivity, 'T&D dashboard is missing the friction-sensitivity roadmap');
  assert.deepEqual(seriesNames(sensitivity), ['Low friction', 'Base friction', 'High friction']);

  const observed = workbook.worksheets.getItem('Observed Data');
  assert.match(String(observed.getRange('A3').values[0][0]), /mock.*replace/i);
  const settings = workbook.worksheets.getItem('Chart Settings');
  assert.equal(settings.getRange('A5').values[0][0], 'Setting');
  assert.equal(settings.getRange('A6').values[0][0], 'Selected MD');
  assert.deepEqual(settings.getRange('B8:B10').values.flat(), [0.8, 1, 1.2]);
});

test('hydraulics exposes low/base/high flow families and a selected-depth engineering dashboard', async () => {
  const workbook = await loadWorkbook('Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx');
  const names = workbook.worksheets.items.map((sheet) => sheet.name);
  for (const required of ['Hydraulics Dashboard', 'Flow Cases', 'Chart Settings']) {
    assert.ok(names.includes(required), `Hydraulics is missing ${required}`);
  }

  const cases = workbook.worksheets.getItem('Flow Cases');
  assert.deepEqual(cases.getRange('B6:B8').values.flat(), [0.85, 1, 1.15]);
  assert.deepEqual(cases.getRange('C6:C8').values.flat(), ['Low', 'Base', 'High']);

  const dashboard = workbook.worksheets.getItem('Hydraulics Dashboard');
  assert.equal(dashboard.getRange('A5').values[0][0], 'Selected MD');
  assert.match(String(dashboard.getRange('A8').values[0][0]), /Nearest station/i);
  assert.ok(dashboard.charts.items.length >= 4, 'Hydraulics dashboard needs pressure, ECD, velocity, and nozzle charts');

  const ecd = chartByTitle(dashboard, /ECD.*flow.*window/i);
  assert.ok(ecd, 'Hydraulics dashboard is missing the ECD flow-family roadmap');
  for (const expected of ['Low flow ECD', 'Base flow ECD', 'High flow ECD', 'Static density', 'ECD limit']) {
    assert.ok(seriesNames(ecd).includes(expected), `ECD roadmap is missing ${expected}`);
  }

  const velocity = chartByTitle(dashboard, /annular velocity.*flow/i);
  assert.ok(velocity, 'Hydraulics dashboard is missing the annular-velocity flow-family roadmap');
  for (const expected of ['Low flow velocity', 'Base flow velocity', 'High flow velocity', 'Minimum transport velocity']) {
    assert.ok(seriesNames(velocity).includes(expected), `velocity roadmap is missing ${expected}`);
  }
});

test('all five workbooks persist chart configuration inside the workbook', async () => {
  const files = [
    'API_7G_Drill_String_Strength_and_Torque_SI.xlsx',
    'Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx',
    'Torque_Drag_and_Buckling_SI.xlsx',
    'BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx',
    'Directional_Drilling_Wellplan_and_Survey_SI.xlsx',
  ];
  for (const file of files) {
    const workbook = await loadWorkbook(file);
    const settings = workbook.worksheets.items.find((sheet) => sheet.name === 'Chart Settings');
    assert.ok(settings, `${file} is missing Chart Settings`);
    assert.equal(settings.getRange('A5').values[0][0], 'Setting');
    assert.match(String(settings.getRange('D3').values[0][0]), /saved with the case/i);
  }
});

test('VBA runtime recalculates dashboard helpers and the Windows builder validates them', async () => {
  const [core, torqueDrag, hydraulics, builder] = await Promise.all([
    fs.readFile(path.join(root, 'VBA/WellForgeCore.bas'), 'utf8'),
    fs.readFile(path.join(root, 'VBA/WellForgeTorqueDrag.bas'), 'utf8'),
    fs.readFile(path.join(root, 'VBA/WellForgeHydraulics.bas'), 'utf8'),
    fs.readFile(path.join(root, 'tools/Build-WellForgeVbaSuite.ps1'), 'utf8'),
  ]);
  assert.match(core, /Public Function WF_NearestDepthRow/);
  assert.match(core, /Public Sub WellForge_VisualizationSelfTest/);
  assert.match(core, /Case "Chart Settings", "Observed Data", "Flow Cases"/);
  assert.match(torqueDrag, /WF_WriteTDIndustryDashboard/);
  assert.match(torqueDrag, /Observed hookload/);
  assert.match(torqueDrag, /Torsional rating/);
  assert.match(hydraulics, /WF_WriteHydraulicsDashboard/);
  assert.match(hydraulics, /Low flow ECD/);
  assert.match(hydraulics, /High flow velocity/);
  assert.match(builder, /WellForge_VisualizationSelfTest/);
});

import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (relative) => fs.readFile(path.join(root, relative), 'utf8');

test('hydraulics workbook authority is the verified Rust adapter', async () => {
  const [core, legacy, engine, runtime, builder] = await Promise.all([
    read('VBA/WellForgeCore.bas'),
    read('VBA/WellForgeHydraulics.bas'),
    read('VBA/WellForgeHydraulicsEngine.bas'),
    read('VBA/WellForgeRustEngineRuntime.bas'),
    read('tools/Build-WellForgeVbaSuite.ps1'),
  ]);
  assert.match(core, /Case "HYDRAULICS": WF_RunHydraulicsRustEngine/);
  assert.doesNotMatch(core, /Case "HYDRAULICS": WF_CalcHydraulics\b/);
  assert.match(legacy, /Public Sub WF_CalcHydraulics\(\)\s+WF_RunHydraulicsRustEngine/);
  assert.match(legacy, /Private Sub WF_CalcHydraulicsLegacy/);
  assert.match(engine, /ThisWorkbook\.Path[\s\S]*"wellforge-hydraulics\.exe"/);
  assert.match(engine, /WF_RustIsSha256[\s\S]*WF_RustFileSha256/);
  assert.match(engine, /validate --input[\s\S]*run --input[\s\S]*verify-result --input/);
  assert.match(engine, /For nozzleIndex = 1 To 5/);
  assert.match(engine, /WF_HydCaptureSnapshots/);
  assert.match(engine, /LAST ACCEPTED VALUES PRESERVED/);
  assert.doesNotMatch(engine, /cmd\.exe/i);
  assert.match(runtime, /Public Function WF_RustExecBounded/);
  assert.match(runtime, /process\.Terminate/);
  assert.match(builder, /WellForgeHydraulicsEngine\.bas/);
});

test('torque-drag workbook authority verifies all six Rust operation states before commit', async () => {
  const [core, legacy, engine, builder] = await Promise.all([
    read('VBA/WellForgeCore.bas'),
    read('VBA/WellForgeTorqueDrag.bas'),
    read('VBA/WellForgeTorqueDragEngine.bas'),
    read('tools/Build-WellForgeVbaSuite.ps1'),
  ]);
  assert.match(core, /Case "TORQUE_DRAG": WF_RunTorqueDragRustEngine/);
  assert.doesNotMatch(core, /Case "TORQUE_DRAG": WF_CalcTorqueDrag\b/);
  assert.match(legacy, /Public Sub WF_CalcTorqueDrag\(\)\s+WF_RunTorqueDragRustEngine/);
  assert.match(legacy, /Private Sub WF_CalcTorqueDragLegacy/);
  assert.match(engine, /ThisWorkbook\.Path[\s\S]*"wellforge-torque-drag\.exe"/);
  for (const state of ['pickup', 'slack_off', 'backreaming', 'sliding', 'rotating_off_bottom', 'drilling']) {
    assert.match(engine, new RegExp(`"${state}"`));
  }
  assert.match(engine, /For i = LBound\(operationStates\) To UBound\(operationStates\)/);
  assert.match(engine, /validate --input[\s\S]*run --input[\s\S]*verify-result --input/);
  assert.match(engine, /mudDensity = WF_ToSI\(WF_Num\("Inputs", "B5"/);
  assert.match(engine, /"mud_density_kg_m3", mudDensity/);
  assert.match(engine, /WF_TDCaptureSnapshots/);
  assert.match(engine, /LAST ACCEPTED VALUES PRESERVED/);
  assert.doesNotMatch(engine, /cmd\.exe/i);
  assert.match(builder, /WellForgeTorqueDragEngine\.bas/);
});

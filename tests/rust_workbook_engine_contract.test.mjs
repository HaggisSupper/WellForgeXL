import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (relative) => fs.readFile(path.join(root, relative), 'utf8');

test('hydraulics workbook authority is the verified Rust adapter', async () => {
  const [core, worksheetModel, engine, runtime, builder, benchmark] = await Promise.all([
    read('VBA/WellForgeCore.bas'),
    read('VBA/WellForgeHydraulics.bas'),
    read('VBA/WellForgeHydraulicsEngine.bas'),
    read('VBA/WellForgeRustEngineRuntime.bas'),
    read('tools/Build-WellForgeVbaSuite.ps1'),
    read('tools/Benchmark-WellForgeHydraulics.ps1'),
  ]);
  assert.match(core, /Case "HYDRAULICS": WF_RunHydraulicsRustEngine/);
  assert.doesNotMatch(core, /Case "HYDRAULICS": WF_CalcHydraulics\b/);
  assert.match(worksheetModel, /Public Sub WF_CalcHydraulics\(\)\s+WF_RunHydraulicsRustEngine/);
  assert.match(worksheetModel, /Private Sub WF_CalcHydraulicsWorksheetModel/);
  assert.doesNotMatch(worksheetModel, /(?:Sub|Function)\s+\w*Legacy\b/);
  assert.match(engine, /ThisWorkbook\.Path[\s\S]*"wellforge-hydraulics\.exe"/);
  assert.match(engine, /WF_RustIsSha256[\s\S]*WF_RustFileSha256/);
  assert.match(engine, /validate-batch --input[\s\S]*run-batch --input[\s\S]*verify-batch --request/);
  assert.match(engine, /candidateRequests\.Add WF_BuildHydraulicsRequest\(nozzleDiameter\)/);
  assert.match(engine, /Private Function WF_HydRunBatch/);
  assert.doesNotMatch(engine, /WF_HydRunCandidate/);
  assert.match(engine, /For nozzleIndex = 1 To 5/);
  assert.match(engine, /If Len\(flowCorrelation\) = 0 Then contractVersion = "0\.1\.0" Else contractVersion = "0\.2\.0"/);
  assert.match(engine, /"flow_correlation", flowCorrelation/);
  assert.match(engine, /"compute_backend", computeBackend/);
  assert.match(engine, /"thermal_assumption", thermalAssumption/);
  assert.match(engine, /"active_flow_loop", flowType/);
  assert.match(engine, /"nozzle_discharge_coefficient", WF_Num\("Inputs", "B13"\)/);
  assert.match(engine, /"surface_backpressure_pa", WF_Num\("Inputs", "B17", 0#\)/);
  assert.match(engine, /"ecd_reference_tvd_m", WF_Num\("Inputs", "B16"\)/);
  assert.match(engine, /rheologyModel = LCase\$\(Trim\$\(WF_Str\("Fluid Model", "G6", "power_law"\)\)\)/);
  assert.match(engine, /Case "newtonian"[\s\S]*"dynamic_viscosity_pa_s", WF_ToSI\(WF_Num\("Fluid Model", "B7"\)/);
  assert.match(engine, /Case "power_law"[\s\S]*"consistency_k_pa_s_n", WF_Num\("Fluid Model", "B9"\)[\s\S]*"flow_behavior_index", WF_Num\("Fluid Model", "B8"\)/);
  assert.match(engine, /Case "bingham"[\s\S]*"yield_stress_pa", WF_ToSI\(WF_Num\("Fluid Model", "B10"\)[\s\S]*"plastic_viscosity_pa_s", WF_ToSI\(WF_Num\("Fluid Model", "B11"\)/);
  assert.match(engine, /Case "herschel_bulkley"[\s\S]*"high_shear_flow_index", WF_Num\("Fluid Model", "B14"\)/);
  assert.match(engine, /"surface_temperature_k", WF_ToSI\(WF_Num\("Fluid Model", "B12"\)/);
  assert.match(engine, /Private Function WF_HydRegimeLabel[\s\S]*section\.Exists\("flow_regime"\)/);
  assert.match(engine, /flowData\(i, 9\) = WF_HydRegimeLabel\(section, re\)/);
  assert.match(core, /"pa\*s", "pa-s", "pa\*s\^n", "pa-s\^n", "k", "1\/pa"/);
  assert.match(engine, /WF_HydCaptureSnapshots/);
  assert.match(engine, /LAST ACCEPTED VALUES PRESERVED/);
  assert.doesNotMatch(engine, /cmd\.exe/i);
  assert.match(runtime, /Public Function WF_RustExecBounded/);
  assert.match(runtime, /process\.Terminate/);
  assert.match(builder, /WellForgeHydraulicsEngine\.bas/);
  assert.match(benchmark, /validate-batch[\s\S]*run-batch[\s\S]*verify-batch/);
  assert.match(benchmark, /Single15LaunchMedianMs[\s\S]*Batch3LaunchMedianMs/);
});

test('hydraulics Rust correlation uses neutral domain names and SI-labelled fields', async () => {
  const [request, correlation] = await Promise.all([
    read('engine/crates/wellforge-hydraulics-contract/src/request.rs'),
    read('engine/crates/wellforge-hydraulics-core/src/correlation.rs'),
  ]);
  assert.match(request, /GeneralizedYieldPowerLaw/);
  assert.match(request, /ParallelCpu/);
  assert.match(request, /surface_backpressure_pa/);
  assert.match(request, /nozzle_discharge_coefficient/);
  assert.match(request, /top_tvd_m/);
  assert.match(request, /active_flow_loop/);
  assert.match(request, /ThermalAssumption/);
  assert.match(correlation, /evaluate_flow_response/);
  assert.doesNotMatch(correlation, /SecurePressureLoss|RedBook|AGIP|CalculatePipe|CalculateAnn/);
});

test('torque-drag workbook authority verifies all six Rust operation states before commit', async () => {
  const [core, worksheetModel, engine, builder] = await Promise.all([
    read('VBA/WellForgeCore.bas'),
    read('VBA/WellForgeTorqueDrag.bas'),
    read('VBA/WellForgeTorqueDragEngine.bas'),
    read('tools/Build-WellForgeVbaSuite.ps1'),
  ]);
  assert.match(core, /Case "TORQUE_DRAG": WF_RunTorqueDragRustEngine/);
  assert.doesNotMatch(core, /Case "TORQUE_DRAG": WF_CalcTorqueDrag\b/);
  assert.match(worksheetModel, /Public Sub WF_CalcTorqueDrag\(\)\s+WF_RunTorqueDragRustEngine/);
  assert.match(worksheetModel, /Private Sub WF_CalcTorqueDragWorksheetModel/);
  assert.doesNotMatch(worksheetModel, /(?:Sub|Function)\s+\w*Legacy\b/);
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

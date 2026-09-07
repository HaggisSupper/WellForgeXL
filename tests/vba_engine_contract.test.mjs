import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (relative) => fs.readFile(path.join(root, relative), 'utf8');

// Source contracts only: these do not execute VBA or prove native error semantics.
async function buildInitializeSource() {
  const core = await read('VBA/WellForgeCore.bas');
  const match = core.match(/^Public Sub WellForge_BuildInitialize\(\)\r?\n([\s\S]*?)^End Sub\s*$/m);
  assert.ok(match, 'BuildInitialize must remain a complete procedure');
  return match[1].replace(/^\s*'.*$/gm, '').trim();
}

test('BuildInitialize source disables its handler before propagating the saved error once', async () => {
  const source = await buildInitializeSource();
  const [initialization, cleanupAndHandler] = source.split(/^Cleanup:\s*$/m);
  assert.ok(cleanupAndHandler, 'initialization must have a cleanup path');
  const [cleanup, handler] = cleanupAndHandler.split(/^Failed:\s*$/m);
  assert.match(initialization, /On Error GoTo Failed/);
  assert.equal((source.match(/\bErr\.Raise\b/g) ?? []).length, 1, 'one propagation site for the saved error');
  assert.match(cleanup, /WF_Busy = False\s+On Error GoTo 0\s+If failureNumber <> 0 Then Err\.Raise failureNumber, failureSource, failureDescription\s+Exit Sub\s*$/,
    'disable error handling before the final raise so it cannot catch itself or swallow cleanup failure');
  assert.match(handler, /^\s*failureNumber = Err\.Number\s+failureSource = Err\.Source\s+failureDescription = Err\.Description\s+Resume Cleanup\s*$/,
    'save the original number/source/description before leaving the active handler');
  assert.doesNotMatch(cleanup, /On Error GoTo Failed|^\s*Resume\b|\bGoTo Cleanup\b/m);
});

test('BuildInitialize source attempts all restorations without replacing the first failure', async () => {
  const source = await buildInitializeSource();
  const cleanup = source.split(/^Cleanup:\s*$/m)[1]?.split(/^Failed:\s*$/m)[0];
  assert.ok(cleanup, 'cleanup must be identifiable');
  assert.match(cleanup, /^\s*On Error Resume Next\s+If stateCaptured Then/,
    'restoration errors must continue to the next attempt, not re-enter Failed');
  let previousEnd = 0;
  for (const [setting, saved] of [
    ['Calculation', 'oldCalc'], ['EnableEvents', 'oldEvents'], ['ScreenUpdating', 'oldScreen'],
  ]) {
    const attempt = new RegExp(`Err\\.Clear\\s+Application\\.${setting} = ${saved}\\s+`
      + 'If failureNumber = 0 And Err\\.Number <> 0 Then\\s+'
      + 'failureNumber = Err\\.Number\\s+failureSource = Err\\.Source\\s+'
      + 'failureDescription = Err\\.Description\\s+End If');
    const match = attempt.exec(cleanup);
    assert.ok(match, `${setting}: clear stale Err, attempt restoration, retain only the first failure`);
    assert.ok(match.index >= previousEnd, `${setting}: each restoration must be attempted in order`);
    previousEnd = match.index + match[0].length;
  }
  const busyClear = cleanup.indexOf('WF_Busy = False');
  assert.ok(busyClear > previousEnd, 'clear the busy latch even after restoration failures');
  assert.doesNotMatch(cleanup.slice(0, busyClear), /\b(?:Exit Sub|Err\.Raise|GoTo|Do|Loop|Resume Cleanup)\b/,
    'no early exit, throw, jump or retry may bypass later restorations or clearing WF_Busy');
});

test('BuildInitialize source handles capture failure without restoring uncaptured settings', async () => {
  const source = await buildInitializeSource();
  assert.match(source, /If WF_Busy Then Exit Sub\s+WF_Busy = True\s+On Error GoTo Failed\s+oldCalc = Application\.Calculation\s+oldEvents = Application\.EnableEvents\s+oldScreen = Application\.ScreenUpdating\s+stateCaptured = True\s+Application\.Calculation = xlCalculationManual/,
    'catch state-read failures; capture all settings before any application-setting mutation');
  const cleanup = source.split(/^Cleanup:\s*$/m)[1]?.split(/^Failed:\s*$/m)[0];
  assert.match(cleanup, /If stateCaptured Then[\s\S]*Application\.Calculation = oldCalc[\s\S]*Application\.EnableEvents = oldEvents[\s\S]*Application\.ScreenUpdating = oldScreen[\s\S]*End If\s+WF_Busy = False/,
    'restore only a complete snapshot, but always clear the owned busy latch');
});

test('BuildInitialize source retains the successful initialization sequence and workbook ownership', async () => {
  const source = await buildInitializeSource();
  const initialization = source.split(/^Cleanup:\s*$/m)[0];
  const calls = initialization.split(/\r?\n/).map((line) => line.trim())
    .filter((line) => /^(?:WF_(?:FreezeAllFormulas|ReplacePocLanguage|InstallControls|UpdateUnitMap|DispatchModel|RefreshCharts|WriteEngineStatus)\b|model = WF_ModelKind\(\))/.test(line));
  assert.deepEqual(calls, [
    'WF_FreezeAllFormulas', 'WF_ReplacePocLanguage', 'WF_InstallControls', 'WF_UpdateUnitMap',
    'model = WF_ModelKind()', 'WF_DispatchModel model', 'WF_RefreshCharts',
    'WF_WriteEngineStatus "READY", model & " compiled and initialized"',
  ]);
  assert.doesNotMatch(source, /CalculateFullRebuild|\b(?:Quit|Close|Terminate|MsgBox)\b/,
    'initialization must not add rebuilds, UI, workbook closure or process shutdown');
});

test('VBA engines expose complete calculation entry points and shared SI/unit runtime', async () => {
  const [core, api, hydraulics, hydraulicsEngine, torqueDrag, torqueDragEngine, bha, directional, json] = await Promise.all([
    read('VBA/WellForgeCore.bas'), read('VBA/WellForgeApi7G.bas'), read('VBA/WellForgeHydraulics.bas'),
    read('VBA/WellForgeHydraulicsEngine.bas'), read('VBA/WellForgeTorqueDrag.bas'), read('VBA/WellForgeTorqueDragEngine.bas'),
    read('VBA/WellForgeBha.bas'), read('VBA/WellForgeDirectional.bas'),
    read('VBA/WellForgeJsonExchange.bas'),
  ]);
  assert.match(core, /WF_ENGINE_VERSION As String = "2\.0\.0-vba"/);
  assert.match(core, /Public Sub WellForge_BuildInitialize/);
  assert.match(core, /SpecialCells\(xlCellTypeFormulas\)/);
  assert.match(core, /Public Sub WF_UpdateUnitMap/);
  assert.match(core, /If systemName = "Custom" Then/);
  assert.match(api, /Public Sub WF_CalcAPI7G/);
  assert.match(api, /Sqr\(tensionUtil \^ 2 \+ torqueUtil \^ 2\)/);
  assert.match(hydraulics, /Public Sub WF_CalcHydraulics/);
  assert.match(hydraulicsEngine, /Public Sub WF_RunHydraulicsRustEngine/);
  assert.match(hydraulicsEngine, /wellforge-hydraulics\.exe/);
  assert.match(hydraulics, /flowDiameter \^ 2 - \(flowDiameter - hydraulicDiameter\) \^ 2/);
  assert.match(torqueDrag, /Public Sub WF_CalcTorqueDrag/);
  assert.match(torqueDragEngine, /Public Sub WF_RunTorqueDragRustEngine/);
  assert.match(torqueDragEngine, /six operation states verified/);
  assert.match(torqueDrag, /sinusoidal = 2# \* Sqr/);
  assert.match(bha, /Public Sub WF_CalcBHA/);
  assert.match(bha, /frequency = 1# \/ \(2# \* BHA_PI\) \* Sqr/);
  assert.match(directional, /Public Sub WF_CalcDirectional/);
  assert.match(directional, /Private Function WF_InterpolatePath/);
  assert.match(directional, /WF_RatioFactor/);
  assert.match(json, /ImportPayloadText payloadText[\s\S]*WellForge_CalculateAll False/);
});

test('Windows builder compiles self-contained XLSM files, rejects residual formulas, and pauses by default', async () => {
  const script = await read('tools/Build-WellForgeVbaSuite.ps1');
  assert.doesNotMatch(script, /[\u0000-\u0008\u000B\u000C\u000E-\u001F]/,
    'PowerShell source contains a hidden control character');
  assert.match(script, /Join-Path \$repositoryRoot 'outputs\\vba-engine'/,
    'default XLSM output path must resolve to outputs\\vba-engine');
  assert.match(script, /Join-Path \$repositoryRoot 'VBA\\ThisWorkbookEvents\.txt'/,
    'ThisWorkbook event source must resolve inside the VBA directory');
  for (const module of ['WellForgeCore.bas', 'WellForgeJsonExchange.bas', 'WellForgeRustEngineRuntime.bas', 'WellForgeApi7G.bas', 'WellForgeHydraulics.bas', 'WellForgeHydraulicsEngine.bas', 'WellForgeTorqueDrag.bas', 'WellForgeTorqueDragEngine.bas', 'WellForgeBha.bas', 'WellForgeBhaEngine.bas', 'WellForgeDirectional.bas']) {
    assert.ok(script.includes(`'${module}'`), module);
  }
  assert.match(script, /WellForge_BuildInitialize/);
  assert.match(script, /WellForge_UnitSwitchSelfTest/,
    'Windows build must exercise SI, Imperial, and Custom display-unit changes before accepting an XLSM');
  assert.match(script, /Get-FormulaCount/);
  assert.match(script, /if \(\$formulaCount -ne 0\)/);
  const packageGuard = await read('tools/WellForgeWorkbookPackage.ps1');
  assert.match(packageGuard, /function Assert-XlsxPackageIntegrity/);
  assert.match(packageGuard, /SelectNodes\("\/\/\*\[local-name\(\)='Override'\]"\)/);
  assert.match(script, /Assert-XlsxPackageIntegrity -Path \$sourcePath/);
  assert.match(script, /\[switch\]\$NoPause/);
  assert.match(script, /Read-Host 'Press Enter to close this window'/);
  assert.match(script, /Full JSONL log/);
});

test('VBA recalculation preserves reversed-depth chart axes and refreshes visible unit labels', async () => {
  const [core, hydraulics, torqueDrag, directional] = await Promise.all([
    read('VBA/WellForgeCore.bas'), read('VBA/WellForgeHydraulics.bas'),
    read('VBA/WellForgeTorqueDrag.bas'), read('VBA/WellForgeDirectional.bas'),
  ]);
  assert.match(core, /Public Sub WF_ConfigureDepthChart/);
  assert.match(core, /\.ChartType = xlXYScatterLinesNoMarkers/);
  assert.match(core, /\.Axes\(xlValue\)\.ReversePlotOrder = True/);
  assert.match(core, /\.Axes\(xlCategory\)\.TickLabelPosition = xlHigh/);
  assert.match(core, /Public Sub WellForge_UnitSwitchSelfTest/);
  assert.match(core, /WF_AssertUnitSwitch/);
  assert.match(core, /WF_AssertModelDepthCharts/);
  assert.match(core, /ReversePlotOrder <> True/);
  assert.match(core, /TickLabelPosition <> xlHigh/);

  assert.match(hydraulics, /WF_ConfigureDepthChart wsCharts\.Name, 1, "Pressure \(" & WF_UnitLabel\("Pressure"\) & "\)", "MD \(" & WF_UnitLabel\("Length"\) & "\)"/);
  assert.match(hydraulics, /Minimum annular velocity/);
  assert.match(torqueDrag, /WF_ConfigureDepthChart/);
  assert.match(torqueDrag, /Axial load \(" & WF_UnitLabel\("Force"\) & "\)"/);
  assert.match(directional, /WF_ConfigureDepthChart/);
  assert.match(directional, /WF_ConfigureDepthChart[\s\S]*MD \(" & lengthUnit & "\)"/);
});

test('engine manifest declares the hybrid Rust/VBA authority and five formula-free XLSM outputs', async () => {
  const manifest = JSON.parse(await read('data/wellforge-vba-engine-manifest.json'));
  assert.equal(manifest.engineVersion, '2.0.0-vba');
  assert.equal(manifest.calculationAuthority, 'Hybrid');
  assert.equal(manifest.worksheetFormulasAllowed, false);
  assert.deepEqual(manifest.unitModes, ['SI', 'Imperial', 'Mixed', 'Custom']);
  assert.equal(manifest.workbooks.length, 5);
  for (const workbook of manifest.workbooks) {
    assert.ok(workbook.output.endsWith('.xlsm'));
    assert.ok(['VBA', 'Rust'].includes(workbook.calculationAuthority));
  }
  const bha = manifest.workbooks.find(({ kind }) => kind === 'bha');
  assert.equal(bha.calculationAuthority, 'Rust');
  assert.equal(bha.entryPoint, 'WF_RunBhaRustEngine');
  assert.equal(bha.executable, 'wellforge-bha.exe');
  assert.equal(bha.hashManifest, 'wellforge-bha.exe.sha256');
  const hydraulics = manifest.workbooks.find(({ kind }) => kind === 'hydraulics');
  assert.equal(hydraulics.calculationAuthority, 'Rust');
  assert.equal(hydraulics.entryPoint, 'WF_RunHydraulicsRustEngine');
  assert.equal(hydraulics.executable, 'wellforge-hydraulics.exe');
  const torqueDrag = manifest.workbooks.find(({ kind }) => kind === 'torqueDrag');
  assert.equal(torqueDrag.calculationAuthority, 'Rust');
  assert.equal(torqueDrag.entryPoint, 'WF_RunTorqueDragRustEngine');
  assert.equal(torqueDrag.executable, 'wellforge-torque-drag.exe');
  assert.deepEqual(
    manifest.standaloneRustEngines.map(({ executable }) => executable),
    ['wellforge-bha.exe', 'wellforge-trajectory.exe', 'wellforge-torque-drag.exe', 'wellforge-hydraulics.exe'],
  );
  assert.deepEqual(
    manifest.standaloneRustEngines.map(({ package: enginePackage }) => enginePackage),
    ['wellforge-bha-cli', 'wellforge-trajectory-cli', 'wellforge-torque-drag-cli', 'wellforge-hydraulics-cli'],
  );
});

test('VBA source passes deterministic structural lint', () => {
  const result = spawnSync(process.execPath, ['tools/lint_vba.mjs'], { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /structural lint passed/);
});

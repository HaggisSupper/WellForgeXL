import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

const read = (relative) => {
  const url = new URL(`../${relative}`, import.meta.url);
  return fs.existsSync(url) ? fs.readFileSync(url, 'utf8') : '';
};

const engine = read('VBA/WellForgeTrajectoryEngine.bas');
const core = read('VBA/WellForgeCore.bas');
const directional = read('VBA/WellForgeDirectional.bas');
const suiteBuilder = read('tools/Build-WellForgeVbaSuite.ps1');
const engineBuilder = read('tools/Build-WellForgeTrajectoryEngine.ps1');
const engineTester = read('tools/Test-WellForgeTrajectoryEngine.ps1');

function procedure(source, name) {
  const match = source.match(new RegExp(`(?:Public|Private) (?:Sub|Function) ${name}\\b[\\s\\S]*?End (?:Sub|Function)`, 'i'));
  assert.ok(match, `${name} procedure is missing`);
  return match[0];
}

test('adapter source routes directional production to the fixed colocated Rust executable', () => {
  assert.match(core, /Case "DIRECTIONAL": WF_RunTrajectoryRustEngine/);
  assert.doesNotMatch(core, /Case "DIRECTIONAL": WF_CalcDirectional/);
  assert.match(engine, /Public Sub WF_RunTrajectoryRustEngine/);
  assert.match(engine, /ThisWorkbook\.Path[\s\S]*"wellforge-trajectory\.exe"/);
  assert.match(engine, /"wellforge-trajectory\.exe\.sha256"/);
  assert.match(engine, /WF_TrajectoryFileSha256\(executablePath\)/);
  assert.match(engine, /ENGINE HASH MISMATCH/);
  assert.match(engine, /RUST REQUIRED — NO VBA FALLBACK/);
  assert.doesNotMatch(engine, /WF_CalcDirectional/);
});

test('adapter source declares a fresh bounded run and the implemented CLI sequence', () => {
  assert.match(engine, /WF_CreateFreshTrajectoryRunDirectory/);
  assert.match(engine, /Environ\$\("TEMP"\)[\s\S]*WellForgeTrajectory/);
  assert.match(engine, /WF_ExecTrajectoryBounded[\s\S]*timeoutSeconds/);
  assert.match(engine, /\.Terminate/);
  assert.match(engine, /Application\.Interactive = False/);
  assert.match(engine, /Application\.Interactive = previousInteractive/);
  assert.doesNotMatch(engine, /cmd\.exe/i);
  const validate = engine.indexOf(' validate --input ');
  const run = engine.indexOf(' run --input ');
  const verify = engine.indexOf(' verify-result --input ');
  const bridge = engine.indexOf(' bridge --input ');
  assert.ok(validate >= 0 && validate < run && run < verify && verify < bridge,
    'expected validate, run, verify-result, bridge order');
  assert.match(engine, / run --input [\s\S]*--output [\s\S]*--diagnostics /);
  assert.match(engine, /WF_TrajectoryRequestHashFromDiagnostics/);
  assert.match(engine, /diagnostics\.jsonl/);
  assert.match(engine, /failureState = "INVALID RESULT"\s+requestHash = WF_TrajectoryRequestHashFromDiagnostics/);
  assert.match(engine, /request\.json[\s\S]*result\.json[\s\S]*result\.wfbridge/);
});

test('request-export source reads explicit workbook provenance and stored row UUIDs', () => {
  const request = procedure(engine, 'WF_BuildTrajectoryRequest');
  for (const field of ['contract_version', 'analysis_id', 'sources', 'md_datum', 'azimuth_reference', 'vertical_section_azimuth_rad', 'plan', 'survey', 'targets', 'slides', 'formations', 'projection']) {
    assert.match(request, new RegExp(`"${field}"`), field);
  }
  for (const field of ['uuid', 'uri', 'object_type', 'content_hash', 'citation_name', 'source_system']) {
    assert.match(engine, new RegExp(`"${field}"`), field);
  }
  assert.match(engine, /Inputs"\)\.Range\("Q6:V9"\)/);
  assert.match(engine, /Inputs"\)\.Range\("Q12:Q17"\)/);
  assert.match(engine, /Calc"\)\.Range\("JA7:JE506"\)/);
  assert.match(engine, /WF_TrajectoryValidateUnitAndControlInputs/);
  assert.match(engine, /WF_TrajectoryRequireUnit "E6", Array\("deg", "rad"\)/);
  assert.match(engine, /WF_TrajectoryInputNumber\("B16", "vertical section azimuth"\), WF_Str\("Inputs", "E6", "rad"\)/);
  assert.match(engine, /WF_TrajectoryOptionalInputNumber[\s\S]*optional input contains an Excel error/);
  assert.match(engine, /WF_TrajectoryOptionalNullableNumber[\s\S]*contains an Excel error/);
  assert.doesNotMatch(request, /Scriptlet\.TypeLib|Randomize|Rnd\(|Environ\$\(|ThisWorkbook\.Path/);
});

test('trajectory numeric helpers reject VBA Booleans before numeric coercion', () => {
  const required = procedure(engine, 'WF_TrajectoryRequiredNumber');
  const requiredBoolean = required.indexOf('VarType(value) = vbBoolean');
  assert.ok(requiredBoolean >= 0, 'required numeric helper must reject vbBoolean');
  assert.ok(requiredBoolean < required.indexOf('IsNumeric(value)'),
    'required Boolean rejection must precede IsNumeric');
  assert.ok(requiredBoolean < required.indexOf('CDbl(value)'),
    'required Boolean rejection must precede CDbl');

  for (const name of ['WF_TrajectoryOptionalInputNumber', 'WF_TrajectoryOptionalNullableNumber']) {
    const optional = procedure(engine, name);
    const booleanGuard = optional.indexOf('VarType(value) = vbBoolean');
    assert.ok(booleanGuard >= 0, `${name} must reject vbBoolean explicitly`);
    assert.ok(booleanGuard < optional.indexOf('WF_TrajectoryRequiredNumber'),
      `${name} Boolean rejection must precede numeric coercion`);
  }
});

test('adapter source stages and validates the complete strict bridge before result mutations', () => {
  const parse = procedure(engine, 'WF_ParseAndValidateTrajectoryBridge');
  const commit = procedure(engine, 'WF_CommitTrajectoryBridge');
  assert.doesNotMatch(parse, /ClearContents|\.Value2\s*=/);
  assert.match(commit, /ClearContents/);
  assert.match(commit, /\.Value2\s*=/);
  assert.match(engine, /WF_ParseAndValidateTrajectoryBridge[\s\S]*WF_CommitTrajectoryBridge/);
  assert.doesNotMatch(engine, /JsonParse\(/);
  for (const token of ['WF_TRAJECTORY_MAX_PLAN = 500', 'WF_TRAJECTORY_MAX_SURVEY = 500', 'WF_TRAJECTORY_MAX_TARGETS = 100', 'WF_TRAJECTORY_MAX_SLIDES = 200', 'WF_TRAJECTORY_MAX_FORMATIONS = 100']) {
    assert.ok(engine.includes(token), token);
  }
  for (const guard of ['INVALID BRIDGE VERSION', 'INVALID BRIDGE REQUEST HASH', 'INVALID BRIDGE RESULT HASH', 'DUPLICATE BRIDGE ID', 'MISSING BRIDGE RECORD', 'UNKNOWN BRIDGE RECORD', 'INVALID BRIDGE ENUM', 'NON-FINITE BRIDGE NUMBER', 'BRIDGE CAPACITY EXCEEDED']) {
    assert.ok(engine.includes(guard), guard);
  }
  assert.match(engine, /FAILED — LAST ACCEPTED VALUES PRESERVED/);
  assert.match(engine, /Private Function WF_TrajectoryRestoreSnapshots[\s\S]*As Boolean/);
  assert.match(engine, /ROLLBACK INCOMPLETE/);
  assert.match(engine, /NO ACTUAL PICKS/);
  assert.match(engine, /SHORT SLIDE/);
  assert.match(engine, /yieldOutlierLimit/);
  for (const state of ['ENGINE UNAVAILABLE', 'ENGINE HASH MISMATCH', 'INVALID REQUEST', 'ANALYSIS FAILED', 'INVALID RESULT']) assert.ok(engine.includes(state), state);
  assert.match(directional, /Public Sub WF_RefreshDirectionalPresentation/);
});

test('trajectory rollback is equality-verified and exercised through an injected commit failure', () => {
  const entry = procedure(engine, 'WF_RunTrajectoryRustEngine');
  const selfTest = procedure(engine, 'WellForge_TrajectoryRollbackSelfTest');
  const capture = procedure(engine, 'WF_TrajectoryCaptureSnapshots');
  assert.match(engine, /WF_TRAJECTORY_INJECT_COMMIT_FAILURE/);
  assert.match(engine, /WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED/);
  assert.match(engine, /WF_TrajectorySnapshotsMatch/);
  assert.match(engine, /Public Sub WellForge_TrajectoryRollbackSelfTest/);
  assert.match(engine, /Rollback:[\s\S]*WF_TrajectoryRestoreSnapshots[\s\S]*WF_TrajectorySnapshotsMatch/);
  assert.match(capture, /WF_TrajectorySnapshot snapshots, "Results", "P7:P8"/);
  assert.match(capture, /WF_TrajectorySnapshot snapshots, "Results", "P10:P14"/);
  assert.doesNotMatch(capture, /"P5:P14"|"P5"|"P6"|"P9"/);
  assert.match(selfTest, /Set snapshots = WF_TrajectoryCaptureSnapshots\(\)[\s\S]*WF_RunTrajectoryRustEngine[\s\S]*If Not WF_TrajectorySnapshotsMatch\(snapshots\)/);
  assert.match(selfTest, /Range\("P5"\)[\s\S]*WF_TRAJECTORY_EXECUTION_MODE/);
  assert.match(selfTest, /Range\("P6"\)[\s\S]*FAILED — LAST ACCEPTED VALUES PRESERVED/);
  assert.match(selfTest, /previousDiagnostic = CStr\(ThisWorkbook\.Worksheets\("Results"\)\.Range\("P9"\)\.Value2\)[\s\S]*WF_RunTrajectoryRustEngine/);
  assert.match(selfTest, /telemetryDiagnostic[\s\S]*StrComp\(telemetryDiagnostic, previousDiagnostic, vbBinaryCompare\)[\s\S]*Dir\$/);
  assert.match(entry, /WF_PublishTrajectoryFailure failureState, diagnosticPath, lastAcceptedValuesPreserved/);
});

test('adapter keeps presentation-only rotations out of canonical result blocks', () => {
  const commit = procedure(engine, 'WF_CommitTrajectoryBridge');
  const station = procedure(engine, 'WF_TrajectoryFillStationCalc');
  const contract = procedure(engine, 'WF_TrajectoryFillContractRow');
  assert.match(station, /PRESENTATION_ONLY_NOT_RUST_RESULT/);
  assert.doesNotMatch(station, /verticalSection|crossline/i);
  assert.match(contract, /PRESENTATION_ONLY_NOT_RUST_RESULT/);
  assert.doesNotMatch(contract, /verticalSection|crossline/i);
  assert.match(commit, /compareValues\(rowIndex, 25\) = "PRESENTATION_ONLY_NOT_RUST_RESULT"/);
  assert.match(commit, /compareValues\(rowIndex, 26\) = "PRESENTATION_ONLY_NOT_RUST_RESULT"/);
  assert.match(commit, /compareValues\(rowIndex, 30\) = CDbl\(residuals\(rowIndex, 7\)\)/);
  assert.match(commit, /compareValues\(rowIndex, 32\) = CDbl\(residuals\(rowIndex, 8\)\)/);
});

test('adapter maps the canonical target helper only from exact bridge fields or typed unavailable states', () => {
  const commit = procedure(engine, 'WF_CommitTrajectoryBridge');
  assert.match(commit, /targetHelper\(rowIndex, 1\) = targets\(rowIndex, 1\)/);
  for (const column of [4, 5, 6, 7, 11, 12]) {
    assert.match(commit, new RegExp(`targetHelper\\(rowIndex, ${column}\\) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"`));
  }
  assert.match(commit, /targetHelper\(rowIndex, 13\) = targets\(rowIndex, 12\)/);
  assert.match(commit, /targetHelper\(rowIndex, 18\) = targets\(rowIndex, 4\)/);
  assert.doesNotMatch(commit, /targetHelper\(rowIndex, (?:9|10)\)[^\r\n]*(?:surfaceNorth|surfaceEast)/);
});

test('public trajectory entry preserves caller runtime guards on success and failure', () => {
  const entry = procedure(engine, 'WF_RunTrajectoryRustEngine');
  assert.match(entry, /previousBusy = WF_Busy/);
  assert.match(entry, /previousEvents = Application\.EnableEvents/);
  assert.match(entry, /previousInteractive = Application\.Interactive/);
  assert.match(entry, /WF_Busy = True/);
  assert.match(entry, /Application\.EnableEvents = False/);
  assert.match(entry, /Application\.Interactive = False/);
  assert.ok((entry.match(/WF_Busy = previousBusy/g) ?? []).length >= 2);
  assert.ok((entry.match(/Application\.EnableEvents = previousEvents/g) ?? []).length >= 2);
  assert.ok((entry.match(/Application\.Interactive = previousInteractive/g) ?? []).length >= 2);
});

test('release scripts declare pinned workspace gates, hash, smoke-test wiring, and build before Excel', () => {
  assert.match(engineBuilder, /rustup run 1\.98\.0 rustc --version --verbose/);
  assert.match(engineBuilder, /cargo \+1\.98\.0 fmt --all -- --check/);
  assert.match(engineBuilder, /cargo \+1\.98\.0 clippy --workspace --all-targets --locked --offline -- -D warnings/);
  assert.match(engineBuilder, /cargo \+1\.98\.0 test --workspace --locked --offline/);
  assert.doesNotMatch(engineBuilder, /cargo \+1\.98\.0 (?:clippy|test) -p wellforge-trajectory-cli/);
  assert.match(engineBuilder, /cargo \+1\.98\.0 build --release --locked --offline -p wellforge-trajectory-cli/);
  assert.match(engineBuilder, /wellforge-trajectory\.exe/);
  assert.match(engineBuilder, /Get-FileHash[\s\S]*SHA256/);
  assert.match(engineTester, /trajectory-release-one-minimal\.json/);
  assert.match(engineTester, /validate[\s\S]*run[\s\S]*verify-result[\s\S]*bridge/);
  const trajectoryBuild = suiteBuilder.indexOf('Build-WellForgeTrajectoryEngine.ps1');
  const excelStart = suiteBuilder.indexOf('New-Object -ComObject Excel.Application');
  assert.ok(trajectoryBuild >= 0 && trajectoryBuild < excelStart);
  assert.match(suiteBuilder, /WellForgeTrajectoryEngine\.bas/);
  assert.match(core, /\.Range\("J3"\)\.Value2 = "Calculation client \/ engine"/);
  assert.match(core, /compiled calculation client\/engine workbook/);
  assert.doesNotMatch(core, /\.Range\("J3"\)\.Value2 = "VBA calculation engine"/);
  assert.doesNotMatch(core, /Replacement:="VBA calculation engine workbook"/);
});

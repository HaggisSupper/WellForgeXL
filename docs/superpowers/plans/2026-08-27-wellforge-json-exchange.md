# WellForge JSON Exchange Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one unit-preserving JSON exchange contract to all five WellForge workbooks, with equivalent offline VBA and Office Script import/export implementations.

**Architecture:** A versioned JSON Schema and unit registry define the wire contract. Each workbook exposes declarative `Exchange Map` and hidden `Exchange State` sheets; both automation hosts interpret those sheets, convert through canonical SI, protect formulas, and merge only workbook-owned payload branches.

**Tech Stack:** JavaScript ES modules on the Codex primary Node runtime, `@oai/artifact-tool`, JSON Schema Draft 2020-12, Excel VBA7 with late binding, Office Scripts TypeScript, Windows PowerShell 5.1 Excel COM automation.

**Spec:** `docs/superpowers/specs/2026-08-27-wellforge-json-exchange-design.md`

## Global Constraints

- JSON is the only executable exchange format; YAML is not read or written by the macros.
- Every physical quantity is `{ "value": number, "unit": string }`; unitless values use unit `"1"`.
- Workbook calculations and stored calculation inputs remain canonical SI.
- Import writes mapped inputs only and never overwrites formulas or results.
- Export may include results but preserves unknown payload branches.
- VBA must be self-contained, late-bound and usable offline on 64-bit desktop Excel.
- Office Script must consume the same mappings, state records and unit registry.
- `.xlsx` originals remain intact; the Windows installer creates `.xlsm` copies.
- No external links, network calls, Docker, formula injection, silent unit substitution or partial import writes.
- Display precision remains two decimals; conversion factors retain audit precision.

---

### Task 1: Canonical unit registry and JSON Schema

**Files:**
- Create: `schema/wellforge-analysis-exchange.schema.json`
- Create: `data/wellforge-unit-registry.json`
- Create: `src/exchange/schema_contract.mjs`
- Create: `src/exchange/schema_validator.mjs`
- Test: `tests/exchange_schema.test.mjs`

**Interfaces:**
- Produces: `SCHEMA_VERSION: "1.0.0"`, `UNIT_REGISTRY`, `quantity(value, unit, metadata?)`, and `validateExchangePayload(payload): { valid: boolean, errors: string[] }`.
- Consumes: no task-local interfaces.

- [ ] **Step 1: Write failing schema and unit-registry tests**

```js
test('registry declares reversible units with dimensions', () => {
  for (const [symbol, unit] of Object.entries(UNIT_REGISTRY)) {
    assert.equal(typeof unit.dimension, 'string', symbol);
    assert.equal(Number.isFinite(unit.toSiMultiplier), true, symbol);
    assert.equal(Number.isFinite(unit.toSiOffset), true, symbol);
  }
});

test('schema rejects a quantity without a unit', () => {
  const payload = minimalExchangePayload();
  payload.operatingPoint.wob = { value: 120000 };
  const result = validateExchangePayload(payload);
  assert.equal(result.valid, false);
  assert.match(result.errors.join('\n'), /operatingPoint\.wob\.unit/);
});
```

- [ ] **Step 2: Run the tests and verify the contract is absent**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_schema.test.mjs`

Expected: FAIL because `src/exchange/schema_contract.mjs` and the schema artifacts do not exist.

- [ ] **Step 3: Implement focused contract modules and registry**

Define supported symbols for `1`, length, diameter, area, volume, flow rate, density, force, pressure, torque, stress, angle, speed, angular gradient, viscosity, frequency, RPM, date and temperature. Use this exact conversion interface:

```js
export function toSi(quantity, expectedDimension) {
  const definition = UNIT_REGISTRY[quantity.unit];
  if (!definition || definition.dimension !== expectedDimension) {
    throw new Error(`Unit ${quantity.unit} is not valid for ${expectedDimension}`);
  }
  return quantity.value * definition.toSiMultiplier + definition.toSiOffset;
}

export function fromSi(siValue, unit, expectedDimension) {
  const definition = UNIT_REGISTRY[unit];
  if (!definition || definition.dimension !== expectedDimension) {
    throw new Error(`Unit ${unit} is not valid for ${expectedDimension}`);
  }
  return (siValue - definition.toSiOffset) / definition.toSiMultiplier;
}
```

Implement a dependency-free validator for the exact exchange contract: required top-level keys, object/array/scalar types, semantic-version pattern, finite quantity values, registered unit symbols, required stable identifiers and duplicate identifiers. The JSON Schema remains the authoritative external contract.

- [ ] **Step 4: Run schema tests and verify green**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_schema.test.mjs`

Expected: PASS with malformed quantities, unknown units and duplicate IDs rejected.

- [ ] **Step 5: Commit the schema contract**

```bash
git add schema data/wellforge-unit-registry.json src/exchange/schema_contract.mjs src/exchange/schema_validator.mjs tests/exchange_schema.test.mjs
git commit -m "feat: define WellForge JSON exchange contract"
```

### Task 2: Complete mock payload generated from the shared case

**Files:**
- Create: `src/exchange/build_mock_payload.mjs`
- Create: `data/wellforge-mock-case.json`
- Modify: `src/shared_mock_case.mjs`
- Test: `tests/exchange_mock_payload.test.mjs`

**Interfaces:**
- Consumes: `SCHEMA_VERSION`, `quantity`, `validateExchangePayload`, `MOCK_CASE`, and `directionalReferenceData`.
- Produces: `buildMockExchangePayload(): WellForgeExchangePayload` and deterministic `data/wellforge-mock-case.json`.

- [ ] **Step 1: Write failing completeness and consistency tests**

```js
test('mock payload covers every analysis branch and validates', () => {
  const payload = buildMockExchangePayload();
  assert.deepEqual(Object.keys(payload.analyses).sort(),
    ['api7g', 'bha', 'directional', 'hydraulics', 'torqueDrag']);
  assert.equal(validateExchangePayload(payload).valid, true);
});

test('trajectory and repeated operating values come from the shared case', () => {
  const payload = buildMockExchangePayload();
  assert.equal(payload.trajectory.survey.length, 60);
  assert.deepEqual(payload.fluids[0].density, quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'));
  assert.deepEqual(payload.operatingPoint.wob, quantity(MOCK_CASE.operation.wobN, 'N'));
});
```

- [ ] **Step 2: Run the tests and confirm they fail for missing payload generation**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_mock_payload.test.mjs`

Expected: FAIL because `buildMockExchangePayload` is undefined.

- [ ] **Step 3: Build every payload branch from existing shared sources**

Use stable identifiers (`survey-000`, `dp-01`, `bha-motor-rss`) and represent each physical value with `quantity`. Analysis results that require Excel formula evaluation must use this explicit state:

```js
{
  calculationState: 'notCalculated',
  method: 'WellForge workbook formula model',
  results: []
}
```

Do not invent cached outputs. Include the reference plan, actual survey, targets, slide intervals, formation tops, tubular sections, BHA components, fluid, rig limits, nozzle candidates and operating cases.

- [ ] **Step 4: Export deterministic mock JSON**

Add to `build_mock_payload.mjs`:

```js
if (import.meta.url === `file://${process.argv[1]}`) {
  const payload = buildMockExchangePayload();
  await fs.writeFile('data/wellforge-mock-case.json', `${JSON.stringify(payload, null, 2)}\n`);
}
```

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" src/exchange/build_mock_payload.mjs`

Expected: `data/wellforge-mock-case.json` is created and stable on a second run.

- [ ] **Step 5: Run mock-payload tests and commit**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_mock_payload.test.mjs`

```bash
git add src/shared_mock_case.mjs src/exchange/build_mock_payload.mjs data/wellforge-mock-case.json tests/exchange_mock_payload.test.mjs
git commit -m "feat: add complete WellForge mock exchange payload"
```

### Task 3: Declarative workbook exchange mappings

**Files:**
- Create: `src/exchange/workbook_maps.mjs`
- Create: `src/exchange/add_exchange_sheets.mjs`
- Modify: `src/workbook.mjs`
- Modify: `src/directional_contract.mjs`
- Modify: `src/build_api7g.mjs`
- Modify: `src/build_hydraulics.mjs`
- Modify: `src/build_torque_drag.mjs`
- Modify: `src/build_bha.mjs`
- Modify: `src/build_directional.mjs`
- Test: `tests/exchange_mapping.test.mjs`

**Interfaces:**
- Consumes: workbook builders and unit registry dimensions.
- Produces: `WORKBOOK_MAPS`, `addExchangeSheets(workbook, workbookKind)`, visible `Exchange Map`, hidden `Exchange State`, and `Exchange Buffer` sheets.

- [ ] **Step 1: Write failing mapping topology tests**

```js
test('every workbook exposes complete exchange infrastructure', () => {
  for (const [kind, build] of Object.entries(BUILDERS)) {
    const workbook = build();
    assert.ok(workbook.worksheets.getItem('Exchange Map'), kind);
    assert.ok(workbook.worksheets.getItem('Exchange State'), kind);
    assert.ok(workbook.worksheets.getItem('Exchange Buffer'), kind);
  }
});

test('formula destinations are export-only', () => {
  for (const mapping of Object.values(WORKBOOK_MAPS).flat()) {
    if (mapping.direction === 'Output') assert.equal(mapping.writable, false);
  }
});
```

- [ ] **Step 2: Run tests and verify missing sheets/maps failure**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_mapping.test.mjs`

Expected: FAIL because the three exchange sheets do not exist.

- [ ] **Step 3: Define exact mapping records**

Use this frozen shape:

```js
{
  pointer: '/operatingPoint/wob',
  direction: 'Input',
  sheet: 'Inputs',
  address: 'B7',
  shape: 'Scalar',
  unitSource: 'N',
  dimension: 'force',
  dataType: 'number',
  required: true,
  writable: true,
}
```

Define mappings for every user input and every decision/result surfaced by each workbook. Map trajectory tables by stable ID and include row capacity explicitly.

- [ ] **Step 4: Add and format exchange sheets**

`Exchange Map` is visible and protected from accidental edits except its documentation cells. `Exchange State` is hidden and has columns `Pointer`, `Original value`, `Original unit`, `Canonical value`, `Destination`, `Imported at`. `Exchange Buffer` contains `Action`, `Payload`, `Status`, and `Diagnostics` cells, with payload wrapped and column width capped.

Append these sheet names to default and directional topology arrays without changing the established order of existing sheets.

- [ ] **Step 5: Run mapping and existing topology tests**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_mapping.test.mjs tests/directional_structure.test.mjs tests/suite_acceptance.test.mjs`

Expected: PASS after expected sheet arrays are updated to include the three exchange sheets.

- [ ] **Step 6: Commit workbook mappings**

```bash
git add src/exchange src/workbook.mjs src/directional_contract.mjs src/build_*.mjs tests/exchange_mapping.test.mjs tests/directional_structure.test.mjs tests/suite_acceptance.test.mjs
git commit -m "feat: add declarative JSON exchange mappings"
```

### Task 4: Dependency-free exchange engine and round-trip oracle

**Files:**
- Create: `src/exchange/json_pointer.mjs`
- Create: `src/exchange/exchange_engine.mjs`
- Create: `tests/helpers/mock_workbook_adapter.mjs`
- Create: `tests/exchange_roundtrip.test.mjs`

**Interfaces:**
- Consumes: `WORKBOOK_MAPS`, `UNIT_REGISTRY`, `toSi`, `fromSi`, and `validateExchangePayload`.
- Produces: `importMappedPayload(adapter, payload, map, state)`, `exportMappedPayload(adapter, existingPayload, map, state, options)`, `getPointer`, `setPointer`, and `mergeByStableId`.

- [ ] **Step 1: Write failing transaction and unit-preservation tests**

```js
test('unchanged imported quantities retain their original units', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 0 });
  const payload = minimalExchangePayload({ wob: { value: 28, unit: 'klbf' } });
  const imported = importMappedPayload(adapter, payload, wobMap, []);
  assert.equal(adapter.read('Inputs', 'B7'), 124550.4428);
  const exported = exportMappedPayload(adapter, payload, wobMap, imported.state, {});
  assert.deepEqual(exported.payload.operatingPoint.wob, { value: 28, unit: 'klbf' });
});

test('failed imports leave all destination values unchanged', () => {
  const adapter = mockAdapter({ 'Inputs!B7': 120000 });
  const result = importMappedPayload(adapter, payloadWithWrongPressureUnitAtWob(), wobMap, []);
  assert.equal(result.ok, false);
  assert.equal(adapter.read('Inputs', 'B7'), 120000);
});
```

- [ ] **Step 2: Run round-trip tests and verify missing engine failure**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_roundtrip.test.mjs`

Expected: FAIL because `importMappedPayload` and `exportMappedPayload` do not exist.

- [ ] **Step 3: Implement parse/validate/plan/apply transaction stages**

Use an adapter interface:

```js
// read(sheet, address) -> scalar | matrix
// capture(sheet, address) -> immutable prior value
// write(sheet, address, value) -> void
// restore(changeSet) -> void
// isFormula(sheet, address) -> boolean
```

Build the entire change set before calling `write`. Reject formula destinations, dimension mismatches, missing required inputs, duplicate table IDs and non-finite values. Prefix string inputs beginning with formula-control characters as literal text.

- [ ] **Step 4: Implement merge and unit-state semantics**

Compare current canonical values with stored canonical values using relative tolerance `1e-12`. Preserve original `{value, unit}` only when unchanged. When changed, use the Unit Map preference passed in `options.displayUnits[dimension]`; fall back to canonical SI.

Merge objects recursively and arrays by stable `id`, retaining unknown fields and records.

- [ ] **Step 5: Run all exchange-engine tests and commit**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/exchange_roundtrip.test.mjs tests/exchange_schema.test.mjs tests/exchange_mock_payload.test.mjs`

```bash
git add src/exchange/json_pointer.mjs src/exchange/exchange_engine.mjs tests/helpers/mock_workbook_adapter.mjs tests/exchange_roundtrip.test.mjs
git commit -m "feat: implement unit-safe exchange engine"
```

### Task 5: Office Script import, export and validation

**Files:**
- Create: `OfficeScripts/WellForgeJsonExchange.ts`
- Create: `tests/office_script_exchange.test.mjs`
- Modify: `README.md`

**Interfaces:**
- Consumes: worksheet tables `Exchange Map`, `Exchange State`, `Exchange Buffer` and the JSON contract.
- Produces: `main(workbook, action, jsonText, includeResults): ExchangeScriptResult`.

- [ ] **Step 1: Write failing Office Script source-contract tests**

```js
test('Office Script exposes all three actions and never evaluates JSON as code', async () => {
  const source = await fs.readFile('OfficeScripts/WellForgeJsonExchange.ts', 'utf8');
  assert.match(source, /action:\s*"Import"\s*\|\s*"Export"\s*\|\s*"Validate"/);
  assert.match(source, /JSON\.parse/);
  assert.doesNotMatch(source, /\beval\s*\(|new Function/);
  assert.match(source, /Exchange State/);
});
```

- [ ] **Step 2: Run the test and verify the script is absent**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/office_script_exchange.test.mjs`

Expected: FAIL because the Office Script file does not exist.

- [ ] **Step 3: Implement Office Script types and pure helpers**

Use no external imports. Define `Quantity`, `MappingRow`, `StateRow`, `Diagnostic`, and:

```ts
function main(
  workbook: ExcelScript.Workbook,
  action: "Import" | "Export" | "Validate",
  jsonText: string = "",
  includeResults: boolean = true,
): ExchangeScriptResult
```

Port JSON pointer, unit conversion, validation, merge and transaction behavior from the JavaScript oracle. Read mappings from the sheet at runtime.

- [ ] **Step 4: Implement workbook operations**

Import parses `jsonText` or `Exchange Buffer!B5`, validates every mapping, captures prior values, writes only approved inputs, updates `Exchange State`, recalculates fully and writes diagnostics. Export merges into supplied JSON or a new payload, optionally includes outputs, writes pretty JSON to `Exchange Buffer!B5`, and returns it.

- [ ] **Step 5: Run Office Script contract and suite tests**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/office_script_exchange.test.mjs tests/suite_acceptance.test.mjs`

Expected: PASS; source contains no dynamic execution or network APIs.

- [ ] **Step 6: Document Automate-tab usage and commit**

Document exact `Import`, `Export` and `Validate` calls, the file-dialog limitation, and copying the returned JSON or buffer cell to a file.

```bash
git add OfficeScripts/WellForgeJsonExchange.ts tests/office_script_exchange.test.mjs README.md
git commit -m "feat: add Office Script JSON exchange"
```

### Task 6: Self-contained VBA JSON codec and exchange macro

**Files:**
- Create: `VBA/WellForgeJsonExchange.bas`
- Create: `tests/vba_exchange_contract.test.mjs`
- Modify: `README.md`

**Interfaces:**
- Consumes: `Exchange Map`, `Exchange State`, `Exchange Buffer`, unit registry embedded as a private VBA constant table, and schema version `1.0.0`.
- Produces: public macros `WellForge_LoadJson`, `WellForge_SaveJson`, `WellForge_ValidateExchange`.

- [ ] **Step 1: Write failing VBA public-interface and safety tests**

```js
test('VBA module is self-contained and exposes the required offline macros', async () => {
  const source = await fs.readFile('VBA/WellForgeJsonExchange.bas', 'utf8');
  assert.match(source, /Public Sub WellForge_LoadJson\(\)/);
  assert.match(source, /Public Sub WellForge_SaveJson\(\)/);
  assert.match(source, /Public Sub WellForge_ValidateExchange\(\)/);
  assert.match(source, /CreateObject\("Scripting\.Dictionary"\)/i);
  assert.doesNotMatch(source, /References\.AddFrom|ScriptControl|WinHttp|XMLHTTP/i);
});
```

- [ ] **Step 2: Run the test and verify the module is absent**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/vba_exchange_contract.test.mjs`

Expected: FAIL because `VBA/WellForgeJsonExchange.bas` does not exist.

- [ ] **Step 3: Implement the VBA JSON codec**

Implement recursive-descent functions with these signatures:

```vb
Private Function JsonParse(ByVal Text As String) As Variant
Private Function ParseValue(ByRef Cursor As Long, ByVal Text As String) As Variant
Private Function ParseObject(ByRef Cursor As Long, ByVal Text As String) As Object
Private Function ParseArray(ByRef Cursor As Long, ByVal Text As String) As Collection
Private Function ParseString(ByRef Cursor As Long, ByVal Text As String) As String
Private Function JsonStringify(ByVal Value As Variant, Optional ByVal Indent As Long = 0) As String
```

Support UTF-8 files through late-bound `ADODB.Stream`, Unicode escapes including surrogate pairs, invariant decimal serialization, Booleans, null, arrays and dictionaries. Reject trailing tokens, non-finite numbers and duplicate object keys.

- [ ] **Step 4: Implement mapping, conversion and transaction logic**

Use private routines `ReadExchangeMap`, `ValidatePayload`, `BuildChangeSet`, `ApplyChangeSet`, `RestoreChangeSet`, `ReadExchangeState`, `WriteExchangeState`, `MergePayload`, `ToSi`, and `FromSi`. Test `Range.HasFormula` before every import write. No JSON value may supply a sheet name or address.

- [ ] **Step 5: Implement public macros and atomic file handling**

`WellForge_LoadJson` uses `Application.FileDialog(msoFileDialogFilePicker)`. `WellForge_SaveJson` uses `msoFileDialogSaveAs`, writes a temporary sibling file, creates a timestamped `.bak` when replacing, then renames the temporary file. Both restore `Application.Calculation`, `EnableEvents` and `ScreenUpdating` in a single cleanup block.

- [ ] **Step 6: Run VBA contract tests and commit**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/vba_exchange_contract.test.mjs tests/exchange_roundtrip.test.mjs`

```bash
git add VBA/WellForgeJsonExchange.bas tests/vba_exchange_contract.test.mjs README.md
git commit -m "feat: add offline VBA JSON exchange macro"
```

### Task 7: Windows installer and macro-enabled workbook creation

**Files:**
- Create: `tools/Install-WellForgeJsonMacro.ps1`
- Create: `tools/Test-WellForgeJsonMacro.ps1`
- Create: `tests/vba_installer_contract.test.mjs`

**Interfaces:**
- Consumes: `outputs/*.xlsx` and `VBA/WellForgeJsonExchange.bas`.
- Produces: `outputs/macro-enabled/*.xlsm` on Windows with Excel installed.

- [ ] **Step 1: Write failing installer safety tests**

```js
test('installer creates copies and never overwrites source xlsx files', async () => {
  const source = await fs.readFile('tools/Install-WellForgeJsonMacro.ps1', 'utf8');
  assert.match(source, /macro-enabled/);
  assert.match(source, /xlOpenXMLWorkbookMacroEnabled/);
  assert.doesNotMatch(source, /Remove-Item\s+.*\.xlsx|SaveAs\([^\n]*\.xlsx/i);
});
```

- [ ] **Step 2: Run the test and confirm the installer is absent**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/vba_installer_contract.test.mjs`

Expected: FAIL because the installer does not exist.

- [ ] **Step 3: Implement idempotent Excel COM installer**

The script resolves its own repository root, validates the five expected `.xlsx` files, creates `outputs/macro-enabled`, opens each source read-only, saves a copy using file format `52`, imports the `.bas` module through `VBProject.VBComponents.Import`, saves, closes and releases every COM object in `finally` blocks.

Before processing, probe `VBProject`; if Trust Center access is disabled, stop without creating partial outputs and print: `Enable Trust access to the VBA project object model, then rerun this installer.`

- [ ] **Step 4: Implement Windows smoke test script**

`Test-WellForgeJsonMacro.ps1` opens each `.xlsm`, verifies `VBProject.VBComponents.Item("WellForgeJsonExchange")`, runs `WellForge_ValidateExchange`, checks `Checks` for no blocking diagnostic, then closes without saving.

- [ ] **Step 5: Run static installer tests and commit**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/vba_installer_contract.test.mjs`

```bash
git add tools/Install-WellForgeJsonMacro.ps1 tools/Test-WellForgeJsonMacro.ps1 tests/vba_installer_contract.test.mjs
git commit -m "feat: add xlsm macro installer and smoke test"
```

### Task 8: Build, visual QA, end-to-end verification and package

**Files:**
- Modify: `src/build_suite.mjs`
- Modify: `.work/verify_suite.mjs`
- Modify: `tests/render_contract.test.mjs`
- Modify: `tests/suite_acceptance.test.mjs`
- Modify: `README.md`
- Create: `docs/JSON_EXCHANGE_GUIDE.md`
- Create: `WellForge_Analysis_Workbook_Suite_SI.zip`

**Interfaces:**
- Consumes: all prior task outputs.
- Produces: rebuilt `.xlsx` workbooks, verified package, documented optional `.xlsm` generation and final downloadable archive.

- [ ] **Step 1: Write failing end-to-end acceptance assertions**

```js
test('suite package contains schema, mock data and both automation hosts', async () => {
  for (const path of [
    'schema/wellforge-analysis-exchange.schema.json',
    'data/wellforge-mock-case.json',
    'VBA/WellForgeJsonExchange.bas',
    'OfficeScripts/WellForgeJsonExchange.ts',
    'tools/Install-WellForgeJsonMacro.ps1',
  ]) assert.equal(await exists(path), true, path);
});
```

- [ ] **Step 2: Run the acceptance test and verify package integration is incomplete**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/suite_acceptance.test.mjs`

Expected: FAIL until every exchange artifact is included by the suite packager.

- [ ] **Step 3: Rebuild all workbooks**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" src/build_suite.mjs`

Expected: five `.xlsx` files with `Exchange Map`, hidden `Exchange State`, and `Exchange Buffer` sheets.

- [ ] **Step 4: Run complete automated verification**

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" --test tests/*.test.mjs`

Expected: all tests PASS, including schema, mock payload, mappings, round trips, Office Script, VBA source and installer contracts.

Run: `"$CODEX_PRIMARY_RUNTIME_NODE" .work/verify_suite.mjs`

Expected: zero visible-sheet formula errors and all existing charts present.

- [ ] **Step 5: Render and inspect exchange surfaces**

Render `Exchange Map`, `Exchange Buffer`, `Inputs`, `Results` and `Graphs` for each workbook. Fix clipped headers, exposed state sheets, formula errors, broken chart ranges or unreadable diagnostics. Rebuild and rerun the compact checks after every visual repair.

- [ ] **Step 6: Write operator documentation**

Document schema structure, unit-preservation rules, VBA installation, VBA load/save commands, Office Script parameters, buffer workflow, merge behavior, recovery, version compatibility and the exact Trust Center prerequisite.

- [ ] **Step 7: Assemble and verify the archive**

Create a fresh staging directory. Copy `README.md`, `schema`, `data`, `VBA`, `OfficeScripts`, `tools`, `docs`, `src`, `tests`, `.work` and five output workbooks. Zip as `WellForge_Analysis_Workbook_Suite_SI.zip` and run `unzip -t`.

- [ ] **Step 8: Commit the completed exchange release**

```bash
git add README.md docs/JSON_EXCHANGE_GUIDE.md src/build_suite.mjs .work/verify_suite.mjs tests outputs schema data VBA OfficeScripts tools
git commit -m "feat: deliver WellForge unit-safe JSON exchange"
```


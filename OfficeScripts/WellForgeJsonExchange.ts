/**
 * WellForge JSON exchange for the Excel Automate tab.
 *
 * The workbook remains the authority for engineering calculations. This script
 * only transfers mapped values, preserving imported units in Exchange State.
 * It deliberately uses neither imports nor dynamic execution APIs.
 */

type ExchangeAction = "Import" | "Export" | "Validate";
type JsonValue = string | number | boolean | null | JsonObject | JsonValue[];
interface JsonObject { [key: string]: JsonValue; }
interface Quantity extends JsonObject { value: number; unit: string; quality?: string; source?: string; timestamp?: string; note?: string; }
interface MappingRow {
  pointer: string; direction: string; sheet: string; address: string; shape: string;
  valueColumn: string; idColumn: string; capacity: number; unitSource: string;
  dimension: string; dataType: string; required: boolean; writable: boolean;
}
interface StateRow { pointer: string; originalValue: number; originalUnit: string; canonicalValue: number; destination: string; importedAt: string; }
interface Diagnostic { severity: "error" | "warning" | "info"; message: string; pointer?: string; }
interface ExchangeScriptResult { success: boolean; action: ExchangeAction; diagnostics: Diagnostic[]; jsonText?: string; }
interface UnitDefinition { dimension: string; dimensions?: string[]; multiplier: number; offset: number; }
interface PendingWrite { range: ExcelScript.Range; value: string | number | boolean; destination: string; state?: StateRow; }
interface TransactionEntry { range: ExcelScript.Range; values: ExcelScript.RangeValue[][]; }

const SCHEMA_VERSION = "1.0.0";
const MAP_HEADERS = ["JSON Pointer", "Direction", "Sheet", "Address", "Shape", "Value column", "Stable ID column", "Row capacity", "Unit source", "Dimension", "Data type", "Required", "Writable"];
const UNIT_REGISTRY: { [unit: string]: UnitDefinition } = {
  "1": { dimension: "unitless", multiplier: 1, offset: 0 }, "m": { dimension: "length", dimensions: ["diameter"], multiplier: 1, offset: 0 },
  "ft": { dimension: "length", dimensions: ["diameter"], multiplier: 0.3048, offset: 0 }, "km": { dimension: "length", multiplier: 1000, offset: 0 },
  "mm": { dimension: "diameter", dimensions: ["length"], multiplier: 0.001, offset: 0 }, "in": { dimension: "diameter", dimensions: ["length"], multiplier: 0.0254, offset: 0 },
  "m2": { dimension: "area", multiplier: 1, offset: 0 }, "ft2": { dimension: "area", multiplier: 0.09290304, offset: 0 }, "in2": { dimension: "area", multiplier: 0.00064516, offset: 0 }, "cm2": { dimension: "area", multiplier: 0.0001, offset: 0 },
  "m3": { dimension: "volume", multiplier: 1, offset: 0 }, "bbl": { dimension: "volume", multiplier: 0.158987294928, offset: 0 }, "gal": { dimension: "volume", multiplier: 0.003785411784, offset: 0 }, "L": { dimension: "volume", multiplier: 0.001, offset: 0 },
  "m3/s": { dimension: "flowRate", multiplier: 1, offset: 0 }, "L/s": { dimension: "flowRate", multiplier: 0.001, offset: 0 }, "L/min": { dimension: "flowRate", multiplier: 1 / 60000, offset: 0 }, "gpm": { dimension: "flowRate", multiplier: 0.0000630901964, offset: 0 },
  "kg/m3": { dimension: "density", multiplier: 1, offset: 0 }, "ppg": { dimension: "density", multiplier: 119.826427316, offset: 0 }, "lb/ft3": { dimension: "density", multiplier: 16.01846337396, offset: 0 },
  "N": { dimension: "force", multiplier: 1, offset: 0 }, "lbf": { dimension: "force", multiplier: 4.4482301, offset: 0 }, "klbf": { dimension: "force", multiplier: 4448.2301, offset: 0 }, "kN": { dimension: "force", multiplier: 1000, offset: 0 },
  "Pa": { dimension: "pressure", dimensions: ["stress"], multiplier: 1, offset: 0 }, "kPa": { dimension: "pressure", multiplier: 1000, offset: 0 }, "psi": { dimension: "pressure", dimensions: ["stress"], multiplier: 6894.757293168, offset: 0 }, "bar": { dimension: "pressure", multiplier: 100000, offset: 0 },
  "N*m": { dimension: "torque", multiplier: 1, offset: 0 }, "N-m": { dimension: "torque", multiplier: 1, offset: 0 }, "kN*m": { dimension: "torque", multiplier: 1000, offset: 0 }, "kN-m": { dimension: "torque", multiplier: 1000, offset: 0 }, "ft-lbf": { dimension: "torque", multiplier: 1.3558179483314, offset: 0 },
  "MPa": { dimension: "stress", multiplier: 1000000, offset: 0 }, "ksi": { dimension: "stress", multiplier: 6894757.293168, offset: 0 }, "rad": { dimension: "angle", multiplier: 1, offset: 0 }, "deg": { dimension: "angle", multiplier: Math.PI / 180, offset: 0 },
  "m/s": { dimension: "speed", multiplier: 1, offset: 0 }, "ft/min": { dimension: "speed", multiplier: 0.00508, offset: 0 }, "m/min": { dimension: "speed", multiplier: 1 / 60, offset: 0 },
  "rad/m": { dimension: "angularGradient", multiplier: 1, offset: 0 }, "deg/100ft": { dimension: "angularGradient", multiplier: 0.0005729577951308232, offset: 0 }, "deg/30m": { dimension: "angularGradient", multiplier: 0.0005817764173314432, offset: 0 },
  "Pa*s": { dimension: "viscosity", multiplier: 1, offset: 0 }, "cP": { dimension: "viscosity", multiplier: 0.001, offset: 0 }, "Hz": { dimension: "frequency", multiplier: 1, offset: 0 },
  "rad/s": { dimension: "rotationalSpeed", multiplier: 1, offset: 0 }, "rpm": { dimension: "rotationalSpeed", multiplier: 0.10471975511965977, offset: 0 }, "d": { dimension: "date", multiplier: 86400, offset: 0 },
  "K": { dimension: "temperature", multiplier: 1, offset: 0 }, "C": { dimension: "temperature", multiplier: 1, offset: 273.15 }, "F": { dimension: "temperature", multiplier: 5 / 9, offset: 255.3722222222222 },
};

function main(workbook: ExcelScript.Workbook, action: "Import" | "Export" | "Validate", jsonText: string = "", includeResults: boolean = true): ExchangeScriptResult {
  const diagnostics: Diagnostic[] = [];
  const buffer = workbook.getWorksheet("Exchange Buffer");
  try {
    const mappings = readMappings(workbook, diagnostics);
    if (diagnostics.some((diagnostic) => diagnostic.severity === "error")) return finish(buffer, action, diagnostics, false);
    if (action === "Validate") {
      if (jsonText.trim() !== "" || String(buffer.getRange("B5").getValue()).trim() !== "") validatePayload(parsePayload(jsonText, buffer, diagnostics), diagnostics);
      validateMappings(workbook, mappings, diagnostics);
      return finish(buffer, action, diagnostics, !hasErrors(diagnostics));
    }
    if (action === "Import") return importPayload(workbook, buffer, mappings, jsonText, diagnostics);
    return exportPayload(workbook, buffer, mappings, jsonText, includeResults, diagnostics);
  } catch (error) {
    diagnostics.push({ severity: "error", message: `Exchange failed: ${errorMessage(error)}` });
    return finish(buffer, action, diagnostics, false);
  }
}

function importPayload(workbook: ExcelScript.Workbook, buffer: ExcelScript.Worksheet, mappings: MappingRow[], jsonText: string, diagnostics: Diagnostic[]): ExchangeScriptResult {
  const payload = parsePayload(jsonText, buffer, diagnostics);
  validatePayload(payload, diagnostics);
  validateMappings(workbook, mappings, diagnostics);
  const pending = buildImportWrites(workbook, mappings, payload, diagnostics);
  if (hasErrors(diagnostics)) return finish(buffer, "Import", diagnostics, false);
  const stateRows = pending.filter((change) => change.state !== undefined).map((change) => change.state as StateRow);
  const stateSheet = workbook.getWorksheet("Exchange State");
  const existingStateRows = Math.max(0, stateSheet.getUsedRange().getRowCount() - 5);
  const stateBackup = stateSheet.getRange(`A6:F${5 + Math.max(existingStateRows, stateRows.length, 1)}`);
  const transaction = captureTransaction(pending, [stateBackup]);
  try {
    for (const change of pending) change.range.setValue(change.value);
    writeState(workbook, stateRows);
    workbook.getApplication().calculate(ExcelScript.CalculationType.full);
    diagnostics.push({ severity: "info", message: `Imported ${pending.length} mapped values.` });
    return finish(buffer, "Import", diagnostics, true);
  } catch (error) {
    rollback(transaction, diagnostics);
    diagnostics.push({ severity: "error", message: `Import write failed and rollback was attempted: ${errorMessage(error)}` });
    return finish(buffer, "Import", diagnostics, false);
  }
}

function exportPayload(workbook: ExcelScript.Workbook, buffer: ExcelScript.Worksheet, mappings: MappingRow[], jsonText: string, includeResults: boolean, diagnostics: Diagnostic[]): ExchangeScriptResult {
  validateMappings(workbook, mappings, diagnostics);
  let payload: JsonObject;
  if (jsonText.trim() !== "" || String(buffer.getRange("B5").getValue()).trim() !== "") { payload = parsePayload(jsonText, buffer, diagnostics); validatePayload(payload, diagnostics); }
  else payload = newPayload();
  if (hasErrors(diagnostics)) return finish(buffer, "Export", diagnostics, false);
  const state = readState(workbook);
  for (const mapping of mappings) {
    if (mapping.direction === "Input" || (mapping.direction !== "Input" && includeResults)) exportMapping(workbook, payload, mapping, state, diagnostics);
  }
  if (hasErrors(diagnostics)) return finish(buffer, "Export", diagnostics, false);
  const exported = JSON.stringify(payload, null, 2);
  buffer.getRange("B5").setValue(exported);
  diagnostics.push({ severity: "info", message: "Exported mapped JSON to Exchange Buffer!B5." });
  return finish(buffer, "Export", diagnostics, true, exported);
}

function readMappings(workbook: ExcelScript.Workbook, diagnostics: Diagnostic[]): MappingRow[] {
  const sheet = workbook.getWorksheet("Exchange Map");
  const used = sheet.getUsedRange();
  const values = used.getValues();
  const header = values.findIndex((row) => MAP_HEADERS.every((name, index) => String(row[index]) === name));
  if (header < 0) { diagnostics.push({ severity: "error", message: "Exchange Map has no recognized header row." }); return []; }
  const rows: MappingRow[] = [];
  for (const row of values.slice(header + 1)) {
    if (String(row[0]).trim() === "") continue;
    rows.push({ pointer: String(row[0]), direction: String(row[1]), sheet: String(row[2]), address: String(row[3]), shape: String(row[4]), valueColumn: String(row[5] ?? ""), idColumn: String(row[6] ?? ""), capacity: Number(row[7] ?? 0), unitSource: String(row[8]), dimension: String(row[9]), dataType: String(row[10]), required: Boolean(row[11]), writable: Boolean(row[12]) });
  }
  return rows;
}

function validateMappings(workbook: ExcelScript.Workbook, mappings: MappingRow[], diagnostics: Diagnostic[]): void {
  const seen = new Set<string>();
  for (const mapping of mappings) {
    const key = `${mapping.pointer}|${mapping.sheet}|${mapping.address}`;
    if (seen.has(key)) diagnostics.push({ severity: "error", message: "Exchange Map has duplicate mapping rows.", pointer: mapping.pointer });
    seen.add(key);
    if (!mapping.pointer.startsWith("/") || !["Input", "Output", "Both"].includes(mapping.direction) || !["Scalar", "Table"].includes(mapping.shape)) diagnostics.push({ severity: "error", message: "Exchange Map contains an invalid mapping row.", pointer: mapping.pointer });
    try {
      const range = workbook.getWorksheet(mapping.sheet).getRange(mapping.address);
      if (mapping.writable && range.getFormulas().flat().some((formula) => formula !== "")) diagnostics.push({ severity: "error", message: "Mapped formula destination cannot be imported.", pointer: mapping.pointer });
      if (mapping.shape === "Table" && (mapping.idColumn === "" || mapping.capacity < 1)) diagnostics.push({ severity: "error", message: "Table mapping needs a stable identifier column and capacity.", pointer: mapping.pointer });
    } catch (_) { diagnostics.push({ severity: "error", message: "Mapping references an unavailable sheet or address.", pointer: mapping.pointer }); }
  }
}

function buildImportWrites(workbook: ExcelScript.Workbook, mappings: MappingRow[], payload: JsonObject, diagnostics: Diagnostic[]): PendingWrite[] {
  const pending: PendingWrite[] = [];
  const importedAt = new Date().toISOString();
  for (const mapping of mappings) {
    if (!mapping.writable || (mapping.direction !== "Input" && mapping.direction !== "Both")) continue;
    if (mapping.shape === "Scalar") {
      const value = getPointer(payload, mapping.pointer);
      if (value === undefined) { if (mapping.required) diagnostics.push({ severity: "error", message: "Required mapped value is missing.", pointer: mapping.pointer }); continue; }
      const range = workbook.getWorksheet(mapping.sheet).getRange(mapping.address);
      const converted = importValue(workbook, mapping, value, diagnostics);
      if (converted !== undefined) pending.push({ range, value: converted.value, destination: `${mapping.sheet}!${mapping.address}`, state: converted.state ? { pointer: mapping.pointer, originalValue: converted.state.value, originalUnit: converted.state.unit, canonicalValue: toSi(converted.state, mapping.dimension), destination: `${mapping.sheet}!${mapping.address}`, importedAt } : undefined });
      continue;
    }
    const parts = mapping.pointer.split("/*/");
    const records = getPointer(payload, parts[0]);
    if (!Array.isArray(records)) { if (mapping.required) diagnostics.push({ severity: "error", message: "Required stable-ID table is missing.", pointer: mapping.pointer }); continue; }
    const ids = stableIds(records, mapping.pointer, diagnostics);
    const range = workbook.getWorksheet(mapping.sheet).getRange(mapping.address);
    const rowById = worksheetRowsById(workbook, mapping, diagnostics);
    for (const record of records) {
      if (!isObject(record)) continue;
      const id = String(record.id ?? "");
      const row = rowById[id];
      if (row === undefined) { diagnostics.push({ severity: "error", message: `No worksheet row exists for stable identifier ${id}.`, pointer: mapping.pointer }); continue; }
      const field = parts[1]; const value = record[field];
      if (value === undefined) { if (mapping.required) diagnostics.push({ severity: "error", message: `Required value is missing for stable identifier ${id}.`, pointer: mapping.pointer }); continue; }
      const cell = range.getCell(row, 0);
      const converted = importValue(workbook, mapping, value, diagnostics);
      if (converted !== undefined) pending.push({ range: cell, value: converted.value, destination: `${mapping.sheet}!${mapping.address}:${id}`, state: converted.state ? { pointer: `${parts[0]}/${escapeToken(id)}/${parts[1]}`, originalValue: converted.state.value, originalUnit: converted.state.unit, canonicalValue: toSi(converted.state, mapping.dimension), destination: `${mapping.sheet}!${mapping.address}:${id}`, importedAt } : undefined });
    }
    if (ids.size === 0 && mapping.required) diagnostics.push({ severity: "error", message: "Table mapping has no stable identifiers.", pointer: mapping.pointer });
  }
  return pending;
}

function importValue(workbook: ExcelScript.Workbook, mapping: MappingRow, value: JsonValue, diagnostics: Diagnostic[]): { value: string | number | boolean; state?: Quantity } | undefined {
  if (mapping.dataType === "number" || mapping.dataType === "integer") {
    if (!isQuantity(value) || !Number.isFinite(value.value)) { diagnostics.push({ severity: "error", message: "Physical values must be finite { value, unit } quantities.", pointer: mapping.pointer }); return undefined; }
    if (mapping.dataType === "integer" && !Number.isInteger(value.value)) { diagnostics.push({ severity: "error", message: "Integer mapping received a non-integer value.", pointer: mapping.pointer }); return undefined; }
    const targetUnit = resolveUnit(workbook, mapping, diagnostics);
    if (targetUnit === undefined || !validUnit(value.unit, mapping.dimension) || !validUnit(targetUnit, mapping.dimension)) { diagnostics.push({ severity: "error", message: "Quantity unit is incompatible with its mapping dimension.", pointer: mapping.pointer }); return undefined; }
    return { value: fromSi(toSi(value, mapping.dimension), targetUnit, mapping.dimension), state: value };
  }
  if (mapping.dataType === "Boolean") { if (typeof value !== "boolean") { diagnostics.push({ severity: "error", message: "Boolean mapping received a non-Boolean value.", pointer: mapping.pointer }); return undefined; } return { value }; }
  if (typeof value !== "string") { diagnostics.push({ severity: "error", message: "Text mapping received a non-string value.", pointer: mapping.pointer }); return undefined; }
  return { value: literalText(value) };
}

function exportMapping(workbook: ExcelScript.Workbook, payload: JsonObject, mapping: MappingRow, state: { [pointer: string]: StateRow }, diagnostics: Diagnostic[]): void {
  const sheet = workbook.getWorksheet(mapping.sheet);
  if (mapping.shape === "Scalar") { setPointer(payload, mapping.pointer, exportValue(workbook, mapping, sheet.getRange(mapping.address).getValue(), state[mapping.pointer], diagnostics)); return; }
  const parts = mapping.pointer.split("/*/");
  const records = ensureArray(payload, parts[0]);
  const range = sheet.getRange(mapping.address); const ids = worksheetRowsById(workbook, mapping, diagnostics);
  for (const [id, row] of Object.entries(ids)) {
    const record = findOrCreateById(records, id); const pointer = `${parts[0]}/${escapeToken(id)}/${parts[1]}`;
    record[parts[1]] = exportValue(workbook, mapping, range.getCell(row, 0).getValue(), state[pointer], diagnostics);
  }
}

function exportValue(workbook: ExcelScript.Workbook, mapping: MappingRow, value: ExcelScript.RangeValue, state: StateRow | undefined, diagnostics: Diagnostic[]): JsonValue {
  if (mapping.dataType !== "number" && mapping.dataType !== "integer") {
    if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") return value;
    diagnostics.push({ severity: "error", message: "Workbook mapping contains an Excel error value.", pointer: mapping.pointer });
    return null;
  }
  if (typeof value !== "number" || !Number.isFinite(value)) { diagnostics.push({ severity: "error", message: "Workbook numeric mapping contains a non-finite value.", pointer: mapping.pointer }); return null; }
  const unit = resolveUnit(workbook, mapping, diagnostics);
  if (unit === undefined || !validUnit(unit, mapping.dimension)) return null;
  const canonical = toSi({ value, unit }, mapping.dimension);
  if (state && approximatelyEqual(canonical, state.canonicalValue)) return { value: state.originalValue, unit: state.originalUnit };
  return { value, unit };
}

function parsePayload(jsonText: string, buffer: ExcelScript.Worksheet, diagnostics: Diagnostic[]): JsonObject {
  const source = jsonText.trim() === "" ? String(buffer.getRange("B5").getValue()) : jsonText;
  try { const payload = JSON.parse(source); if (!isObject(payload)) throw new Error("JSON root must be an object"); return payload; }
  catch (error) { diagnostics.push({ severity: "error", message: `JSON parsing failed: ${errorMessage(error)}` }); return {}; }
}

function validatePayload(payload: JsonObject, diagnostics: Diagnostic[]): void {
  if (typeof payload.schemaVersion !== "string" || payload.schemaVersion.split(".")[0] !== SCHEMA_VERSION.split(".")[0]) diagnostics.push({ severity: "error", message: "Payload has an unsupported schema major version." });
  if (typeof payload.caseId !== "string" || payload.caseId === "") diagnostics.push({ severity: "error", message: "Payload caseId is required." });
  for (const path of ["trajectory.plan", "trajectory.survey", "trajectory.targets", "trajectory.slideIntervals", "trajectory.formationTops", "holeSections", "tubulars", "bhaComponents", "fluids", "pumpNozzle.pumps", "pumpNozzle.nozzles"]) {
    const records = getDotPath(payload, path); if (Array.isArray(records)) stableIds(records, path, diagnostics);
  }
}

function stableIds(records: JsonValue[], pointer: string, diagnostics: Diagnostic[]): Set<string> {
  const ids = new Set<string>();
  for (const record of records) { const id = isObject(record) && typeof record.id === "string" ? record.id : ""; if (id === "" || ids.has(id)) diagnostics.push({ severity: "error", message: "Table records require a unique stable identifier.", pointer }); else ids.add(id); }
  return ids;
}

function worksheetRowsById(workbook: ExcelScript.Workbook, mapping: MappingRow, diagnostics: Diagnostic[]): { [id: string]: number } {
  const idRange = workbook.getWorksheet(mapping.sheet).getRange(mapping.address.replace(/^[A-Z]+/, mapping.idColumn).replace(/:[A-Z]+/, ":" + mapping.idColumn));
  const result: { [id: string]: number } = {};
  idRange.getValues().forEach((row, index) => { const id = String(row[0] ?? ""); if (id !== "") { if (result[id] !== undefined) diagnostics.push({ severity: "error", message: "Worksheet table contains duplicate stable identifier.", pointer: mapping.pointer }); result[id] = index; } });
  return result;
}

function readState(workbook: ExcelScript.Workbook): { [pointer: string]: StateRow } {
  const state: { [pointer: string]: StateRow } = {}; const values = workbook.getWorksheet("Exchange State").getUsedRange().getValues();
  for (const row of values.slice(5)) if (String(row[0] ?? "") !== "") state[String(row[0])] = { pointer: String(row[0]), originalValue: Number(row[1]), originalUnit: String(row[2]), canonicalValue: Number(row[3]), destination: String(row[4]), importedAt: String(row[5]) };
  return state;
}

function writeState(workbook: ExcelScript.Workbook, state: StateRow[]): void {
  const sheet = workbook.getWorksheet("Exchange State"); const used = sheet.getUsedRange(); if (used.getRowCount() > 5) sheet.getRangeByIndexes(5, 0, used.getRowCount() - 5, 6).clear(ExcelScript.ClearApplyTo.contents);
  if (state.length > 0) sheet.getRangeByIndexes(5, 0, state.length, 6).setValues(state.map((entry) => [entry.pointer, entry.originalValue, entry.originalUnit, entry.canonicalValue, entry.destination, entry.importedAt]));
}

function captureTransaction(pending: PendingWrite[], extraRanges: ExcelScript.Range[] = []): TransactionEntry[] { return [...pending.map((change) => change.range), ...extraRanges].map((range) => ({ range, values: range.getValues() })); }
function rollback(entries: TransactionEntry[], diagnostics: Diagnostic[]): void { try { for (const entry of entries) entry.range.setValues(entry.values); diagnostics.push({ severity: "warning", message: "Workbook values were restored from the transaction log (rollback)." }); } catch (error) { diagnostics.push({ severity: "error", message: `Rollback failed: ${errorMessage(error)}` }); } }
function finish(buffer: ExcelScript.Worksheet, action: ExchangeAction, diagnostics: Diagnostic[], success: boolean, jsonText?: string): ExchangeScriptResult { writeDiagnostics(buffer, action, diagnostics, success); return { success, action, diagnostics, jsonText }; }
function writeDiagnostics(buffer: ExcelScript.Worksheet, action: ExchangeAction, diagnostics: Diagnostic[], success: boolean): void { buffer.getRange("B6").setValue(action); buffer.getRange("B7").setValue(success ? "Success" : "Failed"); buffer.getRange("B8").setValue(diagnostics.map((item) => `${item.severity.toUpperCase()}: ${item.message}`).join("\n")); }

function resolveUnit(workbook: ExcelScript.Workbook, mapping: MappingRow, diagnostics: Diagnostic[]): string | undefined { if (mapping.unitSource === "text") return undefined; if (!mapping.unitSource.includes("!")) return mapping.unitSource; const split = mapping.unitSource.lastIndexOf("!"); const sheet = mapping.unitSource.slice(0, split).replace(/^'|'$/g, ""); const address = mapping.unitSource.slice(split + 1); const unit = String(workbook.getWorksheet(sheet).getRange(address).getValue()); if (!UNIT_REGISTRY[unit]) diagnostics.push({ severity: "error", message: `Unit source does not contain a supported unit: ${unit}.`, pointer: mapping.pointer }); return unit; }
function validUnit(unit: string, dimension: string): boolean { const definition = UNIT_REGISTRY[unit]; return definition !== undefined && (definition.dimension === dimension || (definition.dimensions ?? []).includes(dimension)); }
function toSi(quantity: Quantity, dimension: string): number { if (!validUnit(quantity.unit, dimension)) throw new Error(`Unit ${quantity.unit} is not valid for ${dimension}`); const definition = UNIT_REGISTRY[quantity.unit]; return quantity.value * definition.multiplier + definition.offset; }
function fromSi(value: number, unit: string, dimension: string): number { if (!validUnit(unit, dimension)) throw new Error(`Unit ${unit} is not valid for ${dimension}`); const definition = UNIT_REGISTRY[unit]; return (value - definition.offset) / definition.multiplier; }
function newPayload(): JsonObject { return { schemaVersion: SCHEMA_VERSION, caseId: "", createdAt: new Date().toISOString(), producer: { name: "WellForge Office Script", version: "1.0.0" }, metadata: {}, unitPreferences: {}, trajectory: { plan: [], survey: [], targets: [], slideIntervals: [], formationTops: [] }, holeSections: [], tubulars: [], bhaComponents: [], fluids: [], operatingPoint: {}, rigLimits: {}, pumpNozzle: { pumps: [], nozzles: [] }, analyses: {}, provenance: { notes: [] }, warnings: [] }; }
function getPointer(root: JsonObject, pointer: string): JsonValue | undefined { let value: JsonValue | undefined = root; for (const token of pointer.split("/").slice(1)) { if (!isObject(value) && !Array.isArray(value)) return undefined; value = (value as JsonObject)[unescapeToken(token)]; } return value; }
function setPointer(root: JsonObject, pointer: string, value: JsonValue): void { const tokens = pointer.split("/").slice(1).map(unescapeToken); let cursor = root; for (const token of tokens.slice(0, -1)) { if (!isObject(cursor[token])) cursor[token] = {}; cursor = cursor[token] as JsonObject; } cursor[tokens[tokens.length - 1]] = value; }
function ensureArray(root: JsonObject, pointer: string): JsonObject[] { const current = getPointer(root, pointer); if (Array.isArray(current)) return current.filter(isObject); setPointer(root, pointer, []); return getPointer(root, pointer) as JsonObject[]; }
function findOrCreateById(records: JsonObject[], id: string): JsonObject { const found = records.find((record) => record.id === id); if (found) return found; const created: JsonObject = { id }; records.push(created); return created; }
function getDotPath(root: JsonObject, path: string): JsonValue | undefined { return path.split(".").reduce<JsonValue | undefined>((value, key) => isObject(value) ? value[key] : undefined, root); }
function isObject(value: JsonValue | undefined): value is JsonObject { return value !== null && typeof value === "object" && !Array.isArray(value); }
function isQuantity(value: JsonValue): value is Quantity { return isObject(value) && typeof value.value === "number" && typeof value.unit === "string"; }
function literalText(value: string): string { return /^[=+\-@]/.test(value) ? `'${value}` : value; }
function approximatelyEqual(left: number, right: number): boolean { return Math.abs(left - right) <= 1e-12 * Math.max(1, Math.abs(left), Math.abs(right)); }
function escapeToken(value: string): string { return value.replace(/~/g, "~0").replace(/\//g, "~1"); }
function unescapeToken(value: string): string { return value.replace(/~1/g, "/").replace(/~0/g, "~"); }
function hasErrors(diagnostics: Diagnostic[]): boolean { return diagnostics.some((item) => item.severity === "error"); }
function errorMessage(error: unknown): string { return error instanceof Error ? error.message : String(error); }

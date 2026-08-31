import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const MODULE_PATH = new URL('../VBA/WellForgeJsonExchange.bas', import.meta.url);

async function moduleSource() {
  return readFile(MODULE_PATH, 'utf8');
}

function procedure(source, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const match = source.match(new RegExp(
    `(?:Public|Private)\\s+(?:Sub|Function)\\s+${escaped}\\b[\\s\\S]*?End\\s+(?:Sub|Function)`,
    'i',
  ));
  assert.ok(match, `missing VBA procedure ${name}`);
  return match[0];
}

function normalized(source) {
  return source.replace(/'.*$/gm, '').replace(/\s+/g, ' ');
}

test('module is offline, late-bound, VBA7-safe, and exposes only the required entry points', async () => {
  const source = await moduleSource();
  assert.match(source, /^Attribute VB_Name = "WellForgeJsonExchange"/m);
  assert.match(source, /^Option Explicit$/m);
  for (const macro of ['WellForge_LoadJson', 'WellForge_SaveJson', 'WellForge_ValidateExchange']) {
    assert.match(source, new RegExp(`^Public Sub ${macro}\\(\\)$`, 'm'));
  }
  assert.match(source, /CreateObject\("Scripting\.Dictionary"\)/i);
  assert.match(source, /CreateObject\("ADODB\.Stream"\)/i);
  assert.doesNotMatch(source, /References\.AddFrom|ScriptControl|WinHttp|XMLHTTP|VBProject|Declare\s+(?:PtrSafe\s+)?Function/i);
});

test('module stays within importable VBA7 source limits', async () => {
  const source = await moduleSource();
  const lines = source.split(/\r?\n/);
  assert.ok(lines.every((line) => line.length <= 1023), 'VBA physical lines must not exceed 1,023 characters');
  let continuationCount = 0;
  for (const line of lines) {
    continuationCount = /\s_\s*(?:'.*)?$/.test(line) ? continuationCount + 1 : 0;
    assert.ok(continuationCount <= 24, 'a VBA logical statement cannot use more than 24 continuations');
  }
  const declarations = [...source.matchAll(/^(?:Public|Private)\s+(?:Sub|Function)\s+(\w+)/gmi)].map((match) => match[1].toLowerCase());
  assert.equal(new Set(declarations).size, declarations.length, 'procedure names must be unique');
  assert.equal((source.match(/^\s*End Sub\s*$/gmi) ?? []).length, (source.match(/^(?:Public|Private)\s+Sub\s+/gmi) ?? []).length);
  assert.equal((source.match(/^\s*End Function\s*$/gmi) ?? []).length, (source.match(/^(?:Public|Private)\s+Function\s+/gmi) ?? []).length);
  assert.doesNotMatch(source, /\bSet\s+\w+\([^\n]*\)\s*=/i, 'object assignment cannot target a function invocation');
});

test('embedded unit registry exactly covers the canonical offline registry', async () => {
  const source = await moduleSource();
  const canonical = JSON.parse(await readFile(new URL('../data/wellforge-unit-registry.json', import.meta.url), 'utf8'));
  const blocks = [...source.matchAll(/Private Const UNIT_TABLE_\d+ As String = _([\s\S]*?)(?=Private Const UNIT_TABLE_|Public Sub)/g)]
    .map((match) => [...match[1].matchAll(/"([^"]*)"/g)].map((part) => part[1]).join(''));
  const embedded = Object.fromEntries(blocks.join('').split(';').map((record) => {
    const [symbol, dimensions, multiplier, offset] = record.split('|');
    return [symbol, { dimensions: dimensions.split(','), multiplier: Number(multiplier), offset: Number(offset) }];
  }));
  assert.deepEqual(Object.keys(embedded).sort(), Object.keys(canonical).sort());
  for (const [symbol, definition] of Object.entries(canonical)) {
    assert.deepEqual(new Set(embedded[symbol].dimensions), new Set([definition.dimension, ...(definition.dimensions ?? [])]), symbol);
    assert.equal(embedded[symbol].multiplier, definition.toSiMultiplier, symbol);
    assert.equal(embedded[symbol].offset, definition.toSiOffset, symbol);
  }
});

test('recursive-descent codec enforces complete JSON and Unicode contracts', async () => {
  const source = await moduleSource();
  const signatures = [
    /Public Function JsonParse\(ByVal Text As String\) As Variant/,
    /Private Function ParseValue\(ByRef Cursor As Long, ByVal Text As String\) As Variant/,
    /Private Function ParseObject\(ByRef Cursor As Long, ByVal Text As String\) As Object/,
    /Private Function ParseArray\(ByRef Cursor As Long, ByVal Text As String\) As Collection/,
    /Private Function ParseString\(ByRef Cursor As Long, ByVal Text As String\) As String/,
    /Public Function JsonStringify\(ByVal Value As Variant, Optional ByVal Indent As Long = 0\) As String/,
  ];
  for (const signature of signatures) assert.match(source, signature);

  const parse = procedure(source, 'JsonParse');
  const object = procedure(source, 'ParseObject');
  const string = procedure(source, 'ParseString');
  const number = procedure(source, 'ParseNumber');
  const stringifyNumber = procedure(source, 'JsonNumber');
  assert.match(parse, /trailing/i);
  assert.match(object, /\.Exists\(/i, 'duplicate object members must be detected');
  assert.match(string, /surrogate/i);
  assert.match(string, /&HD800|55296/i);
  assert.match(string, /&HDC00|56320/i);
  assert.match(number, /[eE].*sign|exponent/is);
  assert.match(number, /finite/i);
  assert.match(stringifyNumber, /Application\.DecimalSeparator/i);
  assert.match(procedure(source, 'JsonStringify'), /vbNullString|null/i);
});

test('UTF-8 input and atomic output use sibling temp files and timestamped backups', async () => {
  const source = await moduleSource();
  const read = procedure(source, 'ReadUtf8File');
  const write = procedure(source, 'WriteUtf8File');
  const atomic = procedure(source, 'AtomicWriteUtf8');
  assert.match(read, /Charset\s*=\s*"utf-8"/i);
  assert.match(write, /Charset\s*=\s*"utf-8"/i);
  assert.match(write, /adSaveCreateOverWrite|2/);
  assert.match(atomic, /\.tmp/i);
  assert.match(atomic, /\.bak/i);
  assert.match(atomic, /Format\$\([^\n]+yyyymmdd[^\n]+hhnnss/i);
  assert.match(atomic, /FileCopy/i);
  assert.match(atomic, /Name\s+.+\s+As\s+/i);
});

test('mapping is workbook-owned and import never accepts a sheet or address from JSON', async () => {
  const source = await moduleSource();
  const map = procedure(source, 'ReadExchangeMap');
  const changes = procedure(source, 'BuildChangeSet');
  assert.match(map, /Worksheets\(MAP_SHEET\)/i);
  assert.match(map, /Cells\([^\n]+,[^\n]+\)/i);
  assert.match(map, /ValidateMapDestination/i);
  assert.match(changes, /mapping\("sheet"\)/i);
  assert.match(changes, /mapping\("address"\)/i);
  assert.doesNotMatch(changes, /payload\([^\n]*(?:sheet|address)/i);
  assert.match(procedure(source, 'ValidateMapDestination'), /Like\s+"\[A-Z\]\*"|Address/i);
});

test('validation, conversion, stable-ID merge, and state/buffer contracts are explicit', async () => {
  const source = await moduleSource();
  for (const routine of [
    'ReadExchangeMap', 'ValidatePayload', 'BuildChangeSet', 'ApplyChangeSet',
    'RestoreChangeSet', 'ReadExchangeState', 'WriteExchangeState', 'MergePayload',
    'ToSi', 'FromSi',
  ]) procedure(source, routine);

  const validation = procedure(source, 'ValidatePayload')
    + procedure(source, 'ValidateIdentifiedPath')
    + procedure(source, 'ValidateQuantitiesRecursive');
  assert.match(validation, /SCHEMA_VERSION/i);
  assert.match(validation, /caseId/i);
  assert.match(validation, /duplicate/i);
  assert.match(validation, /unit/i);
  assert.match(validation, /ValidateQuantityMembers/i, 'operatingPoint members must all be quantity objects');
  const quantityValidation = procedure(source, 'ValidateQuantityMembers')
    + procedure(source, 'ValidateQuantitiesRecursive');
  assert.match(quantityValidation, /quality/i);
  assert.match(quantityValidation, /source/i);
  assert.match(quantityValidation, /timestamp/i);
  assert.match(quantityValidation, /note/i);

  const conversions = normalized(procedure(source, 'ToSi') + procedure(source, 'FromSi'));
  assert.match(conversions, /LookupUnit/i);
  assert.match(conversions, /dimension/i);
  assert.match(conversions, /multiplier/i);
  assert.match(conversions, /offset/i);

  const merge = procedure(source, 'MergePayload');
  assert.match(merge, /id/i);
  assert.match(merge, /\.Exists\(/i);
  assert.match(merge, /Collection/i);

  assert.match(procedure(source, 'ReadExchangeBuffer'), /BUFFER_PAYLOAD_CELL/i);
  assert.match(procedure(source, 'WriteExchangeBufferStatus'), /BUFFER_STATUS_CELL|BUFFER_DIAGNOSTICS_CELL/i);
  assert.match(source, /Private Const BUFFER_PAYLOAD_CELL As String = "B5"/);
});

test('imports are formula-protected transactions with rollback before recalculation', async () => {
  const source = await moduleSource();
  const build = procedure(source, 'BuildChangeSet')
    + procedure(source, 'AddMappedChange')
    + procedure(source, 'AddLiteralChange');
  const apply = procedure(source, 'ApplyChangeSet');
  const restore = procedure(source, 'RestoreChangeSet');
  const load = procedure(source, 'ImportPayloadText');
  assert.match(build, /HasFormula/i);
  assert.match(build, /oldValue/i);
  assert.match(build, /oldFormula/i);
  assert.match(apply, /HasFormula/i, 'formula status must be rechecked immediately before every write');
  assert.match(apply, /\.Value2\s*=/i);
  assert.match(restore, /oldFormula/i);
  assert.match(restore, /oldValue/i);
  assert.match(load, /On Error GoTo Rollback/i);
  assert.match(load, /BuildChangeSet[\s\S]*ApplyChangeSet[\s\S]*WriteExchangeState/i);
  assert.match(load, /Rollback:[\s\S]*RestoreChangeSet/i);
});

test('exchange rollback is verified at runtime through the real import transaction', async () => {
  const source = await moduleSource();
  assert.match(source, /Private WF_EXCHANGE_INJECT_WRITE_FAILURE As Boolean/i);
  assert.match(source, /Private WF_EXCHANGE_LAST_ROLLBACK_VERIFIED As Boolean/i);
  assert.match(procedure(source, 'ApplyChangeSet'), /WF_EXCHANGE_INJECT_WRITE_FAILURE/i);
  assert.match(procedure(source, 'ApplyChangeSet'), /WF_ExchangeFaultValue/i);
  assert.match(procedure(source, 'WF_ExchangeFaultValue'), /WELLFORGE_ROLLBACK_PROBE/i);
  assert.match(procedure(source, 'ImportPayloadText'), /WF_ExchangeChangesRestored/i);
  const selfTest = procedure(source, 'WellForge_ExchangeRollbackSelfTest');
  assert.match(selfTest, /ImportPayloadText/i);
  assert.match(selfTest, /WF_EXCHANGE_LAST_ROLLBACK_VERIFIED/i);
});

test('table capacity and formula-like text cannot corrupt workbook-owned rows', async () => {
  const source = await moduleSource();
  const build = procedure(source, 'BuildChangeSet');
  const assign = procedure(source, 'AssignStableRow');
  const validateIds = procedure(source, 'ValidateStableIdColumn');
  const decode = procedure(source, 'DecodeMappedValue');
  const literal = procedure(source, 'LiteralCellText');
  assert.match(build, /ValidateStableIdColumn/i, 'all workbook IDs must be checked before matching');
  assert.match(validateIds, /seen\.Exists\(stableId\)/i);
  assert.match(assign, /AssignStableRow\s*=\s*0/i, 'a full table must retain an unknown JSON record off-sheet');
  assert.match(build, /If rowNumber > 0 Then/i, 'off-sheet records must not become row zero writes');
  assert.match(decode, /LiteralCellText/i);
  for (const control of ['=', '\\+', '-', '@']) assert.match(literal, new RegExp(`firstCharacter = "${control}"`));
  assert.match(literal, /"'"\s*&\s*Value/i, 'formula-like JSON text must be stored literally');
});

test('round-trip state preserves an imported unit only while its canonical value is unchanged', async () => {
  const source = await moduleSource();
  const encode = procedure(source, 'EncodeMappedValue');
  const compare = procedure(source, 'NearlyEqual');
  assert.match(encode, /canonicalValue/i);
  assert.match(encode, /stateEntry\("canonicalValue"\)/i);
  assert.match(encode, /NearlyEqual/i);
  assert.match(encode, /wireUnit\s*=\s*destinationUnit/i, 'changed values must fall back to the mapped unit');
  assert.match(encode, /originalUnit/i);
  assert.match(compare, /Abs\(leftValue - rightValue\)/i);
});

test('public macros use dialogs, Buffer B5, and one cleanup path restoring Excel state', async () => {
  const source = await moduleSource();
  const load = procedure(source, 'WellForge_LoadJson');
  const save = procedure(source, 'WellForge_SaveJson');
  const validate = procedure(source, 'WellForge_ValidateExchange');
  assert.match(load, /FileDialog\(msoFileDialogFilePicker\)/i);
  assert.match(save, /FileDialog\(msoFileDialogSaveAs\)/i);
  assert.match(load, /ReadUtf8File/i);
  assert.match(save, /AtomicWriteUtf8/i);
  assert.match(load, /ImportPayloadText/i);
  assert.match(save, /ExportPayloadText/i);
  assert.match(validate, /ReadExchangeBuffer/i);
  assert.match(validate, /ValidatePayload/i);

  for (const [name, body] of [['load', load], ['save', save]]) {
    assert.match(body, /Cleanup:/i, `${name}: single cleanup label`);
    assert.equal((body.match(/^Cleanup:/gim) ?? []).length, 1, `${name}: exactly one cleanup label`);
    assert.match(body, /Application\.Calculation\s*=\s*oldCalculation/i);
    assert.match(body, /Application\.EnableEvents\s*=\s*oldEnableEvents/i);
    assert.match(body, /Application\.ScreenUpdating\s*=\s*oldScreenUpdating/i);
  }
});

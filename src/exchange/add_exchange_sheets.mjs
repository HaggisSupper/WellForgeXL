import { COLORS, formatInput, formatOutput, sectionHeader } from '../common.mjs';
import { EXCHANGE_SHEET_NAMES, WORKBOOK_MAPS } from './workbook_maps.mjs';

const MAP_HEADERS = ['JSON Pointer', 'Direction', 'Sheet', 'Address', 'Shape', 'Value column', 'Stable ID column', 'Row capacity', 'Unit source', 'Dimension', 'Data type', 'Required', 'Writable'];
const STATE_HEADERS = ['Pointer', 'Original value', 'Original unit', 'Canonical value', 'Destination', 'Imported at'];

function getOrAdd(workbook, name) {
  return workbook.worksheets.items.find((sheet) => sheet.name === name) ?? workbook.worksheets.add(name);
}

function styleHeader(sheet, address) {
  sheet.getRange(address).format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, borders: { preset: 'all', style: 'thin', color: COLORS.line }, wrapText: true };
}

export function addExchangeSheets(workbook, workbookKind) {
  const mappings = WORKBOOK_MAPS[workbookKind];
  if (!mappings) throw new Error(`Unknown workbook mapping kind: ${workbookKind}`);
  const [map, state, buffer] = EXCHANGE_SHEET_NAMES.map((name) => getOrAdd(workbook, name));

  sectionHeader(map, 'A3:M3', 'Declarative JSON exchange destinations — documentation only; automation reads this protected table');
  map.getRange('A5:M5').values = [MAP_HEADERS];
  map.getRange(`A6:M${5 + mappings.length}`).values = mappings.map((mapping) => [
    mapping.pointer, mapping.direction, mapping.sheet, mapping.address, mapping.shape,
    mapping.valueColumn ?? '', mapping.idColumn ?? '', mapping.capacity ?? '', mapping.unitSource,
    mapping.dimension, mapping.dataType, mapping.required, mapping.writable,
  ]);
  styleHeader(map, 'A5:M5');
  map.getRange(`A6:M${5 + mappings.length}`).format.borders = { preset: 'all', style: 'thin', color: COLORS.line };
  map.getRange('A:A').format.columnWidth = 48;
  map.getRange('B:M').format.columnWidth = 16;
  map.getRange('A3:M3').format.wrapText = true;
  map.freezePanes.freezeRows(5);
  map.visibility = 'visible';
  map.protection = Object.freeze({ protected: true, editableRanges: Object.freeze(['A3:M3']) });

  sectionHeader(state, 'A3:F3', 'Round-trip unit preservation state — managed by WellForge exchange automation');
  state.getRange('A5:F5').values = [STATE_HEADERS];
  styleHeader(state, 'A5:F5');
  state.getRange('A:F').format.columnWidth = 24;
  state.getRange('A:A').format.columnWidth = 48;
  state.visibility = 'hidden';
  state.protection = Object.freeze({ protected: true, editableRanges: Object.freeze([]) });

  sectionHeader(buffer, 'A3:B3', 'Office Script and desktop JSON exchange buffer');
  buffer.getRange('A5:B8').values = [['Payload', ''], ['Action', ''], ['Status', 'Ready'], ['Diagnostics', '']];
  styleHeader(buffer, 'A5:A8');
  formatInput(buffer.getRange('B5:B6'));
  formatOutput(buffer.getRange('B7:B8'));
  buffer.getRange('B5:B8').format.wrapText = true;
  buffer.getRange('A:A').format.columnWidth = 18;
  buffer.getRange('B:B').format.columnWidth = 72;
  buffer.getRange('5:5').format.rowHeight = 96;
  buffer.visibility = 'visible';

  return { map, state, buffer };
}

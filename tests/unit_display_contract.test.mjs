import test from 'node:test';
import assert from 'node:assert/strict';
import { FileBlob, SpreadsheetFile } from '@oai/artifact-tool';

const root = new URL('..', import.meta.url);
const cases = [
  ['API_7G_Drill_String_Strength_and_Torque_SI.xlsx', 'Results', 'B6'],
  ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Results', 'B6'],
  ['Torque_Drag_and_Buckling_SI.xlsx', 'Results', 'A6'],
  ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'Results', 'C6'],
  ['Directional_Drilling_Wellplan_and_Survey_SI.xlsx', 'Summary', 'B6'],
];

test('result displays convert SI calculations using Unit Map formulas', async () => {
  for (const [name, sheetName, cell] of cases) {
    const file = await FileBlob.load(new URL(`outputs/${name}`, root).pathname);
    const wb = await SpreadsheetFile.importXlsx(file);
    const formula = wb.worksheets.getItem(sheetName).getRange(cell).formulas[0][0];
    assert.match(formula, /'Unit Map'!/, `${name} ${sheetName}!${cell} does not use Unit Map`);
  }
});

test('directional workbook keeps raw-input conversion, canonical SI, and display conversion separate', async () => {
  const file = await FileBlob.load(new URL('outputs/Directional_Drilling_Wellplan_and_Survey_SI.xlsx', root).pathname);
  const wb = await SpreadsheetFile.importXlsx(file);
  const rawToCanonical = wb.worksheets.getItem('Calc').getRange('C7').formulas[0][0];
  const displayFormula = wb.worksheets.getItem('Summary').getRange('B6').formulas[0][0];
  assert.match(rawToCanonical, /'?Inputs'?\!\$?E\$?5/);
  assert.match(rawToCanonical, /'Unit Map'!\$?E\$?8/);
  assert.doesNotMatch(rawToCanonical, /'Unit Map'!\$?B\$?5|'Unit Map'!\$?I\$?8/);
  assert.match(displayFormula, /'Unit Map'!\$?I\$?8/);
});

test('every calculated view derives its UOM label and displayed value from Unit Map', async () => {
  const cases = [
    ['API_7G_Drill_String_Strength_and_Torque_SI.xlsx', 'Section Detail', 'C5', 'C6', 14],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Flow Path', 'D5', 'D6', 8],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Pressure Profile', 'B5', 'B6', 8],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Nozzle Cases', 'B5', 'B6', 9],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Nozzle Cases', 'D5', 'D6', 10],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Nozzle Cases', 'E5', 'E6', 19],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Nozzle Cases', 'F5', 'F6', 15],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Graphs', 'E40', 'E41', 9],
    ['Steady_State_Hydraulics_and_Nozzle_Optimization_SI.xlsx', 'Hydraulics Charts', 'A63', 'A64', 9],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'BHA Assembly', 'C5', 'C6', 8],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'BHA Assembly', 'F5', 'F6', 9],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'BHA Assembly', 'H5', 'H6', 10],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'Results', 'G5', 'G6', 18],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'Graphs', 'C3', 'C4', 17],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'Bending Response', 'B5', 'B6', 8],
    ['BHA_Vibration_Bending_and_Drill_Ahead_Tendency_SI.xlsx', 'Tendency Matrix', 'C5', 'C6', 14],
    ['Directional_Drilling_Wellplan_and_Survey_SI.xlsx', 'Plan', 'G5', 'G7', 8],
    ['Directional_Drilling_Wellplan_and_Survey_SI.xlsx', 'Survey', 'G5', 'G7', 8],
    ['Directional_Drilling_Wellplan_and_Survey_SI.xlsx', 'Calc', 'DI6', 'DI7', 8],
  ];
  const cache = new Map();
  for (const [name, sheetName, labelCell, valueCell, unitRow] of cases) {
    if (!cache.has(name)) {
      const file = await FileBlob.load(new URL(`outputs/${name}`, root).pathname);
      cache.set(name, await SpreadsheetFile.importXlsx(file));
    }
    const sheet = cache.get(name).worksheets.getItem(sheetName);
    const labelFormula = sheet.getRange(labelCell).formulas[0][0];
    const valueFormula = sheet.getRange(valueCell).formulas[0][0];
    assert.match(labelFormula, new RegExp(`'Unit Map'!\\$?H\\$?${unitRow}`), `${name} ${sheetName}!${labelCell} has a fixed UOM label`);
    assert.match(valueFormula, new RegExp(`'Unit Map'!\\$?I\\$?${unitRow}`), `${name} ${sheetName}!${valueCell} stays in canonical units on the calculated view`);
  }

  const headerOnlyCases = [
    ['Torque_Drag_and_Buckling_SI.xlsx', 'ALL', 'A5', 8],
    ['Torque_Drag_and_Buckling_SI.xlsx', 'PUW', 'A5', 8],
    ['Torque_Drag_and_Buckling_SI.xlsx', 'Graphs', 'A3', 8],
  ];
  for (const [name, sheetName, labelCell, unitRow] of headerOnlyCases) {
    if (!cache.has(name)) {
      const file = await FileBlob.load(new URL(`outputs/${name}`, root).pathname);
      cache.set(name, await SpreadsheetFile.importXlsx(file));
    }
    const formula = cache.get(name).worksheets.getItem(sheetName).getRange(labelCell).formulas[0][0];
    assert.match(formula, new RegExp(`'Unit Map'!\\$?H\\$?${unitRow}`), `${name} ${sheetName}!${labelCell} has a fixed UOM label`);
  }
});

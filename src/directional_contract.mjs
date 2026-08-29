import { UNIT_ROWS } from './common.mjs';
import { EXCHANGE_SHEET_NAMES } from './exchange/workbook_maps.mjs';

export const DIRECTIONAL_SHEET_NAMES = Object.freeze(['Summary', 'Inputs', 'Plan', 'Survey', 'Targets', 'Slide Performance', 'Formation Tops', 'Results', 'Graphs', 'Chart Settings', 'Unit Map', 'Checks', 'Calc', ...EXCHANGE_SHEET_NAMES]);

export const DIRECTIONAL_CAPACITIES = Object.freeze({ plan: 500, survey: 500, targets: 100, slideIntervals: 200, formationTops: 100 });

function columns(headers) {
  return Object.fromEntries(headers.map(([key, letter, header]) => [key, { letter, header }]));
}

function table(sheetName, tableName, capacity, columnMap, editableColumns) {
  const headerRow = 6;
  const firstDataRow = 7;
  return Object.freeze({
    sheetName, tableName, capacity, headerRow, firstDataRow,
    lastDataRow: firstDataRow + capacity - 1,
    columns: columns(columnMap),
    editableColumns,
    calculatedColumns: columnMap.map(([key]) => key).filter((key) => !editableColumns.includes(key)),
  });
}

export const DIRECTIONAL_TABLES = Object.freeze({
  plan: table('Plan', 'DirectionalPlanInput', DIRECTIONAL_CAPACITIES.plan, [
    ['stationId', 'A', 'Station ID'], ['md', 'B', 'MD'], ['inc', 'C', 'Inc'], ['azi', 'D', 'Azi'], ['source', 'E', 'Source / Comment'],
    ['active', 'F', 'Active'], ['tvd', 'G', 'TVD'], ['north', 'H', 'North'], ['east', 'I', 'East'], ['verticalSection', 'J', 'Vertical Section'],
    ['crossline', 'K', 'Crossline'], ['dls', 'L', 'DLS'], ['rowQc', 'M', 'Row QC'], ['recordId', 'N', 'Exchange Record ID'],
  ], ['stationId', 'md', 'inc', 'azi', 'source', 'recordId']),
  survey: table('Survey', 'DirectionalSurveyInput', DIRECTIONAL_CAPACITIES.survey, [
    ['stationId', 'A', 'Station ID'], ['md', 'B', 'MD'], ['inc', 'C', 'Inc'], ['azi', 'D', 'Azi'], ['source', 'E', 'Source / Comment'],
    ['active', 'F', 'Active'], ['tvd', 'G', 'TVD'], ['north', 'H', 'North'], ['east', 'I', 'East'], ['verticalSection', 'J', 'Vertical Section'],
    ['crossline', 'K', 'Crossline'], ['planTvd', 'L', 'Plan TVD'], ['planNorth', 'M', 'Plan North'], ['planEast', 'N', 'Plan East'],
    ['deltaTvd', 'O', 'Delta TVD'], ['deltaNorth', 'P', 'Delta North'], ['deltaEast', 'Q', 'Delta East'], ['deltaVs', 'R', 'Delta VS'],
    ['alongError', 'S', 'Along-track Error'], ['crosslineError', 'T', 'Crossline Error'], ['horizontalError', 'U', 'Horizontal Error'],
    ['error3d', 'V', '3D Error'], ['coverage', 'W', 'Coverage'], ['dls', 'X', 'DLS'], ['rowQc', 'Y', 'Row QC'], ['recordId', 'Z', 'Exchange Record ID'],
  ], ['stationId', 'md', 'inc', 'azi', 'source', 'recordId']),
  targets: table('Targets', 'DirectionalTargetsInput', DIRECTIONAL_CAPACITIES.targets, [
    ['id', 'A', 'Target ID'], ['md', 'B', 'Target MD'], ['centerNorth', 'C', 'Center North'], ['centerEast', 'D', 'Center East'],
    ['centerTvd', 'E', 'Center TVD'], ['type', 'F', 'Type'], ['major', 'G', 'Major / Half-length'], ['minor', 'H', 'Minor / Half-width'],
    ['rotation', 'I', 'Rotation'], ['verticalTolerance', 'J', 'Vertical Tolerance'], ['note', 'K', 'Note'], ['basis', 'L', 'Basis'],
    ['localMajor', 'M', 'Local Major'], ['localMinor', 'N', 'Local Minor'], ['envelopeUtilization', 'O', 'Envelope Utilization'],
    ['verticalUtilization', 'P', 'Vertical Utilization'], ['status', 'Q', 'Status'],
  ], ['id', 'md', 'centerNorth', 'centerEast', 'centerTvd', 'type', 'major', 'minor', 'rotation', 'verticalTolerance', 'note']),
  slideIntervals: table('Slide Performance', 'DirectionalSlidePerformanceInput', DIRECTIONAL_CAPACITIES.slideIntervals, [
    ['stand', 'A', 'Stand'], ['date', 'B', 'Date'], ['mdIn', 'C', 'MD In'], ['mdOut', 'D', 'MD Out'], ['slideLength', 'E', 'Slide Length'],
    ['rotateLength', 'F', 'Rotate Length'], ['commandedToolface', 'G', 'Commanded Toolface'], ['rotaryBuild', 'H', 'Rotary Build Background'],
    ['rotaryTurn', 'I', 'Rotary Effective-turn Background'], ['source', 'J', 'Source'], ['buildResponse', 'K', 'Build Response'],
    ['effectiveTurn', 'L', 'Effective-turn Response'], ['residualBuild', 'M', 'Residual Build'], ['residualTurn', 'N', 'Residual Effective Turn'],
    ['slideYield', 'O', 'Slide Yield'], ['responseToolface', 'P', 'Response Toolface'], ['toolfaceError', 'Q', 'Toolface Error'],
    ['rollingYield', 'R', 'Rolling Yield'], ['rowQc', 'S', 'Row QC'], ['recordId', 'T', 'Exchange Record ID'],
  ], ['stand', 'date', 'mdIn', 'mdOut', 'slideLength', 'rotateLength', 'commandedToolface', 'rotaryBuild', 'rotaryTurn', 'source', 'recordId']),
  formationTops: table('Formation Tops', 'DirectionalFormationTopsInput', DIRECTIONAL_CAPACITIES.formationTops, [
    ['name', 'A', 'Formation Name'], ['prognosedMd', 'B', 'Prognosed MD'], ['prognosedTvd', 'C', 'Prognosed TVD'],
    ['actualPickMd', 'D', 'Actual Pick MD'], ['verticalTolerance', 'E', 'Vertical Tolerance'], ['note', 'F', 'Note'],
    ['actualTvd', 'G', 'Actual TVD'], ['highLow', 'H', 'High (+) / Low (-)'], ['structuralSense', 'I', 'Structural Sense'],
    ['coverage', 'J', 'Coverage'], ['rowQc', 'K', 'Row QC'], ['recordId', 'L', 'Exchange Record ID'],
  ], ['name', 'prognosedMd', 'prognosedTvd', 'actualPickMd', 'verticalTolerance', 'note', 'recordId']),
});

export const DIRECTIONAL_INPUT_CELLS = Object.freeze({
  metadata: { wellName: 'B5', wellIdentifier: 'B6', operator: 'B7', fieldPad: 'B8', rig: 'B9', datum: 'B10', northReference: 'B11', coordinateReference: 'B12', surfaceNorth: 'B13', surfaceEast: 'B14', groundElevation: 'B15', verticalSectionAzimuth: 'B16' },
  rawUnits: { planLength: 'E5', planAngle: 'E6', surveyLength: 'E7', surveyAngle: 'E8', targetLength: 'E9', slideLength: 'E10', formationLength: 'E11' },
  dls: { limit: 'H5', unit: 'H6', source: 'H7' },
  projection: { bitMd: 'K5', aheadMd: 'K6', buildTendency: 'K7', effectiveTurnTendency: 'K8', gradientUnit: 'K9' },
  controls: { lowInclination: 'N5', minimumSlideLength: 'N6', slideYieldOutlier: 'N7', calibrationWindow: 'N8', surveyGapWarning: 'N9' },
});

export function directionalUnitRow(domain) {
  const index = UNIT_ROWS.findIndex((row) => row.domain === domain);
  if (index < 0) throw new Error(`Unknown directional unit domain: ${domain}`);
  return 8 + index;
}

export const lengthUnitRow = () => directionalUnitRow('Length');
export const angleUnitRow = () => directionalUnitRow('Angle');
export const angularGradientUnitRow = () => directionalUnitRow('Angular gradient');

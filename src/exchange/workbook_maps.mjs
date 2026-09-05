export const EXCHANGE_SHEET_NAMES = Object.freeze(['Exchange Map', 'Exchange State', 'Exchange Buffer']);

const freeze = (records) => Object.freeze(records.map((record) => Object.freeze(record)));

function scalar(pointer, direction, sheet, address, unitSource, dimension, dataType = 'number', required = true) {
  return { pointer, direction, sheet, address, shape: 'Scalar', unitSource, dimension, dataType, required, writable: direction === 'Input' || direction === 'Both' };
}

function table(pointer, direction, sheet, address, valueColumn, idColumn, capacity, unitSource, dimension, dataType = 'number', required = true) {
  return { pointer, direction, sheet, address, shape: 'Table', valueColumn, idColumn, idPointer: 'id', capacity, unitSource, dimension, dataType, required, writable: direction === 'Input' || direction === 'Both' };
}

const textScalar = (pointer, direction, sheet, address, required = true) => scalar(pointer, direction, sheet, address, 'text', 'text', 'string', required);
const statusScalar = (pointer, sheet, address) => scalar(pointer, 'Output', sheet, address, 'text', 'status', 'status', false);

const API7G_CAPACITY = 6;
const api7gSection = (field, column, unit, dimension, dataType = 'number') => table(`/analyses/api7g/sections/*/${field}`, 'Input', 'Inputs', `${column}6:${column}11`, column, 'I', API7G_CAPACITY, unit, dimension, dataType);
const api7gResult = (field, column, unit, dimension, dataType = 'number') => table(`/analyses/api7g/results/*/${field}`, 'Output', 'Results', `${column}6:${column}11`, column, 'I', API7G_CAPACITY, unit, dimension, dataType, false);

const api7g = freeze([
  api7gSection('length', 'B', 'm', 'length'), api7gSection('fluidDensity', 'C', 'kg/m3', 'density'),
  api7gSection('outerDiameter', 'D', 'm', 'diameter'), api7gSection('innerDiameter', 'E', 'm', 'diameter'),
  api7gSection('axialLoad', 'F', 'N', 'force'), api7gSection('tensionLimit', 'G', 'N', 'force'), api7gSection('operatingTorque', 'H', 'N*m', 'torque'),
  scalar('/analyses/api7g/controls/surfaceTorque', 'Input', 'Inputs', 'K6', 'N*m', 'torque'),
  scalar('/analyses/api7g/controls/hookload', 'Input', 'Inputs', 'K7', 'N', 'force'),
  scalar('/analyses/api7g/controls/designUtilisationLimit', 'Input', 'Inputs', 'K8', '1', 'unitless'),
  api7gResult('section', 'A', 'text', 'text', 'string'), api7gResult('buoyedLoad', 'B', 'Results!B4', 'force'),
  api7gResult('tensionUtilisation', 'C', '1', 'unitless'), api7gResult('torqueUtilisation', 'D', '1', 'unitless'),
  api7gResult('combinedUtilisation', 'E', '1', 'unitless'), api7gResult('tensionStatus', 'F', 'text', 'status', 'status'),
  api7gResult('torqueStatus', 'G', 'text', 'status', 'status'), api7gResult('governing', 'H', 'text', 'status', 'status'),
  textScalar('/analyses/api7g/summary/governingSection', 'Output', 'Summary', 'B6', false),
  scalar('/analyses/api7g/summary/maximumCombinedUtilisation', 'Output', 'Summary', 'B7', '1', 'unitless', 'number', false),
  statusScalar('/analyses/api7g/summary/status', 'Summary', 'B8'),
]);

const HYDRAULICS_CAPACITY = 8;
const hydraulicsFlowPath = (field, column, unit, dimension, dataType = 'number') => table(`/analyses/hydraulics/flowPath/*/${field}`, 'Input', 'Inputs', `${column}6:${column}13`, column, 'I', HYDRAULICS_CAPACITY, unit, dimension, dataType);
const hydraulics = freeze([
  textScalar('/analyses/hydraulics/rigPreset', 'Input', 'Inputs', 'B5', false),
  scalar('/rigLimits/surfacePressure', 'Input', 'Inputs', 'B6', 'Pa', 'pressure'), scalar('/analyses/hydraulics/pumpEfficiency', 'Input', 'Inputs', 'B7', '1', 'unitless'),
  scalar('/operatingPoint/flowRate', 'Input', 'Inputs', 'B8', 'm3/s', 'flowRate'), scalar('/analyses/hydraulics/fluidDensity', 'Input', 'Inputs', 'B9', 'kg/m3', 'density'),
  scalar('/analyses/hydraulics/apparentViscosity', 'Input', 'Inputs', 'B10', 'Pa*s', 'viscosity'), scalar('/analyses/hydraulics/nozzleCount', 'Input', 'Inputs', 'B11', '1', 'unitless', 'integer'),
  scalar('/analyses/hydraulics/baseNozzleDiameter', 'Input', 'Inputs', 'B12', 'm', 'diameter'), scalar('/analyses/hydraulics/nozzleDischargeCoefficient', 'Input', 'Inputs', 'B13', '1', 'unitless'),
  scalar('/rigLimits/ecdScreenDensity', 'Input', 'Inputs', 'B14', 'kg/m3', 'density'),
  scalar('/analyses/hydraulics/minimumAnnularVelocity', 'Input', 'Inputs', 'B15', 'm/s', 'speed', 'number', false),
  scalar('/analyses/hydraulics/ecdReferenceTvd', 'Input', 'Inputs', 'B16', 'm', 'length', 'number', false),
  scalar('/analyses/hydraulics/surfaceBackpressure', 'Input', 'Inputs', 'B17', 'Pa', 'pressure', 'number', false),
  textScalar('/analyses/hydraulics/pressureCorrelation', 'Input', 'Inputs', 'B18', false),
  textScalar('/analyses/hydraulics/computeBackend', 'Input', 'Inputs', 'B19', false),
  textScalar('/analyses/hydraulics/thermalAssumption', 'Input', 'Inputs', 'B20', false),
  textScalar('/analyses/hydraulics/rheology/model', 'Input', 'Fluid Model', 'G6', false),
  scalar('/analyses/hydraulics/rheology/flowBehaviorIndex', 'Input', 'Fluid Model', 'B8', '1', 'unitless', 'number', false),
  scalar('/analyses/hydraulics/rheology/consistencyCoefficient', 'Input', 'Fluid Model', 'B9', 'Pa*s^n', 'rheologyConsistency', 'number', false),
  scalar('/analyses/hydraulics/rheology/yieldStress', 'Input', 'Fluid Model', 'B10', 'Pa', 'stress', 'number', false),
  scalar('/analyses/hydraulics/rheology/plasticViscosity', 'Input', 'Fluid Model', 'B11', 'Pa*s', 'viscosity', 'number', false),
  scalar('/analyses/hydraulics/rheology/surfaceTemperature', 'Input', 'Fluid Model', 'B12', 'K', 'temperature', 'number', false),
  scalar('/analyses/hydraulics/rheology/compressibility', 'Input', 'Fluid Model', 'B13', '1/Pa', 'compressibility', 'number', false),
  scalar('/analyses/hydraulics/rheology/highShearFlowIndex', 'Input', 'Fluid Model', 'B14', '1', 'unitless', 'number', false),
  hydraulicsFlowPath('name', 'D', 'text', 'text', 'string'), hydraulicsFlowPath('length', 'E', 'm', 'length'), hydraulicsFlowPath('flowId', 'F', 'm', 'length'),
  hydraulicsFlowPath('flowType', 'G', 'text', 'text', 'string'), hydraulicsFlowPath('hydraulicDiameter', 'H', 'm', 'diameter'),
  table('/pumpNozzle/nozzles/*/diameter', 'Input', 'Calc', 'L6:L10', 'L', 'R', 5, 'm', 'diameter'),
  scalar('/analyses/hydraulics/results/totalFlowPathLoss', 'Output', 'Results', 'B6', 'Results!C6', 'pressure', 'number', false),
  scalar('/analyses/hydraulics/summary/recommendedNozzleDiameter', 'Output', 'Results', 'B7', 'Results!C7', 'diameter', 'number', false),
  scalar('/analyses/hydraulics/results/recommendedSurfacePressure', 'Output', 'Results', 'B8', 'Results!C8', 'pressure', 'number', false),
  scalar('/analyses/hydraulics/results/nozzleVelocity', 'Output', 'Results', 'B9', 'Results!C9', 'speed', 'number', false),
  scalar('/analyses/hydraulics/results/ecdScreeningInput', 'Output', 'Results', 'B10', 'Results!C10', 'density', 'number', false),
  statusScalar('/analyses/hydraulics/results/flowPathStatus', 'Results', 'E6'), statusScalar('/analyses/hydraulics/results/nozzleStatus', 'Results', 'E7'),
  statusScalar('/analyses/hydraulics/summary/surfacePressureStatus', 'Results', 'E8'), statusScalar('/analyses/hydraulics/results/nozzleVelocityStatus', 'Results', 'E9'), statusScalar('/analyses/hydraulics/results/ecdStatus', 'Results', 'E10'),
]);

const TORQUE_DRAG_CAPACITY = 60;
const torqueSurvey = (field, column, unit, dimension) => table(`/trajectory/survey/*/${field}`, 'Input', 'Survey', `${column}6:${column}65`, column, 'E', TORQUE_DRAG_CAPACITY, unit, dimension);
const torqueResult = (field, column, unit, dimension, dataType = 'number') => table(`/analyses/torqueDrag/results/*/${field}`, 'Output', 'Results', `${column}6:${column}65`, column, 'K', TORQUE_DRAG_CAPACITY, unit, dimension, dataType, false);
const torqueDrag = freeze([
  scalar('/analyses/torqueDrag/inputs/fluidDensity', 'Input', 'Inputs', 'B5', 'kg/m3', 'density'), scalar('/operatingPoint/frictionFactor', 'Input', 'Inputs', 'B6', '1', 'unitless'),
  scalar('/operatingPoint/wob', 'Input', 'Inputs', 'B7', 'N', 'force'), scalar('/operatingPoint/surfaceTorque', 'Input', 'Inputs', 'B8', 'N*m', 'torque'),
  scalar('/analyses/torqueDrag/inputs/outerDiameter', 'Input', 'Inputs', 'B9', 'm', 'diameter'), scalar('/analyses/torqueDrag/inputs/innerDiameter', 'Input', 'Inputs', 'B10', 'm', 'diameter'),
  scalar('/analyses/torqueDrag/inputs/materialDensity', 'Input', 'Inputs', 'B11', 'kg/m3', 'density'), scalar('/analyses/torqueDrag/inputs/youngModulus', 'Input', 'Inputs', 'B12', 'Inputs!C12', 'pressure'),
  torqueSurvey('md', 'A', 'm', 'length'), torqueSurvey('inclination', 'B', 'rad', 'angle'), torqueSurvey('azimuth', 'C', 'rad', 'angle'), torqueSurvey('holeDiameter', 'D', 'm', 'diameter'),
  torqueResult('md', 'A', 'Results!A4', 'length'), torqueResult('poohHookload', 'B', 'Results!B4', 'force'), torqueResult('rihAxialLoad', 'C', 'Results!C4', 'force'),
  torqueResult('slideTorque', 'D', 'Results!D4', 'torque'), torqueResult('rotateTorque', 'E', 'Results!E4', 'torque'), torqueResult('backreamTorque', 'F', 'Results!F4', 'torque'),
  torqueResult('sinusoidalLimit', 'G', 'Results!G4', 'force'), torqueResult('helicalLimit', 'H', 'Results!H4', 'force'), torqueResult('bucklingStatus', 'I', 'text', 'status', 'status'), torqueResult('governing', 'J', 'text', 'status', 'status'),
  scalar('/analyses/torqueDrag/summary/peakPoohHookload', 'Output', 'Summary', 'B6', 'Results!B4', 'force', 'number', false),
  scalar('/analyses/torqueDrag/summary/lowestRihAxialLoad', 'Output', 'Summary', 'B7', 'Results!C4', 'force', 'number', false),
  scalar('/analyses/torqueDrag/summary/governingDepth', 'Output', 'Summary', 'B8', 'Results!A4', 'length', 'number', false),
]);

const BHA_CAPACITY = 6;
const bhaComponent = (field, column, unit, dimension, dataType = 'number') => table(`/bhaComponents/*/${field}`, 'Input', 'Inputs', `${column}6:${column}11`, column, 'I', BHA_CAPACITY, unit, dimension, dataType);
const bhaResult = (field, column, unit, dimension, dataType = 'number') => table(`/analyses/bha/results/*/${field}`, 'Output', 'Results', `${column}6:${column}11`, column, 'F', BHA_CAPACITY, unit, dimension, dataType, false);
const bha = freeze([
  scalar('/operatingPoint/rotarySpeed', 'Input', 'Inputs', 'B5', 'rpm', 'rotationalSpeed'), scalar('/operatingPoint/flowRate', 'Input', 'Inputs', 'B6', 'm3/s', 'flowRate'),
  scalar('/analyses/bha/youngModulus', 'Input', 'Inputs', 'B7', 'Inputs!C7', 'pressure'), scalar('/analyses/bha/materialDensity', 'Input', 'Inputs', 'B8', 'kg/m3', 'density'),
  scalar('/analyses/bha/lowWob', 'Input', 'Inputs', 'B9', 'N', 'force'), scalar('/operatingPoint/wob', 'Input', 'Inputs', 'B10', 'N', 'force'),
  bhaComponent('name', 'D', 'text', 'text', 'string'), bhaComponent('length', 'E', 'm', 'length'), bhaComponent('outerDiameter', 'F', 'm', 'diameter'), bhaComponent('innerDiameter', 'G', 'm', 'diameter'), bhaComponent('supportFactor', 'H', '1', 'unitless'),
  bhaResult('component', 'A', 'text', 'text', 'string'), bhaResult('firstModeFrequency', 'B', 'Hz', 'frequency'), bhaResult('bendingStress', 'C', 'Results!C4', 'stress'),
  bhaResult('wobCase1Tendency', 'D', '1', 'unitless'), bhaResult('wobCase2Tendency', 'E', '1', 'unitless'),
  table('/analyses/bha/toolfaceResponse/*/toolface', 'Output', 'Results', 'G6:G17', 'G', 'N', 12, 'Unit Map!H18', 'angle', 'number', false),
  ...['H', 'I', 'J', 'K', 'L', 'M'].map((column, index) => table(`/analyses/bha/toolfaceResponse/*/${['wobCase1Magnitude', 'wobCase1X', 'wobCase1Y', 'wobCase2Magnitude', 'wobCase2X', 'wobCase2Y'][index]}`, 'Output', 'Results', `${column}6:${column}17`, column, 'N', 12, '1', 'unitless', 'number', false)),
  scalar('/analyses/bha/summary/lowestFirstModeFrequency', 'Output', 'Summary', 'B6', 'Hz', 'frequency', 'number', false),
  scalar('/analyses/bha/summary/peakBendingStress', 'Output', 'Summary', 'B7', 'Unit Map!H17', 'stress', 'number', false), statusScalar('/analyses/bha/summary/vibrationScreening', 'Summary', 'B8'),
]);

const DIRECTIONAL_TABLES = {
  plan: { sheet: 'Plan', capacity: 500, first: 7, idColumn: 'N', input: { station: ['A', '1', 'unitless', 'integer'], md: ['B', 'Inputs!E5', 'length'], inclination: ['C', 'Inputs!E6', 'angle'], azimuth: ['D', 'Inputs!E6', 'angle'], source: ['E', 'text', 'text', 'string'], id: ['N', 'text', 'text', 'string'] }, output: { active: ['F', '1', 'unitless', 'Boolean'], tvd: ['G', 'Unit Map!H8', 'length'], north: ['H', 'Unit Map!H8', 'length'], east: ['I', 'Unit Map!H8', 'length'], verticalSection: ['J', 'Unit Map!H8', 'length'], crossline: ['K', 'Unit Map!H8', 'length'], dls: ['L', 'Unit Map!H20', 'angularGradient'], rowQc: ['M', 'text', 'status', 'status'] } },
  survey: { sheet: 'Survey', capacity: 500, first: 7, idColumn: 'Z', input: { station: ['A', '1', 'unitless', 'integer'], md: ['B', 'Inputs!E7', 'length'], inclination: ['C', 'Inputs!E8', 'angle'], azimuth: ['D', 'Inputs!E8', 'angle'], source: ['E', 'text', 'text', 'string'], id: ['Z', 'text', 'text', 'string'] }, output: { active: ['F', '1', 'unitless', 'Boolean'], tvd: ['G', 'Unit Map!H8', 'length'], north: ['H', 'Unit Map!H8', 'length'], east: ['I', 'Unit Map!H8', 'length'], verticalSection: ['J', 'Unit Map!H8', 'length'], crossline: ['K', 'Unit Map!H8', 'length'], planTvd: ['L', 'Unit Map!H8', 'length'], planNorth: ['M', 'Unit Map!H8', 'length'], planEast: ['N', 'Unit Map!H8', 'length'], deltaTvd: ['O', 'Unit Map!H8', 'length'], deltaNorth: ['P', 'Unit Map!H8', 'length'], deltaEast: ['Q', 'Unit Map!H8', 'length'], deltaVs: ['R', 'Unit Map!H8', 'length'], alongError: ['S', 'Unit Map!H8', 'length'], crosslineError: ['T', 'Unit Map!H8', 'length'], horizontalError: ['U', 'Unit Map!H8', 'length'], error3d: ['V', 'Unit Map!H8', 'length'], coverage: ['W', 'text', 'status', 'status'], dls: ['X', 'Unit Map!H20', 'angularGradient'], rowQc: ['Y', 'text', 'status', 'status'] } },
  targets: { sheet: 'Targets', capacity: 100, first: 7, idColumn: 'A', input: { id: ['A', 'text', 'text', 'string'], md: ['B', 'Inputs!E9', 'length'], centerNorth: ['C', 'Inputs!E9', 'length'], centerEast: ['D', 'Inputs!E9', 'length'], centerTvd: ['E', 'Inputs!E9', 'length'], type: ['F', 'text', 'text', 'string'], major: ['G', 'Inputs!E9', 'length'], minor: ['H', 'Inputs!E9', 'length'], rotation: ['I', 'deg', 'angle'], verticalTolerance: ['J', 'Inputs!E9', 'length'], note: ['K', 'text', 'text', 'string'] }, output: { basis: ['L', 'text', 'text', 'string'], localMajor: ['M', 'Unit Map!H8', 'length'], localMinor: ['N', 'Unit Map!H8', 'length'], envelopeUtilization: ['O', '1', 'unitless'], verticalUtilization: ['P', '1', 'unitless'], status: ['Q', 'text', 'status', 'status'] } },
  slideIntervals: { sheet: 'Slide Performance', capacity: 200, first: 7, idColumn: 'T', input: { stand: ['A', '1', 'unitless', 'integer'], date: ['B', 'd', 'date'], mdIn: ['C', 'Inputs!E10', 'length'], mdOut: ['D', 'Inputs!E10', 'length'], slideLength: ['E', 'Inputs!E10', 'length'], rotateLength: ['F', 'Inputs!E10', 'length'], commandedToolface: ['G', 'Inputs!E8', 'angle'], rotaryBuild: ['H', 'Inputs!K9', 'angularGradient', 'number', false], rotaryTurn: ['I', 'Inputs!K9', 'angularGradient', 'number', false], source: ['J', 'text', 'text', 'string'], id: ['T', 'text', 'text', 'string'] }, output: { buildResponse: ['K', 'Unit Map!H20', 'angularGradient'], effectiveTurn: ['L', 'Unit Map!H20', 'angularGradient'], residualBuild: ['M', 'Unit Map!H20', 'angularGradient'], residualTurn: ['N', 'Unit Map!H20', 'angularGradient'], slideYield: ['O', 'Unit Map!H20', 'angularGradient'], responseToolface: ['P', 'Unit Map!H18', 'angle'], toolfaceError: ['Q', 'Unit Map!H18', 'angle'], rollingYield: ['R', 'Unit Map!H20', 'angularGradient'], rowQc: ['S', 'text', 'status', 'status'] } },
  formationTops: { sheet: 'Formation Tops', capacity: 100, first: 7, idColumn: 'L', input: { name: ['A', 'text', 'text', 'string'], prognosedMd: ['B', 'Inputs!E11', 'length'], prognosedTvd: ['C', 'Inputs!E11', 'length'], actualPickMd: ['D', 'Inputs!E11', 'length', 'number', false], verticalTolerance: ['E', 'Inputs!E11', 'length', 'number', false], note: ['F', 'text', 'text', 'string'], id: ['L', 'text', 'text', 'string'] }, output: { actualTvd: ['G', 'Unit Map!H8', 'length'], highLow: ['H', 'Unit Map!H8', 'length'], structuralSense: ['I', 'text', 'status', 'status'], coverage: ['J', 'text', 'status', 'status'], rowQc: ['K', 'text', 'status', 'status'] } },
};

const directionalTableMappings = Object.entries(DIRECTIONAL_TABLES).flatMap(([name, contract]) => {
  const last = contract.first + contract.capacity - 1;
  const make = (direction, [field, [column, unitSource, dimension, dataType = 'number', required = direction === 'Input']]) => table(`/trajectory/${name}/*/${field}`, direction, contract.sheet, `${column}${contract.first}:${column}${last}`, column, contract.idColumn, contract.capacity, unitSource, dimension, dataType, required);
  return [...Object.entries(contract.input).map((entry) => make('Input', entry)), ...Object.entries(contract.output).map((entry) => make('Output', entry))];
});

const directionalSurveyResult = (field, column, unitSource, dimension, dataType = 'number') => table(`/analyses/directional/results/survey/*/${field}`, 'Output', 'Results', `${column}26:${column}525`, column, 'M', 500, unitSource, dimension, dataType, false);
const directionalCheck = (field, column, dataType = 'string') => table(`/analyses/directional/checks/*/${field}`, 'Output', 'Checks', `${column}6:${column}25`, column, 'F', 20, 'text', dataType === 'status' ? 'status' : 'text', dataType, false);

const directional = freeze([
  textScalar('/metadata/well', 'Input', 'Inputs', 'B5'), textScalar('/metadata/wellIdentifier', 'Input', 'Inputs', 'B6', false), textScalar('/metadata/operator', 'Input', 'Inputs', 'B7', false),
  textScalar('/metadata/field', 'Input', 'Inputs', 'B8'), textScalar('/metadata/rig', 'Input', 'Inputs', 'B9'), textScalar('/metadata/datum', 'Input', 'Inputs', 'B10'),
  textScalar('/metadata/northReference', 'Input', 'Inputs', 'B11'), textScalar('/metadata/coordinateReference', 'Input', 'Inputs', 'B12', false),
  scalar('/metadata/surfaceNorth', 'Input', 'Inputs', 'B13', 'Inputs!E5', 'length'), scalar('/metadata/surfaceEast', 'Input', 'Inputs', 'B14', 'Inputs!E5', 'length'),
  scalar('/metadata/groundElevation', 'Input', 'Inputs', 'B15', 'Inputs!E5', 'length'), scalar('/metadata/verticalSectionAzimuth', 'Input', 'Inputs', 'B16', 'Inputs!E6', 'angle'),
  ...[['planLength', 'E5'], ['planAngle', 'E6'], ['surveyLength', 'E7'], ['surveyAngle', 'E8'], ['targetLength', 'E9'], ['slideLength', 'E10'], ['formationLength', 'E11']].map(([field, address]) => textScalar(`/unitPreferences/${field}`, 'Input', 'Inputs', address, false)),
  scalar('/analyses/directional/inputs/dlsLimit', 'Input', 'Inputs', 'H5', 'Inputs!H6', 'angularGradient', 'number', false), textScalar('/analyses/directional/inputs/dlsUnit', 'Input', 'Inputs', 'H6', false), textScalar('/analyses/directional/inputs/dlsSource', 'Input', 'Inputs', 'H7', false),
  scalar('/analyses/directional/inputs/projectionBitMd', 'Input', 'Inputs', 'K5', 'Inputs!E7', 'length', 'number', false), scalar('/analyses/directional/inputs/projectAheadDistance', 'Input', 'Inputs', 'K6', 'Inputs!E7', 'length', 'number', false),
  scalar('/analyses/directional/inputs/buildTendency', 'Input', 'Inputs', 'K7', 'Inputs!K9', 'angularGradient', 'number', false), scalar('/analyses/directional/inputs/effectiveTurnTendency', 'Input', 'Inputs', 'K8', 'Inputs!K9', 'angularGradient', 'number', false), textScalar('/analyses/directional/inputs/gradientUnit', 'Input', 'Inputs', 'K9', false),
  scalar('/analyses/directional/inputs/lowInclinationThreshold', 'Input', 'Inputs', 'N5', 'deg', 'angle', 'number', false), scalar('/analyses/directional/inputs/minimumSlideLength', 'Input', 'Inputs', 'N6', 'Inputs!E10', 'length', 'number', false),
  scalar('/analyses/directional/inputs/slideYieldOutlierLimit', 'Input', 'Inputs', 'N7', 'Inputs!K9', 'angularGradient', 'number', false), scalar('/analyses/directional/inputs/slideCalibrationWindow', 'Input', 'Inputs', 'N8', '1', 'unitless', 'integer'), scalar('/analyses/directional/inputs/surveyGapWarning', 'Input', 'Inputs', 'N9', 'Inputs!E7', 'length', 'number', false),
  ...directionalTableMappings,
  statusScalar('/analyses/directional/summary/decision', 'Summary', 'B5'), scalar('/analyses/directional/summary/latestSurveyMd', 'Output', 'Summary', 'B6', 'Summary!C6', 'length', 'number', false),
  scalar('/analyses/directional/summary/horizontalError', 'Output', 'Summary', 'B7', 'Summary!C7', 'length', 'number', false), scalar('/analyses/directional/summary/error3d', 'Output', 'Summary', 'B8', 'Summary!C8', 'length', 'number', false),
  statusScalar('/analyses/directional/summary/dlsAgainstLimit', 'Summary', 'B9'), statusScalar('/analyses/directional/summary/nextTargetAction', 'Summary', 'B10'),
  scalar('/analyses/directional/results/latestSurveyMd', 'Output', 'Results', 'B6', 'm', 'length', 'number', false),
  scalar('/analyses/directional/results/latestCoveredCrosslineError', 'Output', 'Results', 'B7', 'm', 'length', 'number', false),
  scalar('/analyses/directional/results/latestCovered3dError', 'Output', 'Results', 'B8', 'm', 'length', 'number', false),
  scalar('/analyses/directional/results/maximumActualDls', 'Output', 'Results', 'B9', 'rad/m', 'angularGradient', 'number', false),
  ...['planCoverage', 'nextTargetStatus', 'slideCalibrationStatus', 'formationStatus', 'projectionBasis', 'projectionConfidence'].map((field, index) => statusScalar(`/analyses/directional/results/${field}`, 'Results', `B${10 + index}`)),
  scalar('/analyses/directional/results/terminalCrosslineError', 'Output', 'Results', 'B19', 'Unit Map!H8', 'length', 'number', false), scalar('/analyses/directional/results/terminalHorizontalError', 'Output', 'Results', 'B20', 'Unit Map!H8', 'length', 'number', false), scalar('/analyses/directional/results/terminal3dError', 'Output', 'Results', 'B21', 'Unit Map!H8', 'length', 'number', false),
  directionalSurveyResult('station', 'A', '1', 'unitless', 'integer'), directionalSurveyResult('md', 'B', 'm', 'length'),
  directionalSurveyResult('inclination', 'C', 'rad', 'angle'), directionalSurveyResult('azimuth', 'D', 'rad', 'angle'),
  ...[['tvd', 'E'], ['north', 'F'], ['east', 'G'], ['verticalSection', 'H'], ['crossline', 'I']].map(([field, column]) => directionalSurveyResult(field, column, 'm', 'length')),
  directionalSurveyResult('dls', 'J', 'rad/m', 'angularGradient'), directionalSurveyResult('source', 'K', 'text', 'text', 'string'), directionalSurveyResult('rowStatus', 'L', 'text', 'status', 'status'),
  directionalCheck('check', 'A'), directionalCheck('measuredResult', 'B'), directionalCheck('status', 'C', 'status'), directionalCheck('severity', 'D', 'status'), directionalCheck('requiredAction', 'E'),
]);

export const WORKBOOK_MAPS = Object.freeze({ api7g, hydraulics, torqueDrag, bha, directional });

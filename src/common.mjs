export const UNIT_SYSTEMS = ['SI', 'Imperial', 'Mixed', 'Custom'];
export const CUSTOM_UNIT_SYSTEMS = ['SI', 'Imperial', 'Mixed'];
export const DISPLAY_NUMBER_FORMAT = '#,##0.00;[Red]-#,##0.00';
export const DISPLAY_PERCENT_FORMAT = '0.00%';

export const UNIT_ROWS = [
  { domain: 'Length', siUnit: 'm', imperialUnit: 'ft', mixedUnit: 'm', imperialMultiplier: 3.280839895, mixedMultiplier: 1, multiplier: 3.280839895, offset: 0 },
  { domain: 'Diameter', siUnit: 'm', imperialUnit: 'in', mixedUnit: 'mm', imperialMultiplier: 39.37007874, mixedMultiplier: 1000, multiplier: 39.37007874, offset: 0 },
  { domain: 'Area', siUnit: 'm2', imperialUnit: 'in2', mixedUnit: 'cm2', imperialMultiplier: 1550.0031, mixedMultiplier: 10000, multiplier: 1550.0031, offset: 0 },
  { domain: 'Volume', siUnit: 'm3', imperialUnit: 'bbl', mixedUnit: 'L', imperialMultiplier: 6.28981077, mixedMultiplier: 1000, multiplier: 6.28981077, offset: 0 },
  { domain: 'Flow rate', siUnit: 'm3/s', imperialUnit: 'gpm', mixedUnit: 'L/min', imperialMultiplier: 15850.32314, mixedMultiplier: 60000, multiplier: 15850.32314, offset: 0 },
  { domain: 'Density', siUnit: 'kg/m3', imperialUnit: 'ppg', mixedUnit: 'kg/m3', imperialMultiplier: 0.008345404, mixedMultiplier: 1, multiplier: 0.008345404, offset: 0 },
  { domain: 'Force', siUnit: 'N', imperialUnit: 'lbf', mixedUnit: 'kN', imperialMultiplier: 0.224808943, mixedMultiplier: 0.001, multiplier: 0.224808943, offset: 0 },
  { domain: 'Pressure', siUnit: 'Pa', imperialUnit: 'psi', mixedUnit: 'kPa', imperialMultiplier: 0.000145037738, mixedMultiplier: 0.001, multiplier: 0.000145037738, offset: 0 },
  { domain: 'Torque', siUnit: 'N-m', imperialUnit: 'ft-lbf', mixedUnit: 'kN-m', imperialMultiplier: 0.737562149, mixedMultiplier: 0.001, multiplier: 0.737562149, offset: 0 },
  { domain: 'Stress', siUnit: 'Pa', imperialUnit: 'psi', mixedUnit: 'MPa', imperialMultiplier: 0.000145037738, mixedMultiplier: 0.000001, multiplier: 0.000145037738, offset: 0 },
  { domain: 'Angle', siUnit: 'rad', imperialUnit: 'deg', mixedUnit: 'deg', imperialMultiplier: 57.295779513, mixedMultiplier: 57.295779513, multiplier: 57.295779513, offset: 0 },
  { domain: 'Speed', siUnit: 'm/s', imperialUnit: 'ft/min', mixedUnit: 'm/min', imperialMultiplier: 196.8503937, mixedMultiplier: 60, multiplier: 196.8503937, offset: 0 },
  { domain: 'Angular gradient', siUnit: 'rad/m', imperialUnit: 'deg/100ft', mixedUnit: 'deg/30m', imperialMultiplier: 1746.37535955875, mixedMultiplier: 1718.87338539247, multiplier: 1746.37535955875, offset: 0 },
];

export const COLORS = {
  charcoal: '#1F2933',
  dark: '#111827',
  teal: '#0F766E',
  tealLight: '#CCFBF1',
  amber: '#D97706',
  amberLight: '#FEF3C7',
  red: '#B91C1C',
  redLight: '#FEE2E2',
  input: '#DBEAFE',
  grey: '#F3F4F6',
  line: '#D1D5DB',
  white: '#FFFFFF',
};

export function displayFormula(siCell, multiplierCell, offsetCell = '0') {
  const offset = offsetCell === '0' ? '0' : `'Unit Map'!${offsetCell}`;
  return `=${siCell}*'Unit Map'!${multiplierCell}+${offset}`;
}

export function labelFormula(unitCell) {
  return `='Unit Map'!${unitCell}`;
}

export function addStatusBanner(sheet, title) {
  sheet.getRange('A1:N1').merge();
  sheet.getRange('A1').values = [[`WellForge | ${title} | Formula-driven planning and review workbook`]];
  sheet.getRange('A1:N1').format = {
    fill: COLORS.charcoal,
    font: { bold: true, color: COLORS.white, size: 12 },
    horizontalAlignment: 'left',
    verticalAlignment: 'center',
  };
  sheet.getRange('A1:N1').format.rowHeight = 26;
}

export function sectionHeader(sheet, rangeAddress, title) {
  const range = sheet.getRange(rangeAddress);
  range.merge();
  range.values = [[title]];
  range.format = {
    fill: COLORS.charcoal,
    font: { bold: true, color: COLORS.white },
    horizontalAlignment: 'left',
    verticalAlignment: 'center',
  };
}

export function formatInput(range) {
  range.format = { fill: COLORS.input, borders: { preset: 'outside', style: 'thin', color: COLORS.line } };
}

export function formatOutput(range) {
  range.format = { fill: COLORS.grey, borders: { preset: 'outside', style: 'thin', color: COLORS.line } };
}

export function applyTwoDecimalDisplayPrecision(workbook) {
  for (const sheet of workbook.worksheets.items) {
    const used = sheet.getUsedRange();
    if (used) used.format.numberFormat = DISPLAY_NUMBER_FORMAT;
    for (const chart of sheet.charts.items) {
      if (chart.xAxis) {
        chart.xAxis.numberFormatCode = DISPLAY_NUMBER_FORMAT;
        chart.xAxis.numberFormatSourceLinked = false;
      }
      if (chart.yAxis) {
        chart.yAxis.numberFormatCode = DISPLAY_NUMBER_FORMAT;
        chart.yAxis.numberFormatSourceLinked = false;
      }
    }
  }
  // Unit conversion factors are not operational results; retain their extra
  // precision so that rounding their display never implies rounded physics.
  const unitMap = workbook.worksheets.getItem('Unit Map');
  unitMap.getRange(`E8:G${7 + UNIT_ROWS.length}`).format.numberFormat = '0.000000';
  unitMap.getRange(`I8:I${7 + UNIT_ROWS.length}`).format.numberFormat = '0.000000';
}

export function baseSheet(sheet, title) {
  sheet.showGridLines = false;
  addStatusBanner(sheet, title);
  sheet.getRange('A:N').format.columnWidth = 12;
  sheet.getRange('A:A').format.columnWidth = 22;
}

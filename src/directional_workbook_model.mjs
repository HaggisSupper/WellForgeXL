import { COLORS, sectionHeader } from './common.mjs';
import { DIRECTIONAL_TABLES } from './directional_contract.mjs';
import { addDepthProfileChart } from './workbook.mjs';

const PRESENTATION_ONLY_STATE = 'PRESENTATION_ONLY_NOT_RUST_RESULT';
const RUST_REQUIRED_STATE = 'NOT_RUN_RUST_REQUIRED';
const RUST_UNAVAILABLE_STATE = 'UNAVAILABLE_NOT_IN_RUST_BRIDGE';
const PLAN_HEADERS = ['Active', 'Station ID', 'MD m', 'Inc rad', 'Azi rad', 'dMD m', 'Dogleg rad', 'RF', 'dTVD m', 'dN m', 'dE m', 'TVD m', 'North m', 'East m', 'VS State', 'Crossline State', 'DLS rad/m', 'Row Status'];
const SURVEY_HEADERS = [...PLAN_HEADERS];
const START = 7;
const END = 506;

const q = (sheet, cell) => `'${sheet}'!${cell}`;
const lengthInput = (raw, unitCell) => `IF(${q('Inputs', unitCell)}="ft",${raw}/${q('Unit Map', '$E$8')},${raw})`;
const angleInput = (raw, unitCell) => `IF(${q('Inputs', unitCell)}="deg",${raw}/${q('Unit Map', '$E$18')},${raw})`;
const displayLength = (si) => `${si}*${q('Unit Map', '$I$8')}`;
const displayGradient = (si) => `${si}*${q('Unit Map', '$I$20')}`;
const vsa = `${q('Inputs', '$B$16')}/${q('Unit Map', '$E$18')}`;
const gradientInput = (raw, unitCell) => `IF(${q('Inputs', unitCell)}="rad/m",${raw},IF(${q('Inputs', unitCell)}="deg/100ft",${raw}/${q('Unit Map', '$E$20')},IF(${q('Inputs', unitCell)}="deg/30m",${raw}/${q('Unit Map', '$F$20')},NA())))`;

const QUERY_TARGET_START = 7;
const QUERY_FORMATION_START = 110;
const QUERY_SLIDE_START = 213;

function canonicalPlanRow(row) {
  const p = row - 1;
  // COUNT, rather than `=""`, is deliberate here: MD zero is a valid first
  // station and must not be coerced to blank by lightweight formula engines.
  const active = `IF(COUNT(${q('Plan', `B${row}`)})=0,"",TRUE)`;
  if (row === START) return [
    `=${active}`, `=IF($A${row}="","",${q('Plan', `A${row}`)})`, `=IF($A${row}="","",${lengthInput(q('Plan', `B${row}`), '$E$5')})`,
    `=IF($A${row}="","",${angleInput(q('Plan', `C${row}`), '$E$6')})`, `=IF($A${row}="","",MOD(${angleInput(q('Plan', `D${row}`), '$E$6')},2*PI()))`,
    `=IF($A${row}="","",0)`, `=IF($A${row}="","",0)`, `=IF($A${row}="","",1)`, `=IF($A${row}="","",0)`, `=IF($A${row}="","",0)`, `=IF($A${row}="","",0)`,
    `=IF($A${row}="","",0)`, `=IF($A${row}="","",${lengthInput(q('Inputs', '$B$13'), '$E$5')})`, `=IF($A${row}="","",${lengthInput(q('Inputs', '$B$14'), '$E$5')})`,
    `=IF($A${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($A${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($A${row}="","",0)`,
    `=IF($A${row}="","",IF(OR(C${row}<0,D${row}<0,D${row}>PI()),"INVALID","OK"))`,
  ];
  return [
    `=${active}`, `=IF($A${row}="","",${q('Plan', `A${row}`)})`, `=IF($A${row}="","",${lengthInput(q('Plan', `B${row}`), '$E$5')})`,
    `=IF($A${row}="","",${angleInput(q('Plan', `C${row}`), '$E$6')})`, `=IF($A${row}="","",MOD(${angleInput(q('Plan', `D${row}`), '$E$6')},2*PI()))`,
    `=IF($A${row}="","",C${row}-C${p})`,
    `=IF($A${row}="","",ACOS(MAX(-1,MIN(1,COS(D${p})*COS(D${row})+SIN(D${p})*SIN(D${row})*COS(E${row}-E${p})))))`,
    `=IF($A${row}="","",IF(ABS(G${row})<1E-9,1+G${row}^2/12+G${row}^4/120,2*TAN(G${row}/2)/G${row}))`,
    `=IF($A${row}="","",F${row}/2*(COS(D${p})+COS(D${row}))*H${row})`,
    `=IF($A${row}="","",F${row}/2*(SIN(D${p})*COS(E${p})+SIN(D${row})*COS(E${row}))*H${row})`,
    `=IF($A${row}="","",F${row}/2*(SIN(D${p})*SIN(E${p})+SIN(D${row})*SIN(E${row}))*H${row})`,
    `=IF($A${row}="","",L${p}+I${row})`, `=IF($A${row}="","",M${p}+J${row})`, `=IF($A${row}="","",N${p}+K${row})`,
    `=IF($A${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($A${row}="","","${PRESENTATION_ONLY_STATE}")`,
    `=IF($A${row}="","",IF(F${row}>0,G${row}/F${row},""))`,
    `=IF($A${row}="","",IF(OR(F${row}<=0,D${row}<0,D${row}>PI()),"INVALID",IF(${q('Plan', `D${row}`)}<>MOD(${q('Plan', `D${row}`)},360),"OK - AZI NORMALIZED","OK")))`,
  ];
}

function canonicalSurveyRow(row) {
  const p = row - 1;
  const active = `IF(COUNT(${q('Survey', `B${row}`)})=0,"",TRUE)`;
  if (row === START) return [
    `=${active}`, `=IF($T${row}="","",${q('Survey', `A${row}`)})`, `=IF($T${row}="","",${lengthInput(q('Survey', `B${row}`), '$E$7')})`,
    `=IF($T${row}="","",${angleInput(q('Survey', `C${row}`), '$E$8')})`, `=IF($T${row}="","",MOD(${angleInput(q('Survey', `D${row}`), '$E$8')},2*PI()))`,
    `=IF($T${row}="","",0)`, `=IF($T${row}="","",0)`, `=IF($T${row}="","",1)`, `=IF($T${row}="","",0)`, `=IF($T${row}="","",0)`, `=IF($T${row}="","",0)`,
    `=IF($T${row}="","",0)`, `=IF($T${row}="","",${lengthInput(q('Inputs', '$B$13'), '$E$7')})`, `=IF($T${row}="","",${lengthInput(q('Inputs', '$B$14'), '$E$7')})`,
    `=IF($T${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($T${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($T${row}="","",0)`,
    `=IF($T${row}="","",IF(OR(V${row}<0,W${row}<0,W${row}>PI()),"INVALID","OK"))`,
  ];
  return [
    `=${active}`, `=IF($T${row}="","",${q('Survey', `A${row}`)})`, `=IF($T${row}="","",${lengthInput(q('Survey', `B${row}`), '$E$7')})`,
    `=IF($T${row}="","",${angleInput(q('Survey', `C${row}`), '$E$8')})`, `=IF($T${row}="","",MOD(${angleInput(q('Survey', `D${row}`), '$E$8')},2*PI()))`,
    `=IF($T${row}="","",V${row}-V${p})`,
    `=IF($T${row}="","",ACOS(MAX(-1,MIN(1,COS(W${p})*COS(W${row})+SIN(W${p})*SIN(W${row})*COS(X${row}-X${p})))))`,
    `=IF($T${row}="","",IF(ABS(Z${row})<1E-9,1+Z${row}^2/12+Z${row}^4/120,2*TAN(Z${row}/2)/Z${row}))`,
    `=IF($T${row}="","",Y${row}/2*(COS(W${p})+COS(W${row}))*AA${row})`,
    `=IF($T${row}="","",Y${row}/2*(SIN(W${p})*COS(X${p})+SIN(W${row})*COS(X${row}))*AA${row})`,
    `=IF($T${row}="","",Y${row}/2*(SIN(W${p})*SIN(X${p})+SIN(W${row})*SIN(X${row}))*AA${row})`,
    `=IF($T${row}="","",AE${p}+AB${row})`, `=IF($T${row}="","",AF${p}+AC${row})`, `=IF($T${row}="","",AG${p}+AD${row})`,
    `=IF($T${row}="","","${PRESENTATION_ONLY_STATE}")`, `=IF($T${row}="","","${PRESENTATION_ONLY_STATE}")`,
    `=IF($T${row}="","",IF(Y${row}>0,Z${row}/Y${row},""))`,
    `=IF($T${row}="","",IF(OR(Y${row}<=0,W${row}<0,W${row}>PI()),"INVALID",IF(${q('Survey', `D${row}`)}<>MOD(${q('Survey', `D${row}`)},360),"OK - AZI NORMALIZED","OK")))`,
  ];
}

function interpolationRow(row) {
  const ok = `AL${row}="OK"`;
  const lower = `AM${row}`;
  const i = `(${lower}-6)`;
  const j = `(${lower}-5)`;
  const idx = (col, which = i) => `INDEX($${col}$7:$${col}$506,${which})`;
  const blend = (a, b) => `((1-AN${row})*${a}+AN${row}*${b})`;
  const norm = `SQRT(${blend(`AP${row}`, `AS${row}`)}^2+${blend(`AQ${row}`, `AT${row}`)}^2+${blend(`AR${row}`, `AU${row}`)}^2)`;
  return [
    `=IF(T${row}="","",IF(V${row}<MIN($C$7:$C$506),"BEFORE START",IF(V${row}>MAX($C$7:$C$506),"BEYOND TD","OK")))`,
    `=IF(NOT(${ok}),"",IF(V${row}>=MAX($C$7:$C$506),MATCH(MAX($C$7:$C$506),$C$7:$C$506,0)+5,MATCH(V${row},$C$7:$C$506,1)+6))`,
    `=IF(NOT(${ok}),"",(V${row}-${idx('C')})/(${idx('C', j)}-${idx('C')}))`,
    `=IF(NOT(${ok}),"",ACOS(MAX(-1,MIN(1,COS(${idx('D')})*COS(${idx('D', j)})+SIN(${idx('D')})*SIN(${idx('D', j)})*COS(${idx('E', j)}-${idx('E')})))))`,
    `=IF(NOT(${ok}),"",SIN(${idx('D')})*COS(${idx('E')}))`, `=IF(NOT(${ok}),"",SIN(${idx('D')})*SIN(${idx('E')}))`, `=IF(NOT(${ok}),"",COS(${idx('D')}))`,
    `=IF(NOT(${ok}),"",SIN(${idx('D', j)})*COS(${idx('E', j)}))`, `=IF(NOT(${ok}),"",SIN(${idx('D', j)})*SIN(${idx('E', j)}))`, `=IF(NOT(${ok}),"",COS(${idx('D', j)}))`,
    `=IF(NOT(${ok}),"",IF(ABS(AO${row})<1E-9,${blend(`AP${row}`, `AS${row}`)}/${norm},SIN((1-AN${row})*AO${row})/SIN(AO${row})*AP${row}+SIN(AN${row}*AO${row})/SIN(AO${row})*AS${row}))`,
    `=IF(NOT(${ok}),"",IF(ABS(AO${row})<1E-9,${blend(`AQ${row}`, `AT${row}`)}/${norm},SIN((1-AN${row})*AO${row})/SIN(AO${row})*AQ${row}+SIN(AN${row}*AO${row})/SIN(AO${row})*AT${row}))`,
    `=IF(NOT(${ok}),"",IF(ABS(AO${row})<1E-9,${blend(`AR${row}`, `AU${row}`)}/${norm},SIN((1-AN${row})*AO${row})/SIN(AO${row})*AR${row}+SIN(AN${row}*AO${row})/SIN(AO${row})*AU${row}))`,
    `=IF(NOT(${ok}),"",ACOS(MAX(-1,MIN(1,AX${row}))))`,
    // Azimuth is undefined for a vertical direction vector. Preserve the
    // lower-station azimuth in that limit so exact/partial positions stay
    // finite and the horizontal displacement remains zero.
    `=IF(NOT(${ok}),"",MOD(ATAN2(IF(SQRT(AV${row}^2+AW${row}^2)<1E-9,COS(${idx('E')}),AV${row}),IF(SQRT(AV${row}^2+AW${row}^2)<1E-9,SIN(${idx('E')}),AW${row})),2*PI()))`,
    `=IF(NOT(${ok}),"",V${row}-${idx('C')})`, `=IF(NOT(${ok}),"",AN${row}*AO${row})`,
    `=IF(NOT(${ok}),"",IF(ABS(BB${row})<1E-9,1+BB${row}^2/12+BB${row}^4/120,2*TAN(BB${row}/2)/BB${row}))`,
    `=IF(NOT(${ok}),"",BA${row}/2*(COS(${idx('D')})+COS(AY${row}))*BC${row})`,
    `=IF(NOT(${ok}),"",BA${row}/2*(SIN(${idx('D')})*COS(${idx('E')})+SIN(AY${row})*COS(AZ${row}))*BC${row})`,
    `=IF(NOT(${ok}),"",BA${row}/2*(SIN(${idx('D')})*SIN(${idx('E')})+SIN(AY${row})*SIN(AZ${row}))*BC${row})`,
    `=IF(NOT(${ok}),"",${idx('L')}+BD${row})`, `=IF(NOT(${ok}),"",${idx('M')}+BE${row})`, `=IF(NOT(${ok}),"",${idx('N')}+BF${row})`,
    `=IF(NOT(${ok}),"","${PRESENTATION_ONLY_STATE}")`, `=IF(NOT(${ok}),"","${PRESENTATION_ONLY_STATE}")`,
    `=IF(NOT(${ok}),"",AE${row}-BG${row})`, `=IF(NOT(${ok}),"",AF${row}-BH${row})`, `=IF(NOT(${ok}),"",AG${row}-BI${row})`, `=IF(NOT(${ok}),"","${RUST_REQUIRED_STATE}")`,
    `=IF(NOT(${ok}),"",BM${row}*COS(${vsa})+BN${row}*SIN(${vsa}))`, `=IF(NOT(${ok}),"",-BM${row}*SIN(${vsa})+BN${row}*COS(${vsa}))`,
    `=IF(NOT(${ok}),"",SQRT(BM${row}^2+BN${row}^2))`, `=IF(NOT(${ok}),"",SQRT(BM${row}^2+BN${row}^2+BL${row}^2))`,
  ];
}

export function buildCanonicalModel(sheets) {
  const calc = sheets.Calc;
  calc.getRange('A3:BS3').unmerge();
  sectionHeader(calc, 'A3:BS3', 'Canonical SI trajectory, exact plan-at-survey interpolation, diagnostics, and helper calculations');
  calc.getRange('A6:R6').values = [PLAN_HEADERS];
  calc.getRange('T6:AK6').values = [SURVEY_HEADERS];
  calc.getRange('AL6:BS6').values = [[
    'Plan Coverage', 'Lower Plan Row', 'Fraction', 'Total Dogleg', 'u1 North', 'u1 East', 'u1 Vertical', 'u2 North', 'u2 East', 'u2 Vertical',
    'SLERP North', 'SLERP East', 'SLERP Vertical', 'Partial Inc', 'Partial Azi', 'Partial MD', 'Partial Dogleg', 'Partial RF', 'Partial dTVD', 'Partial dN', 'Partial dE',
    'Plan-at-MD TVD', 'Plan-at-MD North', 'Plan-at-MD East', 'Plan-at-MD VS State', 'Plan-at-MD Crossline State', 'dTVD', 'dNorth', 'dEast', 'dVS', 'Along-track', 'Crossline Error', 'Horizontal Error', '3D Error',
  ]];
  calc.getRange(`A${START}:R${END}`).formulas = Array.from({ length: END - START + 1 }, (_, i) => canonicalPlanRow(START + i));
  calc.getRange(`T${START}:AK${END}`).formulas = Array.from({ length: END - START + 1 }, (_, i) => canonicalSurveyRow(START + i));
  calc.getRange(`AL${START}:BS${END}`).formulas = Array.from({ length: END - START + 1 }, (_, i) => interpolationRow(START + i));
  for (const range of ['A6:R6', 'T6:AK6', 'AL6:BS6']) range && (calc.getRange(range).format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, wrapText: true });
  calc.getRange('C7:Q506').format.numberFormat = '0.000000';
  calc.getRange('V7:AJ506').format.numberFormat = '0.000000';
}

function setFormulas(sheet, address, matrix) { sheet.getRange(address).formulas = matrix; }

export function linkVisibleTables(sheets) {
  const planRows = Array.from({ length: 500 }, (_, i) => {
    const r = START + i;
    const planVs = `(${q('Calc', `M${r}`)}*COS(${vsa})+${q('Calc', `N${r}`)}*SIN(${vsa}))`;
    const planCrossline = `(-${q('Calc', `M${r}`)}*SIN(${vsa})+${q('Calc', `N${r}`)}*COS(${vsa}))`;
    return [`=IF(${q('Calc', `A${r}`)}="","",${q('Calc', `A${r}`)})`, `=IF(${q('Calc', `A${r}`)}="","",${displayLength(q('Calc', `L${r}`))})`, `=IF(${q('Calc', `A${r}`)}="","",${displayLength(q('Calc', `M${r}`))})`, `=IF(${q('Calc', `A${r}`)}="","",${displayLength(q('Calc', `N${r}`))})`, `=IF(${q('Calc', `A${r}`)}="","",${displayLength(planVs)})`, `=IF(${q('Calc', `A${r}`)}="","",${displayLength(planCrossline)})`, `=IF(${q('Calc', `A${r}`)}="","",${displayGradient(q('Calc', `Q${r}`))})`, `=IF(${q('Calc', `A${r}`)}="","",${q('Calc', `R${r}`)})`];
  });
  setFormulas(sheets.Plan, 'F7:M506', planRows);
  const surveyRows = Array.from({ length: 500 }, (_, i) => {
    const r = START + i;
    const c = (cell) => q('Calc', `${cell}${r}`);
    const surveyVs = `(${c('AF')}*COS(${vsa})+${c('AG')}*SIN(${vsa}))`;
    const surveyCrossline = `(-${c('AF')}*SIN(${vsa})+${c('AG')}*COS(${vsa}))`;
    const residualVs = `(${c('BM')}*COS(${vsa})+${c('BN')}*SIN(${vsa}))`;
    const positionValues = [c('AE'), c('AF'), c('AG'), surveyVs, surveyCrossline, ...['BG', 'BH', 'BI', 'BL', 'BM', 'BN'].map(c), residualVs, ...['BP', 'BQ', 'BR', 'BS'].map(c)];
    return [`=IF(${c('T')}="","",${c('T')})`, ...positionValues.map((value) => `=IF(${c('T')}="","",IF(COUNT(${value})=0,"",${displayLength(value)}))`), `=IF(${c('T')}="","",${c('AL')})`, `=IF(${c('T')}="","",${displayGradient(c('AJ'))})`, `=IF(${c('T')}="","",${c('AK')})`];
  });
  setFormulas(sheets.Survey, 'F7:Y506', surveyRows);
  sheets.Plan.getRange('B5:D5').formulas = [[`=${q('Inputs', '$E$5')}`, `=${q('Inputs', '$E$6')}`, `=${q('Inputs', '$E$6')}`]];
  sheets.Plan.getRange('G5:L5').formulas = [[`=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$20')}`]];
  sheets.Survey.getRange('B5:D5').formulas = [[`=${q('Inputs', '$E$7')}`, `=${q('Inputs', '$E$8')}`, `=${q('Inputs', '$E$8')}`]];
  sheets.Survey.getRange('G5:X5').formulas = [[...Array(16).fill(`=${q('Unit Map', '$H$8')}`), '', `=${q('Unit Map', '$H$20')}`]];
}

const targetRawToSi = (cell, row) => lengthInput(q('Targets', `${cell}${row}`), '$E$9');

function actualInterpolationRow(row) {
  const ok = `FK${row}="OK"`;
  const lower = `FL${row}`;
  const i = `(${lower}-6)`;
  const j = `(${lower}-5)`;
  const idx = (col, which = i) => `INDEX($${col}$7:$${col}$506,${which})`;
  const blend = (a, b) => `((1-FM${row})*${a}+FM${row}*${b})`;
  const norm = `SQRT(${blend(`FO${row}`, `FR${row}`)}^2+${blend(`FP${row}`, `FS${row}`)}^2+${blend(`FQ${row}`, `FT${row}`)}^2)`;
  const horizontal = `SQRT(FU${row}^2+FV${row}^2)`;
  return [
    `=IF(COUNT(FJ${row})=0,"",IF(FJ${row}<MIN($V$7:$V$506),"BEFORE START",IF(FJ${row}>MAX($V$7:$V$506),"BEYOND TD","OK")))`,
    `=IF(NOT(${ok}),"",IF(FJ${row}>=MAX($V$7:$V$506),MATCH(MAX($V$7:$V$506),$V$7:$V$506,0)+5,MATCH(FJ${row},$V$7:$V$506,1)+6))`,
    `=IF(NOT(${ok}),"",IF(ABS(${idx('V', j)}-${idx('V')})<1E-9,"",(FJ${row}-${idx('V')})/(${idx('V', j)}-${idx('V')})))`,
    `=IF(NOT(${ok}),"",ACOS(MAX(-1,MIN(1,COS(${idx('W')})*COS(${idx('W', j)})+SIN(${idx('W')})*SIN(${idx('W', j)})*COS(${idx('X', j)}-${idx('X')})))))`,
    `=IF(NOT(${ok}),"",SIN(${idx('W')})*COS(${idx('X')}))`, `=IF(NOT(${ok}),"",SIN(${idx('W')})*SIN(${idx('X')}))`, `=IF(NOT(${ok}),"",COS(${idx('W')}))`,
    `=IF(NOT(${ok}),"",SIN(${idx('W', j)})*COS(${idx('X', j)}))`, `=IF(NOT(${ok}),"",SIN(${idx('W', j)})*SIN(${idx('X', j)}))`, `=IF(NOT(${ok}),"",COS(${idx('W', j)}))`,
    `=IF(NOT(${ok}),"",IF(ABS(FN${row})<1E-9,${blend(`FO${row}`, `FR${row}`)}/${norm},SIN((1-FM${row})*FN${row})/SIN(FN${row})*FO${row}+SIN(FM${row}*FN${row})/SIN(FN${row})*FR${row}))`,
    `=IF(NOT(${ok}),"",IF(ABS(FN${row})<1E-9,${blend(`FP${row}`, `FS${row}`)}/${norm},SIN((1-FM${row})*FN${row})/SIN(FN${row})*FP${row}+SIN(FM${row}*FN${row})/SIN(FN${row})*FS${row}))`,
    `=IF(NOT(${ok}),"",IF(ABS(FN${row})<1E-9,${blend(`FQ${row}`, `FT${row}`)}/${norm},SIN((1-FM${row})*FN${row})/SIN(FN${row})*FQ${row}+SIN(FM${row}*FN${row})/SIN(FN${row})*FT${row}))`,
    `=IF(NOT(${ok}),"",ACOS(MAX(-1,MIN(1,FW${row}))))`,
    `=IF(NOT(${ok}),"",MOD(ATAN2(IF(${horizontal}<1E-9,COS(${idx('X')}),FU${row}),IF(${horizontal}<1E-9,SIN(${idx('X')}),FV${row})),2*PI()))`,
    `=IF(NOT(${ok}),"",FJ${row}-${idx('V')})`, `=IF(NOT(${ok}),"",FM${row}*FN${row})`,
    `=IF(NOT(${ok}),"",IF(ABS(GA${row})<1E-9,1+GA${row}^2/12+GA${row}^4/120,2*TAN(GA${row}/2)/GA${row}))`,
    `=IF(NOT(${ok}),"",FZ${row}/2*(COS(${idx('W')})+COS(FX${row}))*GB${row})`,
    `=IF(NOT(${ok}),"",FZ${row}/2*(SIN(${idx('W')})*COS(${idx('X')})+SIN(FX${row})*COS(FY${row}))*GB${row})`,
    `=IF(NOT(${ok}),"",FZ${row}/2*(SIN(${idx('W')})*SIN(${idx('X')})+SIN(FX${row})*SIN(FY${row}))*GB${row})`,
    `=IF(NOT(${ok}),"",${idx('AE')}+GC${row})`, `=IF(NOT(${ok}),"",${idx('AF')}+GD${row})`, `=IF(NOT(${ok}),"",${idx('AG')}+GE${row})`,
    `=IF(NOT(${ok}),"",GG${row}*COS(${vsa})+GH${row}*SIN(${vsa}))`, `=IF(NOT(${ok}),"",-GG${row}*SIN(${vsa})+GH${row}*COS(${vsa}))`,
  ];
}

function buildActualQueryModel(sheets) {
  const calc = sheets.Calc;
  sectionHeader(calc, 'FI3:GJ3', 'Canonical actual-survey interpolation queries — exact partial minimum curvature');
  calc.getRange('FI6:GJ6').values = [[
    'Query', 'Query MD m', 'Coverage', 'Lower Survey Row', 'Fraction', 'Total Dogleg', 'u1 North', 'u1 East', 'u1 Vertical', 'u2 North', 'u2 East', 'u2 Vertical',
    'SLERP North', 'SLERP East', 'SLERP Vertical', 'Inc rad', 'Azi rad', 'Partial MD m', 'Partial Dogleg', 'Partial RF', 'dTVD m', 'dN m', 'dE m', 'TVD m', 'North m', 'East m', 'VS m', 'Crossline m',
  ]];
  const targetQueries = Array.from({ length: 100 }, (_, index) => {
    const r = START + index;
    return [`=IF(${q('Targets', `A${r}`)}="","",${q('Targets', `A${r}`)})`, `=IF(COUNT(${q('Targets', `B${r}`)})=0,"",${targetRawToSi('B', r)})`];
  });
  const formationQueries = Array.from({ length: 100 }, (_, index) => {
    const r = START + index;
    return [`=IF(${q('Formation Tops', `A${r}`)}="","",${q('Formation Tops', `A${r}`)})`, `=IF(COUNT(${q('Formation Tops', `D${r}`)})=0,"",${lengthInput(q('Formation Tops', `D${r}`), '$E$11')})`];
  });
  const slideQueries = Array.from({ length: 400 }, (_, index) => {
    const r = START + Math.floor(index / 2);
    const isIn = index % 2 === 0;
    const rawCell = isIn ? 'C' : 'D';
    const suffix = isIn ? ' In' : ' Out';
    return [`=IF(${q('Slide Performance', `A${r}`)}="","","Slide "&${q('Slide Performance', `A${r}`)}&"${suffix}")`, `=IF(COUNT(${q('Slide Performance', `${rawCell}${r}`)})=0,"",${lengthInput(q('Slide Performance', `${rawCell}${r}`), '$E$10')})`];
  });
  calc.getRange('FI7:FJ106').formulas = targetQueries;
  calc.getRange('FI110:FJ209').formulas = formationQueries;
  calc.getRange('FI213:FJ612').formulas = slideQueries;
  for (const [first, last] of [[7, 106], [110, 209], [213, 612]]) calc.getRange(`FK${first}:GJ${last}`).formulas = Array.from({ length: last - first + 1 }, (_, index) => actualInterpolationRow(first + index));
  calc.getRange('FI6:GJ6').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, wrapText: true };
  calc.getRange('FJ7:GJ612').format.numberFormat = '0.000000';
}

function buildProjectionModel(sheets) {
  const calc = sheets.Calc;
  sectionHeader(calc, 'GL3:GM3', 'Deterministic projection — latest valid survey to bit and ahead');
  const labels = [
    'Latest survey row', 'Latest survey MD m', 'Latest Inc rad', 'Latest Azi rad', 'Latest TVD m', 'Latest North m', 'Latest East m',
    'Bit MD m', 'Ahead m', 'Build tendency rad/m', 'Effective-turn tendency rad/m', 'Low-inc threshold rad', 'dMD to bit m',
    'Bit Inc rad', 'Bit average Inc rad', 'Bit dAzi rad', 'Bit Azi rad', 'Bit dogleg rad', 'Bit RF', 'Bit dTVD m', 'Bit dN m', 'Bit dE m', 'Bit TVD m', 'Bit North m', 'Bit East m',
    'Projected end MD m', 'End Inc rad', 'End average Inc rad', 'End dAzi rad', 'End Azi rad', 'End dogleg rad', 'End RF', 'End dTVD m', 'End dN m', 'End dE m', 'End TVD m', 'End North m', 'End East m',
    'Projection confidence', 'Projection coverage',
  ];
  calc.getRange('GL6:GL45').values = labels.map((label) => [label]);
  calc.getRange('GM6:GM45').formulas = [
    [`=MATCH(MAX($V$7:$V$506),$V$7:$V$506,0)+6`], [`=INDEX($V$7:$V$506,GM6-6)`], [`=INDEX($W$7:$W$506,GM6-6)`], [`=INDEX($X$7:$X$506,GM6-6)`],
    [`=INDEX($AE$7:$AE$506,GM6-6)`], [`=INDEX($AF$7:$AF$506,GM6-6)`], [`=INDEX($AG$7:$AG$506,GM6-6)`],
    [`=IF(COUNT(${q('Inputs', '$K$5')})=0,"",${lengthInput(q('Inputs', '$K$5'), '$E$7')})`], [`=IF(COUNT(${q('Inputs', '$K$6')})=0,0,${lengthInput(q('Inputs', '$K$6'), '$E$7')})`],
    [`=${gradientInput(q('Inputs', '$K$7'), '$K$9')}`], [`=${gradientInput(q('Inputs', '$K$8'), '$K$9')}`], [`=${q('Inputs', '$N$5')}/${q('Unit Map', '$E$18')}`],
    ['=IF(GM13<GM7,"",GM13-GM7)'], ['=IF(COUNT(GM18)=0,"",MAX(0,MIN(PI(),GM8+GM15*GM18)))'], ['=IF(COUNT(GM19)=0,"",(GM8+GM19)/2)'],
    ['=IF(COUNT(GM18)=0,"",GM16*GM18/MAX(SIN(GM20),SIN(GM17),1E-9))'], ['=IF(COUNT(GM18)=0,"",MOD(GM9+GM21,2*PI()))'],
    ['=IF(COUNT(GM18)=0,"",ACOS(MAX(-1,MIN(1,COS(GM8)*COS(GM19)+SIN(GM8)*SIN(GM19)*COS(GM22-GM9)))))'],
    ['=IF(COUNT(GM18)=0,"",IF(ABS(GM23)<1E-9,1+GM23^2/12+GM23^4/120,2*TAN(GM23/2)/GM23))'],
    ['=IF(COUNT(GM18)=0,"",GM18/2*(COS(GM8)+COS(GM19))*GM24)'], ['=IF(COUNT(GM18)=0,"",GM18/2*(SIN(GM8)*COS(GM9)+SIN(GM19)*COS(GM22))*GM24)'], ['=IF(COUNT(GM18)=0,"",GM18/2*(SIN(GM8)*SIN(GM9)+SIN(GM19)*SIN(GM22))*GM24)'],
    ['=IF(COUNT(GM18)=0,"",GM10+GM25)'], ['=IF(COUNT(GM18)=0,"",GM11+GM26)'], ['=IF(COUNT(GM18)=0,"",GM12+GM27)'],
    ['=IF(COUNT(GM13)=0,"",GM13+GM14)'], ['=IF(COUNT(GM31)=0,"",MAX(0,MIN(PI(),GM19+GM15*GM14)))'], ['=IF(COUNT(GM32)=0,"",(GM19+GM32)/2)'],
    ['=IF(COUNT(GM31)=0,"",GM16*GM14/MAX(SIN(GM33),SIN(GM17),1E-9))'], ['=IF(COUNT(GM31)=0,"",MOD(GM22+GM34,2*PI()))'],
    ['=IF(COUNT(GM31)=0,"",ACOS(MAX(-1,MIN(1,COS(GM19)*COS(GM32)+SIN(GM19)*SIN(GM32)*COS(GM35-GM22)))))'],
    ['=IF(COUNT(GM31)=0,"",IF(ABS(GM36)<1E-9,1+GM36^2/12+GM36^4/120,2*TAN(GM36/2)/GM36))'],
    ['=IF(COUNT(GM31)=0,"",GM14/2*(COS(GM19)+COS(GM32))*GM37)'], ['=IF(COUNT(GM31)=0,"",GM14/2*(SIN(GM19)*COS(GM22)+SIN(GM32)*COS(GM35))*GM37)'], ['=IF(COUNT(GM31)=0,"",GM14/2*(SIN(GM19)*SIN(GM22)+SIN(GM32)*SIN(GM35))*GM37)'],
    ['=IF(COUNT(GM31)=0,"",GM28+GM38)'], ['=IF(COUNT(GM31)=0,"",GM29+GM39)'], ['=IF(COUNT(GM31)=0,"",GM30+GM40)'],
    ['=IF(GM45<>"DETERMINISTIC","INVALID",IF(AND(ABS(GM16)>0,OR(SIN(GM20)<=SIN(GM17),SIN(GM33)<=SIN(GM17))),"LOW - TURN GUARDED","NORMAL"))'],
    ['=IF(COUNT(GM13)=0,"INVALID - BIT MD",IF(GM13<GM7,"INVALID - BIT BEHIND SURVEY",IF(GM14<0,"INVALID - AHEAD","DETERMINISTIC")))'],
  ];
  calc.getRange('GL6:GM45').format.borders = { preset: 'outside', style: 'thin', color: COLORS.line };
  calc.getRange('GM7:GM43').format.numberFormat = '0.000000';
}

function buildTargetModel(sheets) {
  const calc = sheets.Calc;
  const target = sheets.Targets;
  sectionHeader(calc, 'HB3:HT3', 'Canonical Rust target bridge fields and typed unavailable legacy slots');
  calc.getRange('HB6:HT6').values = [[
    'Target UUID', 'Target MD m', 'Basis', 'Inc State', 'Azi State', 'Dogleg State', 'RF State', 'Position TVD m',
    'Position North m (surface-relative Rust)', 'Position East m (surface-relative Rust)', 'North Difference State',
    'East Difference State', 'Vertical Difference m (Rust)', 'Local Major m', 'Local Minor m',
    'Horizontal Utilization', 'Vertical Utilization', 'Rust Evaluation Status', 'Display Status',
  ]];
  calc.getRange('HB7:HT106').formulas = Array.from({ length: 100 }, (_, index) => {
    const r = START + index;
    const active = q('Targets', `A${r}`);
    const whenActive = (value) => `=IF(${active}="","",${value})`;
    return [
      whenActive(`$JC${r}`), whenActive(targetRawToSi('B', r)), whenActive(`"${RUST_REQUIRED_STATE}"`),
      whenActive(`"${RUST_UNAVAILABLE_STATE}"`), whenActive(`"${RUST_UNAVAILABLE_STATE}"`), whenActive(`"${RUST_UNAVAILABLE_STATE}"`), whenActive(`"${RUST_UNAVAILABLE_STATE}"`),
      whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`),
      whenActive(`"${RUST_UNAVAILABLE_STATE}"`), whenActive(`"${RUST_UNAVAILABLE_STATE}"`),
      whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`),
      whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`), whenActive(`"${RUST_REQUIRED_STATE}"`),
    ];
  });
  calc.getRange('HB6:HT6').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, wrapText: true };
  target.getRange('L7:Q106').formulas = Array.from({ length: 100 }, (_, index) => {
    const r = START + index;
    return [`=IF(A${r}="","",${q('Calc', `$HD${r}`)})`, `=IF(A${r}="","",IF(COUNT(${q('Calc', `$HO${r}`)})=0,"",${displayLength(q('Calc', `$HO${r}`))}))`, `=IF(A${r}="","",IF(COUNT(${q('Calc', `$HP${r}`)})=0,"",${displayLength(q('Calc', `$HP${r}`))}))`, `=IF(A${r}="","",IF(COUNT(${q('Calc', `$HQ${r}`)})=0,"",${q('Calc', `$HQ${r}`)}))`, `=IF(A${r}="","",IF(COUNT(${q('Calc', `$HR${r}`)})=0,"",${q('Calc', `$HR${r}`)}))`, `=IF(A${r}="","",${q('Calc', `$HT${r}`)})`];
  });
}

function buildSlideModel(sheets) {
  const slide = sheets['Slide Performance'];
  slide.getRange('C5:I5').formulas = [[`=${q('Inputs', '$E$10')}`, `=${q('Inputs', '$E$10')}`, `=${q('Inputs', '$E$10')}`, `=${q('Inputs', '$E$10')}`, `=${q('Inputs', '$E$8')}`, `=${q('Inputs', '$K$9')}`, `=${q('Inputs', '$K$9')}`]];
  slide.getRange('K5:R5').formulas = [[...Array(5).fill(`=${q('Unit Map', '$H$20')}`), `=${q('Unit Map', '$H$18')}`, `=${q('Unit Map', '$H$18')}`, `=${q('Unit Map', '$H$20')}`]];
  slide.getRange('K7:S206').formulas = Array.from({ length: 200 }, (_, index) => {
    const r = START + index;
    const qIn = QUERY_SLIDE_START + index * 2;
    const qOut = qIn + 1;
    const course = `(${q('Calc', `$FJ${qOut}`)}-${q('Calc', `$FJ${qIn}`)})`;
    const slideLen = lengthInput(q('Slide Performance', `E${r}`), '$E$10');
    const incIn = q('Calc', `$FX${qIn}`); const incOut = q('Calc', `$FX${qOut}`); const aziIn = q('Calc', `$FY${qIn}`); const aziOut = q('Calc', `$FY${qOut}`);
    const rotaryBuild = `IF(H${r}="",0,${gradientInput(`H${r}`, '$K$9')})`; const rotaryTurn = `IF(I${r}="",0,${gradientInput(`I${r}`, '$K$9')})`;
    const norm = `SQRT(M${r}^2+N${r}^2)`;
    const rollingNumerator = `SUMPRODUCT(($A$7:$A$206<=A${r})*($A$7:$A$206>A${r}-${q('Inputs', '$N$8')})*($S$7:$S$206="OK")*$E$7:$E$206*$O$7:$O$206)`;
    const rollingDenominator = `SUMPRODUCT(($A$7:$A$206<=A${r})*($A$7:$A$206>A${r}-${q('Inputs', '$N$8')})*($S$7:$S$206="OK")*$E$7:$E$206)`;
    return [
      `=IF(A${r}="","",IF(OR(${q('Calc', `$FK${qIn}`)}<>"OK",${q('Calc', `$FK${qOut}`)}<>"OK"),"",(${incOut}-${incIn})/${course}*${q('Unit Map', '$I$20')}))`,
      `=IF(A${r}="","",IF(OR(${q('Calc', `$FK${qIn}`)}<>"OK",${q('Calc', `$FK${qOut}`)}<>"OK"),"",(MOD(${aziOut}-${aziIn}+PI(),2*PI())-PI())*SIN((${incIn}+${incOut})/2)/${course}*${q('Unit Map', '$I$20')}))`,
      `=IF(OR(A${r}="",K${r}=""),"",((K${r}/${q('Unit Map', '$I$20')})-${rotaryBuild})*${course}/${slideLen}*${q('Unit Map', '$I$20')})`,
      `=IF(OR(A${r}="",L${r}=""),"",((L${r}/${q('Unit Map', '$I$20')})-${rotaryTurn})*${course}/${slideLen}*${q('Unit Map', '$I$20')})`,
      `=IF(OR(M${r}="",N${r}=""),"",${norm})`,
      `=IF(O${r}="","",IF(${norm}<1E-9,"",MOD(ATAN2(IF(${norm}<1E-9,1,M${r}),IF(${norm}<1E-9,0,N${r})),2*PI())*${q('Unit Map', '$I$18')}))`,
      `=IF(P${r}="","",(MOD((P${r}/${q('Unit Map', '$I$18')})-${angleInput(q('Slide Performance', `G${r}`), '$E$8')}+PI(),2*PI())-PI())*${q('Unit Map', '$I$18')})`,
      `=IF(A${r}="","",IFERROR(${rollingNumerator}/${rollingDenominator},""))`,
      `=IF(A${r}="","",IF(OR(${q('Calc', `$FK${qIn}`)}<>"OK",${q('Calc', `$FK${qOut}`)}<>"OK"),"OUTSIDE SURVEY",IF(${course}<=0,"INVALID INTERVAL",IF(${slideLen}<=0,"INVALID SLIDE LENGTH",IF(((${incIn}+${incOut})/2)<${q('Inputs', '$N$5')}/${q('Unit Map', '$E$18')},"LOW INCLINATION",IF(${slideLen}<${lengthInput(q('Inputs', '$N$6'), '$E$10')},"SHORT SLIDE",IF((O${r}/${q('Unit Map', '$I$20')})>${gradientInput(q('Inputs', '$N$7'), '$K$9')},"OUTLIER","OK")))))))`,
    ];
  });
}

function buildFormationModel(sheets) {
  const tops = sheets['Formation Tops'];
  tops.getRange('B5:E5').formulas = [[`=${q('Inputs', '$E$11')}`, `=${q('Inputs', '$E$11')}`, `=${q('Inputs', '$E$11')}`, `=${q('Inputs', '$E$11')}`]];
  tops.getRange('G5:H5').formulas = [[`=${q('Unit Map', '$H$8')}`, `=${q('Unit Map', '$H$8')}`]];
  tops.getRange('G7:K106').formulas = Array.from({ length: 100 }, (_, index) => {
    const r = START + index;
    const qr = QUERY_FORMATION_START + index;
    const prog = lengthInput(q('Formation Tops', `C${r}`), '$E$11');
    const tolerance = lengthInput(q('Formation Tops', `E${r}`), '$E$11');
    const actualTvd = q('Calc', `$GF${qr}`);
    const coverage = q('Calc', `$FK${qr}`);
    return [
      `=IF(COUNT(D${r})=0,"",IF(${coverage}="OK",${displayLength(actualTvd)},""))`,
      `=IF(G${r}="","",${displayLength(`(${prog}-${actualTvd})`)})`,
      `=IF(H${r}="","",IF(H${r}>0,"HIGH",IF(H${r}<0,"LOW","ON PROGNOSIS")))`,
      `=IF(COUNT(D${r})=0,"",${coverage})`,
      `=IF(COUNT(D${r})=0,"",IF(J${r}<>"OK",J${r},IF(OR(COUNT(E${r})=0,ABS(${prog}-${actualTvd})<=${tolerance}),"OK","OUTSIDE TOLERANCE")))`,
    ];
  });
}

export function buildTargetSlideFormation(sheets) {
  buildActualQueryModel(sheets);
  buildProjectionModel(sheets);
  buildTargetModel(sheets);
  buildSlideModel(sheets);
  buildFormationModel(sheets);
}

export function buildDecisionSurfaces(sheets) {
  const results = sheets.Results; const summary = sheets.Summary; const checks = sheets.Checks;
  sectionHeader(results, 'A3:L3', 'Directional results, terminal comparison, projection summary, and Survey Contract');
  results.getRange('A5:B15').values = [['Metric', 'Value'], ['Latest valid survey MD', null], ['Latest covered crossline error', null], ['Latest covered 3D error', null], ['Maximum actual DLS', null], ['Plan coverage', null], ['Next target status', null], ['Slide calibration status', null], ['Formation status', null], ['Projection basis', null], ['Projection confidence', null]];
  results.getRange('B6:B15').formulas = [
    [`=MAX(${q('Calc', '$V$7:$V$506')})`], [`=LOOKUP(2,1/(${q('Calc', '$BQ$7:$BQ$506')}<>""),${q('Calc', '$BQ$7:$BQ$506')})`], [`=LOOKUP(2,1/(${q('Calc', '$BS$7:$BS$506')}<>""),${q('Calc', '$BS$7:$BS$506')})`],
    [`=MAX(${q('Calc', '$AJ$7:$AJ$506')})`], [`=LOOKUP(2,1/(${q('Calc', '$AL$7:$AL$506')}<>""),${q('Calc', '$AL$7:$AL$506')})`], [`=IFERROR(INDEX(${q('Targets', '$Q$7:$Q$106')},MATCH(TRUE,${q('Targets', '$A$7:$A$106')}<>"",0)),"NO TARGET")`],
    [`=IF(COUNTIF(${q('Slide Performance', '$S$7:$S$206')},"<>OK")>0,"REVIEW","OK")`], [`=IF(COUNTA(${q('Formation Tops', '$D$7:$D$106')})=0,"NO ACTUAL PICKS","OK")`], ['="Latest valid survey"'], ['="DETERMINISTIC - NO UNCERTAINTY MODEL"'],
  ];
  results.getRange('A18:B21').values = [['Terminal endpoint comparison', null], ['Crossline error m', null], ['Horizontal error m', null], ['3D error m', null]];
  const dn = `(${q('Calc', '$AF$66')}-${q('Calc', '$M$66')})`; const de = `(${q('Calc', '$AG$66')}-${q('Calc', '$N$66')})`; const dt = `(${q('Calc', '$AE$66')}-${q('Calc', '$L$66')})`;
  results.getRange('B19:B21').formulas = [[`=-${dn}*SIN(${vsa})+${de}*COS(${vsa})`], [`=SQRT(${dn}^2+${de}^2)`], [`=SQRT(${dn}^2+${de}^2+${dt}^2)`]];
  results.getRange('A25:L25').values = [['Station_ID', 'MD_m', 'Inc_rad', 'Azi_rad', 'TVD_m', 'North_m', 'East_m', 'VS_State', 'Crossline_State', 'DLS_rad_per_m', 'Source', 'Row_Status']];
  results.getRange('A26:L525').formulas = Array.from({ length: 500 }, (_, i) => { const r = START + i; return ['U','V','W','X','AE','AF','AG','AH','AI','AJ'].map((c) => `=IF(${q('Calc', `T${r}`)}="","",${q('Calc', `${c}${r}`)})`).concat([`=IF(${q('Calc', `T${r}`)}="","",${q('Survey', `E${r}`)})`, `=IF(${q('Calc', `T${r}`)}="","",${q('Calc', `AK${r}`)})`]); });
  results.getRange('A5:B5').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
  results.getRange('A25:L25').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white }, wrapText: true };
  results.freezePanes.freezeRows(25);
  buildTrajectoryEngineEvidence(results);

  checks.getRange('A3:E3').unmerge(); sectionHeader(checks, 'A3:E3', 'Formula-linked directional checks and required actions');
  checks.getRange('A5:E5').values = [['Check', 'Measured result', 'Status', 'Severity', 'Required action']];
  const rows = [
    ['Unit metadata', `=IF(AND(${q('Inputs', '$E$5')}<>"",${q('Inputs', '$E$7')}<>""),"Defined","Missing")`, '=IF(B6="Defined","PASS","STOP")', '=IF(C6="STOP","STOP","INFO")', 'Define raw input units'],
    ['Reference metadata', `=IF(AND(${q('Inputs', '$B$5')}<>"",${q('Inputs', '$B$10')}<>""),"Defined","Missing")`, '=IF(B7="Defined","PASS","STOP")', '=IF(C7="STOP","STOP","INFO")', 'Complete well/reference metadata'],
    ['Plan capacity', `=COUNTA(${q('Plan', '$A$7:$A$506')})&" / 500"`, `=IF(COUNTA(${q('Plan', '$A$7:$A$506')})<=500,"PASS","STOP")`, '=IF(C8="STOP","STOP","INFO")', 'Reduce rows or revise engineered capacity'],
    ['Survey capacity', `=COUNTA(${q('Survey', '$A$7:$A$506')})&" / 500"`, `=IF(COUNTA(${q('Survey', '$A$7:$A$506')})<=500,"PASS","STOP")`, '=IF(C9="STOP","STOP","INFO")', 'Reduce rows or revise engineered capacity'],
    ['Plan numeric/range validity', `=COUNTIF(${q('Calc', '$R$7:$R$506')},"INVALID*")`, '=IF(B10=0,"PASS","STOP")', '=IF(C10="STOP","STOP","INFO")', 'Correct plan MD/inclination'],
    ['Survey numeric/range validity', `=COUNTIF(${q('Calc', '$AK$7:$AK$506')},"INVALID*")`, '=IF(B11=0,"PASS","STOP")', '=IF(C11="STOP","STOP","INFO")', 'Correct survey MD/inclination'],
    ['Increasing / duplicate MD', `=COUNTIF(${q('Calc', '$AK$7:$AK$506')},"INVALID*")+COUNTIF(${q('Calc', '$R$7:$R$506')},"INVALID*")`, '=IF(B12=0,"PASS","STOP")', '=IF(C12="STOP","STOP","INFO")', 'Remove duplicate or non-increasing MD'],
    ['Survey gap warning', `=MAX(${q('Calc', '$Y$7:$Y$506')})`, `=IF(B13>${lengthInput(q('Inputs', '$N$9'), '$E$7')},"CAUTION","PASS")`, '=IF(C13="CAUTION","CAUTION","INFO")', 'Review large survey gap'],
    ['Plan coverage', `=LOOKUP(2,1/(${q('Calc', '$AL$7:$AL$506')}<>""),${q('Calc', '$AL$7:$AL$506')})`, '=IF(B14="OK","PASS","CAUTION")', '=IF(C14="CAUTION","CAUTION","INFO")', 'Review terminal plan overrun'],
    ['DLS vs operating limit', `=MAX(${q('Calc', '$AJ$7:$AJ$506')})`, `=IF(B15<=IF(${q('Inputs', '$H$6')}="rad/m",${q('Inputs', '$H$5')},IF(${q('Inputs', '$H$6')}="deg/100ft",${q('Inputs', '$H$5')}/${q('Unit Map', '$E$20')},${q('Inputs', '$H$5')}/${q('Unit Map', '$F$20')})),"PASS","CAUTION")`, '=IF(C15="CAUTION","CAUTION","INFO")', 'Review DLS exceedance'],
    ['Target validity/status', `=COUNTIF(${q('Targets', '$Q$7:$Q$106')},"*INVALID*")`, '=IF(B16=0,"PASS","STOP")', '=IF(C16="STOP","STOP","INFO")', 'Correct target geometry'],
    ['Slide quality', `=COUNTIF(${q('Slide Performance', '$S$7:$S$206')},"<>OK")`, '=IF(B17=0,"PASS","CAUTION")', '=IF(C17="CAUTION","CAUTION","INFO")', 'Review excluded slide intervals'],
    ['Formation coverage', `=COUNTIF(${q('Formation Tops', '$J$7:$J$106')},"BEYOND TD")`, '=IF(B18=0,"PASS","CAUTION")', '=IF(C18="CAUTION","CAUTION","INFO")', 'Review formation pick coverage'],
    ['Formula sentinel', `=COUNTIF(${q('Calc', '$R$7:$R$506')},"#*")+COUNTIF(${q('Calc', '$AK$7:$AK$506')},"#*")`, '=IF(B19=0,"PASS","STOP")', '=IF(C19="STOP","STOP","INFO")', 'Resolve formula errors'],
    ['Rust executable / hash verification', '="NOT RUN"', '="INFO"', '="INFO"', 'Keep the colocated Rust executable and SHA-256 manifest together; calculate to verify'],
    ['ISCWSA covariance / uncertainty', '="Not calculated"', '="INFO"', '="INFO"', 'Use approved uncertainty model for operational decisions'],
    ['Anti-collision / separation factor', '="Not calculated"', '="INFO"', '="INFO"', 'Use approved anti-collision workflow'],
    ['Pipe-fatigue calculation', '="Not calculated"', '="INFO"', '="INFO"', 'Use approved fatigue analysis'],
    ['Deterministic projection', '="Planning screen only"', '="INFO"', '="INFO"', 'Review projection assumptions'],
    ['Rust result integrity / completion', '="NOT RUN"', '="STOP"', '="STOP"', 'Run the hash-verified engine and accept only a verified complete result'],
  ];
  checks.getRange('A6:E25').values = rows.map(([a,,,,e]) => [a, null, null, null, e]);
  checks.getRange('B6:D25').formulas = rows.map(([,b,c,d]) => [b,c,d]);
  checks.getRange('F5:F25').values = [['Exchange record ID'], ...rows.map(([name]) => [`check-${name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')}`])];
  checks.getRange('A5:E5').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };

  summary.getRange('A3:N3').unmerge(); sectionHeader(summary, 'A3:F3', 'Directional decision surface — current state and next action');
  summary.getRange('A5:A10').values = [['Overall state'], ['Latest valid survey MD'], ['Current / terminal crossline error'], ['Current / terminal 3D error'], ['Actual DLS vs operating limit'], ['Next target / required action']];
  summary.getRange('B5:B10').formulas = [
    [`=IF(COUNTIF(${q('Checks', '$D$6:$D$25')},"STOP")>0,"STOP",IF(COUNTIF(${q('Checks', '$D$6:$D$25')},"CAUTION")>0,"CAUTION","READY"))`],
    [`=${q('Results', '$B$6')}*${q('Unit Map', '$I$8')}`], [`=${q('Results', '$B$19')}*${q('Unit Map', '$I$8')}`], [`=${q('Results', '$B$21')}*${q('Unit Map', '$I$8')}`],
    [`=TEXT(${q('Results', '$B$9')}*${q('Unit Map', '$I$20')},"0.00")&" / "&TEXT(IF(${q('Inputs', '$H$6')}="rad/m",${q('Inputs', '$H$5')},IF(${q('Inputs', '$H$6')}="deg/100ft",${q('Inputs', '$H$5')}/${q('Unit Map', '$E$20')},${q('Inputs', '$H$5')}/${q('Unit Map', '$F$20')}))*${q('Unit Map', '$I$20')},"0.00")&" "&${q('Unit Map', '$H$20')}`],
    [`=${q('Results', '$B$11')}&" — "&IF(B5="STOP","Resolve STOP checks",IF(B5="CAUTION","Review cautions before proceeding","Proceed under approved workflow"))`],
  ];
  summary.getRange('C6:C8').formulas = [[`=${q('Unit Map', '$H$8')}`], [`=${q('Unit Map', '$H$8')}`], [`=${q('Unit Map', '$H$8')}`]];
  summary.getRange('B10:F10').merge();
  summary.getRange('A5:A10').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
  summary.getRange('B5:C9').format = { fill: COLORS.grey, font: { bold: true, size: 12 }, borders: { preset: 'outside', style: 'thin', color: COLORS.line } };
  summary.getRange('B10:F10').format = { fill: COLORS.grey, font: { bold: true, size: 12 }, wrapText: true, borders: { preset: 'outside', style: 'thin', color: COLORS.line } };
  summary.getRange('B6:B8').format.numberFormat = '#,##0.00';
  summary.getRange('A:A').format.columnWidth = 35;
  summary.getRange('B:B').format.columnWidth = 31;
  summary.getRange('C:C').format.columnWidth = 14;
  summary.getRange('5:9').format.rowHeight = 26;
  summary.getRange('10:10').format.rowHeight = 44;
  summary.getRange('B5:F10').conditionalFormats.add('containsText', { text: 'STOP', format: { fill: COLORS.redLight, font: { color: COLORS.red, bold: true } } });
  summary.getRange('B5:F10').conditionalFormats.add('containsText', { text: 'CAUTION', format: { fill: COLORS.amberLight, font: { color: COLORS.amber, bold: true } } });
}

function buildTrajectoryEngineEvidence(results) {
  sectionHeader(results, 'O3:P3', 'Rust trajectory engine evidence');
  results.getRange('O5:O14').values = [
    ['Execution mode'], ['State'], ['Request path'], ['Result path'], ['Diagnostic path'],
    ['Request hash'], ['Result hash'], ['Engine version'], ['Executable SHA-256'], ['Accepted UTC'],
  ];
  results.getRange('P5:P14').values = [
    ['RUST REQUIRED — NO VBA FALLBACK'], ['NOT RUN'], [''], [''], [''], [''], [''], [''], [''], [''],
  ];
  results.getRange('O5:O14').format = { fill: COLORS.charcoal, font: { bold: true, color: COLORS.white } };
  results.getRange('P5:P14').format = { fill: COLORS.grey, wrapText: true };
  results.getRange('O:O').format.columnWidth = 24;
  results.getRange('P:P').format.columnWidth = 48;
}

export function buildDirectionalCharts(sheets) {
  const calc = sheets.Calc; const graphs = sheets.Graphs;
  graphs.getRange('A1:N1').unmerge(); graphs.getRange('A1:Q1').merge();
  const blocks = [
    ['DA6:DD506', ['Plan East', 'Plan North', 'Survey East', 'Survey North'], (r) => [`=IF($A${r}="","",$N${r}*${q('Unit Map', '$I$8')})`, `=IF($A${r}="","",$M${r}*${q('Unit Map', '$I$8')})`, `=IF($T${r}="","",$AG${r}*${q('Unit Map', '$I$8')})`, `=IF($T${r}="","",$AF${r}*${q('Unit Map', '$I$8')})`]],
    ['DE6:DH506', ['Plan VS', 'Plan TVD', 'Survey VS', 'Survey TVD'], (r) => [`=IF($A${r}="","",($M${r}*COS(${vsa})+$N${r}*SIN(${vsa}))*${q('Unit Map', '$I$8')})`, `=IF($A${r}="","",$L${r}*${q('Unit Map', '$I$8')})`, `=IF($T${r}="","",($AF${r}*COS(${vsa})+$AG${r}*SIN(${vsa}))*${q('Unit Map', '$I$8')})`, `=IF($T${r}="","",$AE${r}*${q('Unit Map', '$I$8')})`]],
    ['DI6:DK506', ['MD', 'Inclination', 'Azimuth'], (r) => [`=IF($T${r}="","",$V${r}*${q('Unit Map', '$I$8')})`, `=IF($T${r}="","",$W${r}*${q('Unit Map', '$I$18')})`, `=IF($T${r}="","",$X${r}*${q('Unit Map', '$I$18')})`]],
    ['DM6:DO506', ['MD', 'Plan DLS', 'Survey DLS'], (r) => [`=IF($T${r}="","",$V${r}*${q('Unit Map', '$I$8')})`, `=IF($A${r}="","",$Q${r}*${q('Unit Map', '$I$20')})`, `=IF($T${r}="","",$AJ${r}*${q('Unit Map', '$I$20')})`]],
    ['DQ6:DU506', ['MD', 'dTVD', 'dVS', 'Along', 'Crossline'], (r) => [`=IF($T${r}="","",$V${r}*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",$BL${r}*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",($BM${r}*COS(${vsa})+$BN${r}*SIN(${vsa}))*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",$BP${r}*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",$BQ${r}*${q('Unit Map', '$I$8')})`]],
    ['DW6:DY506', ['MD', 'Horizontal Error', '3D Error'], (r) => [`=IF($T${r}="","",$V${r}*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",$BR${r}*${q('Unit Map', '$I$8')})`, `=IF($AL${r}<>"OK","",$BS${r}*${q('Unit Map', '$I$8')})`]],
  ];
  for (const [address, headers, make] of blocks) {
    const [start, end] = address.split(':'); const letters = start.replace(/\d/g, ''); const endLetters = end.replace(/\d/g, '');
    calc.getRange(`${letters}6:${endLetters}6`).values = [headers];
    calc.getRange(`${letters}7:${endLetters}506`).formulas = Array.from({ length: 500 }, (_, i) => make(START + i));
  }
  calc.getRange('DA6:DH6').formulas=[[`="Plan East "&${q('Unit Map', '$H$8')}`,`="Plan North "&${q('Unit Map', '$H$8')}`,`="Survey East "&${q('Unit Map', '$H$8')}`,`="Survey North "&${q('Unit Map', '$H$8')}`,`="Plan VS "&${q('Unit Map', '$H$8')}`,`="Plan TVD "&${q('Unit Map', '$H$8')}`,`="Survey VS "&${q('Unit Map', '$H$8')}`,`="Survey TVD "&${q('Unit Map', '$H$8')}`]];
  calc.getRange('DI6:DK6').formulas=[[`="MD "&${q('Unit Map', '$H$8')}`,`="Inclination "&${q('Unit Map', '$H$18')}`,`="Azimuth "&${q('Unit Map', '$H$18')}`]];
  calc.getRange('DM6:DO6').formulas=[[`="MD "&${q('Unit Map', '$H$8')}`,`="Plan DLS "&${q('Unit Map', '$H$20')}`,`="Survey DLS "&${q('Unit Map', '$H$20')}`]];
  calc.getRange('DQ6:DU6').formulas=[[`="MD "&${q('Unit Map', '$H$8')}`,`="dTVD "&${q('Unit Map', '$H$8')}`,`="dVS "&${q('Unit Map', '$H$8')}`,`="Along "&${q('Unit Map', '$H$8')}`,`="Crossline "&${q('Unit Map', '$H$8')}`]];
  calc.getRange('DW6:DY6').formulas=[[`="MD "&${q('Unit Map', '$H$8')}`,`="Horizontal Error "&${q('Unit Map', '$H$8')}`,`="3D Error "&${q('Unit Map', '$H$8')}`]];
  calc.getRange('EA6:EC206').values = [['Stand', 'Slide Yield', 'Outlier Limit'], ...Array.from({ length: 200 }, () => [null, null, null])];
  calc.getRange('EA7:EC206').formulas = Array.from({ length: 200 }, (_, i) => { const r = START + i; return [`=IF(${q('Slide Performance', `A${r}`)}="","",${q('Slide Performance', `A${r}`)})`, `=IF(${q('Slide Performance', `A${r}`)}="","",${q('Slide Performance', `O${r}`)})`, `=IF(${q('Slide Performance', `A${r}`)}="","",${q('Inputs', '$N$7')})`]; });
  calc.getRange('EE6:EG106').values = [['Target', 'Horizontal Utilization', 'Vertical Utilization'], ...Array.from({ length: 100 }, () => [null, null, null])];
  calc.getRange('EE7:EG106').formulas = Array.from({ length: 100 }, (_, i) => { const r = START + i; return [`=IF(${q('Targets', `A${r}`)}="","",${q('Targets', `A${r}`)})`, `=IF(${q('Targets', `A${r}`)}="","",${q('Targets', `O${r}`)})`, `=IF(${q('Targets', `A${r}`)}="","",${q('Targets', `P${r}`)})`]; });
  const specs = [
    ['scatter', 'DA6:DC506', 'Plan View — East vs North (m)', 'A3', 'H18'], ['scatter', 'DE6:DH506', 'Vertical Section — VS vs TVD (m)', 'J3', 'Q18'],
    ['depth', 'DI6:DK506', 'Inclination and Azimuth vs MD', 'A20', 'H35', ['Inclination','Azimuth'], 'Angle (rad)'], ['depth', 'DM6:DO506', 'Plan / Survey DLS vs MD', 'J20', 'Q35', ['Plan DLS','Survey DLS'], 'DLS (rad/m)'],
    ['depth', 'DQ6:DU506', 'Signed Position Errors vs MD', 'A37', 'H52', ['dTVD','dVS','Along-track','Crossline'], 'Position error (m)'], ['depth', 'DW6:DY506', 'Horizontal and 3D Error vs MD', 'J37', 'Q52', ['Horizontal error','3D error'], 'Position error (m)'],
    ['line', 'EA6:EC206', 'Slide Yield and QC Limit by Stand', 'A54', 'H69'], ['bar', 'EE6:EG106', 'Target Utilization / State Comparison', 'J54', 'Q69'],
  ];
  graphs.charts.deleteAll();
  for (const [type, range, title, start, end, seriesNames, xTitle] of specs) {
    if (type === 'depth') {
      addDepthProfileChart(graphs, calc, range, title, start, end, { seriesNames, xTitle, depthTitle:'MD (m)' });
      continue;
    }
    const chart = graphs.charts.add(type, calc.getRange(range));
    chart.title = title; chart.hasLegend = true; chart.setPosition(start, end);
    if (type === 'line') {
      chart.xAxis.axisType = 'textAxis';
      chart.xAxis.tickLabelInterval = 10;
      chart.xAxis.textStyle.fontSize = 8;
      chart.xAxis.numberFormatCode = '#,##0';
      chart.xAxis.numberFormatSourceLinked = false;
    }
    if (title.startsWith('Plan View')) {
      chart.series.deleteAll();
      const plan = chart.series.add('Plan');
      plan.xFormula = q('Calc', '$DA$7:$DA$506');
      plan.formula = q('Calc', '$DB$7:$DB$506');
      const survey = chart.series.add('Survey');
      survey.xFormula = q('Calc', '$DC$7:$DC$506');
      survey.formula = q('Calc', '$DD$7:$DD$506');
      chart.xAxis.title = { text: 'East (m)' };
      chart.yAxis.title = { text: 'North (m)' };
    }
    if (title.startsWith('Vertical Section')) {
      chart.series.deleteAll();
      const plan = chart.series.add('Plan');
      plan.xFormula = q('Calc', '$DE$7:$DE$506');
      plan.formula = q('Calc', '$DF$7:$DF$506');
      const survey = chart.series.add('Survey');
      survey.xFormula = q('Calc', '$DG$7:$DG$506');
      survey.formula = q('Calc', '$DH$7:$DH$506');
      chart.xAxis.title = { text: 'Vertical section (m)' };
      chart.yAxis.title = { text: 'TVD (m)' };
      chart.yAxis.orientation = 'maxMin';
    }
  }
}

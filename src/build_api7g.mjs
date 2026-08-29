import { createSuiteWorkbook, tableHeader, inputTableStyle, resultsTableStyle, addLineChart, addHeatmapConditionalFormatting } from './workbook.mjs';
import { applyTwoDecimalDisplayPrecision, DISPLAY_PERCENT_FORMAT, sectionHeader } from './common.mjs';
import { MOCK_CASE } from './shared_mock_case.mjs';
import { addExchangeSheets } from './exchange/add_exchange_sheets.mjs';

export function api7gFormulaPlan() {
  return {
    metalArea: '=PI()/4*(D6^2-E6^2)',
    buoyancyFactor: '=1-C6/7850',
    tensionUtilisation: '=F6/G6',
    torqueUtilisation: '=H6/I6',
  };
}

export function buildApi7gWorkbook() {
  const { workbook, sheets } = createSuiteWorkbook('API 7G Drill String Strength and Torque — SI', { extraSheetNames: ['Tubular Catalog', 'Load Cases', 'Section Detail', 'Strength Charts'] });
  const { Summary, Inputs, Results, Graphs, Calc } = sheets;
  const TubularCatalog = sheets['Tubular Catalog'];
  const LoadCases = sheets['Load Cases'];
  const SectionDetail = sheets['Section Detail'];
  const StrengthCharts = sheets['Strength Charts'];
  sectionHeader(Inputs, 'A3:H3', 'SI drill-string inputs — values are stored in SI; display labels are linked to Unit Map');
  Inputs.getRange('A5:H5').values = [['Section', 'Length m', 'Fluid kg/m3', 'OD m', 'ID m', 'Axial load N', 'Tension limit N', 'Operating torque N-m']];
  Inputs.getRange('A6:H11').values = MOCK_CASE.api7g.sections.map((section) => {
    const tubular = MOCK_CASE.tubular[section.tubularKey];
    return [section.name, tubular.lengthM, MOCK_CASE.fluid.densityKgM3, tubular.odM, tubular.idM, section.axialLoadN, section.tensionLimitN, section.operatingTorqueNm];
  });
  Inputs.getRange('I5:I11').values = [['Exchange record ID'], ...MOCK_CASE.api7g.sections.map(({ id }) => [id])];
  tableHeader(Inputs, 'A5:H5'); inputTableStyle(Inputs, 'B6:H11');
  Inputs.getRange('J5:K8').values = [['Control', 'SI value'], ['Surface torque N-m', MOCK_CASE.operation.surfaceTorqueNm], ['Hookload N', MOCK_CASE.rig.hookloadLimitN], ['Design utilisation limit', MOCK_CASE.api7g.designUtilisationLimit]];
  tableHeader(Inputs, 'J5:K5'); inputTableStyle(Inputs, 'K6:K8');

  sectionHeader(Calc, 'A3:J3', 'Section-by-section SI calculations');
  Calc.getRange('A5:J5').values = [['Section', 'Metal area m2', 'Buoyancy factor', 'Buoyed load N', 'Polar moment m4', 'Tension util.', 'Torque capacity N-m', 'Torque util.', 'Combined util.', 'Status']];
  for (let r = 6; r <= 11; r += 1) {
    const ir = r;
    Calc.getRange(`A${r}:J${r}`).formulas = [[
      `=Inputs!A${ir}`,
      `=PI()/4*(Inputs!D${ir}^2-Inputs!E${ir}^2)`,
      `=1-Inputs!C${ir}/7850`,
      `=Inputs!F${ir}*C${r}`,
      `=PI()/32*(Inputs!D${ir}^4-Inputs!E${ir}^4)`,
      `=D${r}/Inputs!G${ir}`,
      `=Inputs!H${ir}*1.35`,
      `=Inputs!$K$6/G${r}`,
      `=SQRT(F${r}^2+H${r}^2)`,
      `=IF(I${r}<=Inputs!K$8,"PASS","REVIEW")`,
    ]];
  }
  Results.getRange('I5').values = [['Exchange record ID']];
  Results.getRange('I6:I11').formulas = Array.from({ length: 6 }, (_, index) => [`=Inputs!I${index + 6}`]);
  tableHeader(Calc, 'A5:J5'); resultsTableStyle(Calc, 'A6:J11');

  sectionHeader(Results, 'A3:H3', 'Strength and torque screening results');
  Results.getRange('A5:H5').values = [['Section', 'Buoyed load N', 'Tension utilisation', 'Torque utilisation', 'Combined utilisation', 'Tension status', 'Torque status', 'Governing']];
  for (let r = 6; r <= 11; r += 1) {
    Results.getRange(`A${r}:H${r}`).formulas = [[`=Calc!A${r}`, `=Calc!D${r}*'Unit Map'!$I$14`, `=Calc!F${r}`, `=Calc!H${r}`, `=Calc!I${r}`, `=IF(C${r}<=Inputs!K$8,"PASS","REVIEW")`, `=IF(D${r}<=Inputs!K$8,"PASS","REVIEW")`, `=IF(E${r}=MAX($E$6:$E$11),"GOVERNING","")`]];
  }
  tableHeader(Results, 'A5:H5'); resultsTableStyle(Results, 'A6:H11');
  Results.getRange('B4').formulas = [[`='Unit Map'!$H$14`]];
  Results.getRange('C4:E4').values = [['fraction', 'fraction', 'fraction']];

  sectionHeader(Summary, 'A3:F3', 'Decision summary');
  Summary.getRange('A5:B8').values = [['Metric', 'Result'], ['Governing section', ''], ['Maximum combined utilisation', ''], ['Status', '']];
  Summary.getRange('B6').formulas = [['=INDEX(Results!A6:A11,MATCH(MAX(Results!E6:E11),Results!E6:E11,0))']];
  Summary.getRange('B7').formulas = [['=MAX(Results!E6:E11)']];
  Summary.getRange('B8').formulas = [['=IF(B7<=Inputs!K8,"WITHIN SCREENING LIMIT","ENGINEERING REVIEW")']];
  tableHeader(Summary, 'A5:B5'); resultsTableStyle(Summary, 'A6:B8');
  Graphs.getRange('A3:C3').values = [['Section','Tension utilisation','Torque utilisation']];
  Graphs.getRange('A4:C9').formulas = [['=Results!A6','=Results!C6','=Results!D6'],['=Results!A7','=Results!C7','=Results!D7'],['=Results!A8','=Results!C8','=Results!D8'],['=Results!A9','=Results!C9','=Results!D9'],['=Results!A10','=Results!C10','=Results!D10'],['=Results!A11','=Results!C11','=Results!D11']];
  const utilisationChart = addLineChart(Graphs, 'A3:C9', 'Section utilisation', 'E3', 'N19');
  Graphs.getRange('A22:C22').values = [['Section','Tension utilisation','Torque utilisation']];
  Graphs.getRange('A23:C28').formulas = Array.from({length:6},(_,i)=>[`=Results!A${i+6}`,`=Results!C${i+6}`,`=Results!D${i+6}`]);
  const comparison = Graphs.charts.add('bar', Graphs.getRange('A22:C28'));
  comparison.title = 'Tension versus torque utilisation'; comparison.hasLegend = true; comparison.setPosition('E21','N37');
  addHeatmapConditionalFormatting(Graphs.getRange('B23:C28'));

  sectionHeader(TubularCatalog,'A3:J3','Tubular and component catalog — editable engineering inputs');
  TubularCatalog.getRange('A5:J5').values=[['Record ID','Component','Length m','OD m','ID m','Metal area m2','Polar moment m4','Tension limit N','Operating torque N-m','Material']];
  MOCK_CASE.api7g.sections.forEach((section,index)=>{const r=6+index; const tubular=MOCK_CASE.tubular[section.tubularKey]; TubularCatalog.getRange(`A${r}:E${r}`).values=[[section.id,section.name,tubular.lengthM,tubular.odM,tubular.idM]]; TubularCatalog.getRange(`F${r}:G${r}`).formulas=[[`=PI()/4*(D${r}^2-E${r}^2)`,`=PI()/32*(D${r}^4-E${r}^4)`]]; TubularCatalog.getRange(`H${r}:J${r}`).values=[[section.tensionLimitN,section.operatingTorqueNm,'Steel']];});
  tableHeader(TubularCatalog,'A5:J5'); inputTableStyle(TubularCatalog,'C6:E11'); resultsTableStyle(TubularCatalog,'F6:J11');

  sectionHeader(LoadCases,'A3:H3','Operation load cases — editable multipliers and design limits');
  LoadCases.getRange('A5:H5').values=[['Operation','Axial multiplier','Torque multiplier','Dynamic factor','Combined factor','Design limit','Enabled','Purpose']];
  LoadCases.getRange('A6:H11').values=[
    ['Static',1,1,1,1,0.90,'Yes','Baseline'],['POOH',1.12,0.80,1.05,1.05,0.90,'Yes','Trip out'],['RIH',0.88,0.65,1.05,1.05,0.90,'Yes','Trip in'],
    ['Rotate',1,1.10,1.10,1.10,0.90,'Yes','Rotary drilling'],['Backream',1.15,1.25,1.15,1.15,0.90,'Yes','Backreaming'],['Overpull',1.30,0.50,1.20,1.20,0.90,'Yes','Contingency'],
  ];
  tableHeader(LoadCases,'A5:H5'); inputTableStyle(LoadCases,'B6:G11');

  sectionHeader(SectionDetail,'A3:L3','Section-by-section operation envelope');
  SectionDetail.getRange('A5:L5').values=[['Operation','Section','Axial load N','Torque N-m','Tension capacity N','Torque capacity N-m','Tension util.','Torque util.','Combined util.','Design limit','Status','Record ID']];
  SectionDetail.getRange('C5:F5').formulas=[[`="Axial load "&'Unit Map'!$H$14`,`="Torque "&'Unit Map'!$H$16`,`="Tension capacity "&'Unit Map'!$H$14`,`="Torque capacity "&'Unit Map'!$H$16`]];
  for(let i=0;i<36;i+=1){const r=6+i; const caseRow=6+Math.floor(i/6); const sectionRow=6+(i%6); SectionDetail.getRange(`A${r}:L${r}`).formulas=[[
    `='Load Cases'!A${caseRow}`,`='Inputs'!A${sectionRow}`,`='Inputs'!F${sectionRow}*'Load Cases'!B${caseRow}*'Load Cases'!D${caseRow}*'Unit Map'!$I$14`,`='Inputs'!$K$6*'Load Cases'!C${caseRow}*'Load Cases'!D${caseRow}*'Unit Map'!$I$16`,
    `='Inputs'!G${sectionRow}*'Unit Map'!$I$14`,`='Calc'!G${sectionRow}*'Unit Map'!$I$16`,`=C${r}/E${r}`,`=D${r}/F${r}`,`=SQRT(G${r}^2+H${r}^2)*'Load Cases'!E${caseRow}`,`='Load Cases'!F${caseRow}`,`=IF(I${r}<=J${r},"PASS","REVIEW")`,`='Inputs'!I${sectionRow}`,
  ]];}
  tableHeader(SectionDetail,'A5:L5'); resultsTableStyle(SectionDetail,'A6:L41'); addHeatmapConditionalFormatting(SectionDetail.getRange('G6:I41'));

  sectionHeader(StrengthCharts,'A3:N3','Operation-specific strength and utilisation profiles');
  StrengthCharts.getRange('A5:D5').values=[['Section','Static','Rotate','Backream']];
  StrengthCharts.getRange('A6:D11').formulas=Array.from({length:6},(_,i)=>[`='Inputs'!A${i+6}`,`='Section Detail'!I${i+6}`,`='Section Detail'!I${i+24}`,`='Section Detail'!I${i+30}`]);
  const envelope = addLineChart(StrengthCharts,'A5:D11','Combined utilisation by operation','F5','N22'); envelope.yAxis.numberFormatCode=DISPLAY_PERCENT_FORMAT; envelope.yAxis.numberFormatSourceLinked=false;
  StrengthCharts.getRange('A25:C25').values=[['Operation','Peak utilisation','Design limit']];
  StrengthCharts.getRange('A26:C31').formulas=Array.from({length:6},(_,i)=>[`='Load Cases'!A${i+6}`,`=MAX('Section Detail'!I${6+i*6}:I${11+i*6})`,`='Load Cases'!F${i+6}`]);
  const caseChart=StrengthCharts.charts.add('bar',StrengthCharts.getRange('A25:C31')); caseChart.title='Operation envelope versus design limit'; caseChart.hasLegend=true; caseChart.setPosition('F25','N42');
  addExchangeSheets(workbook, 'api7g');
  applyTwoDecimalDisplayPrecision(workbook);
  Summary.getRange('B7').format.numberFormat = DISPLAY_PERCENT_FORMAT;
  Results.getRange('C6:E11').format.numberFormat = DISPLAY_PERCENT_FORMAT;
  utilisationChart.yAxis.numberFormatCode = DISPLAY_PERCENT_FORMAT;
  utilisationChart.yAxis.numberFormatSourceLinked = false;
  return workbook;
}

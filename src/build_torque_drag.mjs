import { createSuiteWorkbook, tableHeader, inputTableStyle, resultsTableStyle, addDepthProfileChart, addHeatmapConditionalFormatting } from './workbook.mjs';
import { applyTwoDecimalDisplayPrecision, sectionHeader } from './common.mjs';
import { MOCK_CASE } from './shared_mock_case.mjs';
import { addExchangeSheets } from './exchange/add_exchange_sheets.mjs';

export function torqueDragFormulaPlan() {
  return { dogleg: '=ACOS(COS(B6)*COS(B7)+SIN(B6)*SIN(B7)*COS(C7-C6))', buoyedWeight: '=D6*(1-Inputs!$B$8/7850)', drag: '=E6*Inputs!$B$9', helicalFlag: '=IF(F6<-G6,"REVIEW","PASS")' };
}

export function buildTorqueDragWorkbook() {
  const { workbook, sheets } = createSuiteWorkbook('Torque, Drag and Buckling — SI', { extraSheetNames: ['Wellbore', 'Drillstring', 'Operation Cases', 'ALL', 'PUW', 'SOW', 'BKR', 'SLD', 'ROT', 'DRLG', 'Operation Charts', 'Observed Data', 'Engineering Dashboard'] });
  const { Summary, Inputs, Survey, Results, Graphs, Calc } = sheets;
  const Wellbore=sheets.Wellbore; const Drillstring=sheets.Drillstring; const OperationCases=sheets['Operation Cases']; const All=sheets.ALL; const OperationCharts=sheets['Operation Charts'];
  const firstDataRow = 6;
  const lastDataRow = firstDataRow + MOCK_CASE.surveyStations.length - 1;
  sectionHeader(Inputs,'A3:F3','SI operational assumptions');
  Inputs.getRange('A5:B14').values=[['Fluid density kg/m3',MOCK_CASE.fluid.densityKgM3],['Friction factor',MOCK_CASE.operation.frictionFactor],['WOB N',MOCK_CASE.operation.wobN],['Surface torque N-m',MOCK_CASE.operation.surfaceTorqueNm],['String OD m',MOCK_CASE.tubular.drillPipe.odM],['String ID m',MOCK_CASE.tubular.drillPipe.idM],['Steel density kg/m3',MOCK_CASE.material.steelDensityKgM3],['Young modulus',MOCK_CASE.material.youngModulusPa/1E9],['Tension rating N',MOCK_CASE.rig.hookloadLimitN],['Torsional rating N-m',Math.max(MOCK_CASE.operation.surfaceTorqueNm*2.4,62000)]];
  Inputs.getRange('C12').values=[['GPa']];
  Inputs.getRange('C12').dataValidation={rule:{type:'list',values:['GPa','MPa','Pa','Mpsi','psi']}};
  inputTableStyle(Inputs,'B5:B14'); inputTableStyle(Inputs,'C12');
  const youngModulusPa=`(Inputs!$B$12*IF(Inputs!$C$12="GPa",1E9,IF(Inputs!$C$12="MPa",1E6,IF(Inputs!$C$12="Pa",1,IF(Inputs!$C$12="Mpsi",6894757293.168,IF(Inputs!$C$12="psi",6894.757293168,NA()))))))`;
  sectionHeader(Survey,'A3:H3','SI survey stations — inclination and azimuth are radians');
  Survey.getRange('A5:E5').values=[['MD m','Inclination rad','Azimuth rad','Hole ID m','Exchange record ID']];
  Survey.getRange(`A${firstDataRow}:D${lastDataRow}`).values=MOCK_CASE.surveyStations.map((station) => [station.mdM, station.inclinationRad, station.azimuthRad, station.holeIdM]);
  Survey.getRange(`E${firstDataRow}:E${lastDataRow}`).values=MOCK_CASE.surveyStations.map(({ id }) => [id]);
  tableHeader(Survey,'A5:D5'); inputTableStyle(Survey,`A${firstDataRow}:D${lastDataRow}`);
  sectionHeader(Calc,'A3:N3','Formula-driven profile: POOH, RIH, slide, rotate, backream and buckling screens');
  Calc.getRange('A5:N5').values=[['MD m','dMD m','Dogleg rad','Buoyed w N/m','Normal force N','Drag N','Axial POOH N','Axial RIH N','Slide torque N-m','Rotate torque N-m','Backream torque N-m','Sinusoidal N','Helical N','Status']];
  for(let r=firstDataRow;r<=lastDataRow;r+=1){
    const prev=r-1;
    Calc.getRange(`A${r}:N${r}`).formulas=[[
      `=Survey!A${r}`,r===6?'=0':`=Survey!A${r}-Survey!A${prev}`,r===6?'=0':`=ACOS(COS(Survey!B${prev})*COS(Survey!B${r})+SIN(Survey!B${prev})*SIN(Survey!B${r})*COS(Survey!C${r}-Survey!C${prev}))`,
      `=7850*9.80665*PI()/4*(Inputs!$B$9^2-Inputs!$B$10^2)*(1-Inputs!$B$5/7850)`,
      `=D${r}*B${r}*ABS(C${r})`, `=E${r}*Inputs!$B$6`,
      r===6?`=Inputs!$B$7`:`=G${prev}+D${r}*B${r}+F${r}`, r===6?`=Inputs!$B$7`:`=H${prev}+D${r}*B${r}-F${r}`,
      `=Inputs!$B$8+F${r}*Inputs!$B$9/2`, `=Inputs!$B$8+F${r}*Inputs!$B$9`, `=Inputs!$B$8+F${r}*Inputs!$B$9*1.25`,
      `=2*SQRT(${youngModulusPa}*PI()/64*(Inputs!$B$9^4-Inputs!$B$10^4)*E${r}/MAX(B${r},1))`,
      `=4*SQRT(${youngModulusPa}*PI()/64*(Inputs!$B$9^4-Inputs!$B$10^4)*E${r}/MAX(B${r},1))`,
      `=IF(H${r}<-M${r},"REVIEW","PASS")`
    ]];
  }
  tableHeader(Calc,'A5:N5'); resultsTableStyle(Calc,`A${firstDataRow}:N${lastDataRow}`);
  sectionHeader(Results,'A3:J3','Operation profiles and governing buckling condition');
  Results.getRange('A5:J5').values=[['MD m','POOH N','RIH N','Slide torque N-m','Rotate torque N-m','Backream torque N-m','Sinusoidal limit N','Helical limit N','Buckling status','Governing']];
  Results.getRange('A5:H5').formulas=[[`="MD "&'Unit Map'!$H$8`,`="POOH "&'Unit Map'!$H$14`,`="RIH "&'Unit Map'!$H$14`,`="Slide torque "&'Unit Map'!$H$16`,`="Rotate torque "&'Unit Map'!$H$16`,`="Backream torque "&'Unit Map'!$H$16`,`="Sinusoidal limit "&'Unit Map'!$H$14`,`="Helical limit "&'Unit Map'!$H$14`]];
  for(let r=firstDataRow;r<=lastDataRow;r+=1) Results.getRange(`A${r}:J${r}`).formulas=[[`=Calc!A${r}*'Unit Map'!$I$8`,`=Calc!G${r}*'Unit Map'!$I$14`,`=Calc!H${r}*'Unit Map'!$I$14`,`=Calc!I${r}*'Unit Map'!$I$16`,`=Calc!J${r}*'Unit Map'!$I$16`,`=Calc!K${r}*'Unit Map'!$I$16`,`=Calc!L${r}*'Unit Map'!$I$14`,`=Calc!M${r}*'Unit Map'!$I$14`,`=Calc!N${r}`,`=IF(C${r}=MIN($C$${firstDataRow}:$C$${lastDataRow}),"GOVERNING","")`]];
  Results.getRange('K5').values=[['Exchange record ID']];
  Results.getRange(`K${firstDataRow}:K${lastDataRow}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,index)=>[`=Survey!E${index+firstDataRow}`]);
  tableHeader(Results,'A5:J5'); resultsTableStyle(Results,`A${firstDataRow}:J${lastDataRow}`);
  Results.getRange('A4').formulas = [[`='Unit Map'!$H$8`]]; Results.getRange('B4:C4').formulas = [[`='Unit Map'!$H$14`,`='Unit Map'!$H$14`]]; Results.getRange('D4:F4').formulas = [[`='Unit Map'!$H$16`,`='Unit Map'!$H$16`,`='Unit Map'!$H$16`]]; Results.getRange('G4:H4').formulas = [[`='Unit Map'!$H$14`,`='Unit Map'!$H$14`]];
  sectionHeader(Summary,'A3:F3','Torque, drag and buckling decision summary');
  Summary.getRange('A5:B8').values=[['Metric','Result'],['Peak POOH hookload N',''],['Lowest RIH axial load N',''],['Governing depth m','']];
  Summary.getRange('B6').formulas=[[`=MAX(Results!B${firstDataRow}:B${lastDataRow})`]]; Summary.getRange('B7').formulas=[[`=MIN(Results!C${firstDataRow}:C${lastDataRow})`]]; Summary.getRange('B8').formulas=[[`=INDEX(Results!A${firstDataRow}:A${lastDataRow},MATCH(MIN(Results!C${firstDataRow}:C${lastDataRow}),Results!C${firstDataRow}:C${lastDataRow},0))`]];
  tableHeader(Summary,'A5:B5'); resultsTableStyle(Summary,'A6:B8');
  Graphs.getRange('A3:D3').values=[['MD m','POOH N','RIH N','Rotate torque N-m']];
  Graphs.getRange('A3:D3').formulas=[[`="MD "&'Unit Map'!$H$8`,`="POOH "&'Unit Map'!$H$14`,`="RIH "&'Unit Map'!$H$14`,`="Rotate torque "&'Unit Map'!$H$16`]];
  Graphs.getRange(`A4:D${lastDataRow-2}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=Results!A${i+firstDataRow}`,`=Results!B${i+firstDataRow}`,`=Results!C${i+firstDataRow}`,`=Results!E${i+firstDataRow}`]);
  addDepthProfileChart(Graphs,Graphs,`A3:C${lastDataRow-2}`,'Hookload roadmap vs MD','F3','N19',{seriesNames:['POOH','RIH'],xTitle:'Axial load (N)',depthTitle:'MD (m)'});
  Graphs.getRange('E3:F3').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Rotate torque "&'Unit Map'!$H$16`]]; Graphs.getRange(`E4:F${lastDataRow-2}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=Results!A${i+firstDataRow}`,`=Results!E${i+firstDataRow}`]);
  addDepthProfileChart(Graphs,Graphs,`E3:F${lastDataRow-2}`,'Torque roadmap vs MD','F21','N37',{seriesNames:['Rotating'],xTitle:'Torque (N-m)',depthTitle:'MD (m)'});
  Graphs.getRange('A40:D40').values=[['MD m','Sinusoidal limit N','Helical limit N','RIH axial N']];
  Graphs.getRange('A40:D40').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Sinusoidal limit "&'Unit Map'!$H$14`,`="Helical limit "&'Unit Map'!$H$14`,`="RIH axial "&'Unit Map'!$H$14`]];
  Graphs.getRange(`A41:D${40+MOCK_CASE.surveyStations.length}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=Results!A${i+firstDataRow}`,`=Results!G${i+firstDataRow}`,`=Results!H${i+firstDataRow}`,`=Results!C${i+firstDataRow}`]);
  addDepthProfileChart(Graphs,Graphs,`A40:D${40+MOCK_CASE.surveyStations.length}`,'Buckling roadmap vs MD','F39','N55',{seriesNames:['Sinusoidal limit','Helical limit','RIH axial'],xTitle:'Axial load (N)',depthTitle:'MD (m)'});
  Graphs.getRange('A58:C58').values=[['MD m','Sinusoidal utilisation','Helical utilisation']];
  Graphs.getRange(`A59:C${58+MOCK_CASE.surveyStations.length}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=Results!A${i+firstDataRow}`,`=Results!G${i+firstDataRow}/MAX(Results!B${i+firstDataRow},1)`,`=Results!H${i+firstDataRow}/MAX(Results!B${i+firstDataRow},1)`]);
  addHeatmapConditionalFormatting(Graphs.getRange(`B59:C${58+MOCK_CASE.surveyStations.length}`));

  sectionHeader(Wellbore,'A3:L3','Wellbore section register — filled from surface to TD');
  Wellbore.getRange('A5:L5').values=[['Record ID','Type','Top MD m','Bottom MD m','Length m','OD m','ID m','Friction factor','Branch','Branch type','Fluid density kg/m3','Status']];
  MOCK_CASE.holeSections.forEach((section,index)=>{const r=6+index; Wellbore.getRange(`A${r}:L${r}`).values=[[section.id,'Open hole',section.topMdM,section.bottomMdM,section.bottomMdM-section.topMdM,section.holeIdM,0,MOCK_CASE.operation.frictionFactor,'Main','Primary',MOCK_CASE.fluid.densityKgM3,'PASS']];});
  tableHeader(Wellbore,'A5:L5'); inputTableStyle(Wellbore,'B6:K6');

  sectionHeader(Drillstring,'A3:N3','Drillstring component register — bottom to top');
  Drillstring.getRange('A5:N5').values=[['Record ID','Type','Name','Length m','OD m','ID m','Area m2','Unit weight N/m','Material','Grade YS Pa','Class','Connection','Top m','Bottom m']];
  const stringRows=[['ds-bit-sub','BHA','Bit / Sub','bitSub'],['ds-motor','BHA','Motor / RSS','motorRss'],['ds-mwd','BHA','MWD / LWD','mwdLwd'],['ds-dc','BHA','Drill Collar','drillCollar'],['ds-hwdp','Tubular','HWDP','hwdp'],['ds-dp','Tubular','Drill Pipe','drillPipe']];
  stringRows.forEach((row,index)=>{const r=6+index; const t=MOCK_CASE.tubular[row[3]]; Drillstring.getRange(`A${r}:F${r}`).values=[[row[0],row[1],row[2],t.lengthM,t.odM,t.idM]]; Drillstring.getRange(`G${r}:H${r}`).formulas=[[`=PI()/4*(E${r}^2-F${r}^2)`,`=G${r}*'Inputs'!$B$11*9.80665`]]; Drillstring.getRange(`I${r}:L${r}`).values=[['Steel',758000000,'P','Reference connection']]; Drillstring.getRange(`M${r}:N${r}`).formulas=[[index===0?'=0':`=N${r-1}`,index===0?`=D${r}`:`=M${r}+D${r}`]];});
  tableHeader(Drillstring,'A5:N5'); inputTableStyle(Drillstring,'D6:F11'); resultsTableStyle(Drillstring,'G6:N11');

  sectionHeader(OperationCases,'A3:J3','Torque-and-drag operating cases');
  OperationCases.getRange('A5:J5').values=[['Code','Operation','Direction','RPM','Speed m/s','Overpull N','WOB N','Torque factor','Friction factor','Enabled']];
  OperationCases.getRange('A6:J11').values=[['PUW','Pull out','Up',0,0.25,0,0,0.80,MOCK_CASE.operation.frictionFactor,'Yes'],['SOW','Slack off','Down',0,0.25,0,0,0.65,MOCK_CASE.operation.frictionFactor,'Yes'],['BKR','Backream','Up',80,0.12,80000,0,1.25,MOCK_CASE.operation.frictionFactor,'Yes'],['SLD','Slide drill','Down',0,0.02,0,MOCK_CASE.operation.wobN,0.50,MOCK_CASE.operation.frictionFactor,'Yes'],['ROT','Rotate','Down',MOCK_CASE.operation.rotarySpeedRpm,0.02,0,MOCK_CASE.operation.wobN,1.00,MOCK_CASE.operation.frictionFactor,'Yes'],['DRLG','Drill ahead','Down',MOCK_CASE.operation.rotarySpeedRpm,0.02,0,MOCK_CASE.operation.wobN,1.10,MOCK_CASE.operation.frictionFactor,'Yes']];
  tableHeader(OperationCases,'A5:J5'); inputTableStyle(OperationCases,'D6:J11');

  sectionHeader(All,'A3:N3','Consolidated depth profile across all operations');
  All.getRange('A5:N5').values=[['MD m','Inc rad','Azi rad','TVD screen m','PUW axial N','SOW axial N','BKR torque N-m','SLD torque N-m','ROT torque N-m','DRLG torque N-m','Sinusoidal N','Helical N','Buckling status','Record ID']];
  All.getRange('A5:L5').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Inc "&'Unit Map'!$H$18`,`="Azi "&'Unit Map'!$H$18`,`="TVD screen "&'Unit Map'!$H$8`,`="PUW axial "&'Unit Map'!$H$14`,`="SOW axial "&'Unit Map'!$H$14`,`="BKR torque "&'Unit Map'!$H$16`,`="SLD torque "&'Unit Map'!$H$16`,`="ROT torque "&'Unit Map'!$H$16`,`="DRLG torque "&'Unit Map'!$H$16`,`="Sinusoidal "&'Unit Map'!$H$14`,`="Helical "&'Unit Map'!$H$14`]];
  All.getRange(`A6:N${5+MOCK_CASE.surveyStations.length}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>{const r=i+firstDataRow; return [`='Results'!A${r}`,`='Survey'!B${r}*'Unit Map'!$I$18`,`='Survey'!C${r}*'Unit Map'!$I$18`,`='Results'!A${r}*COS('Survey'!B${r})`,`='Results'!B${r}`,`='Results'!C${r}`,`='Results'!F${r}`,`='Results'!D${r}`,`='Results'!E${r}`,`='Results'!E${r}*1.10`,`='Results'!G${r}`,`='Results'!H${r}`,`='Results'!I${r}`,`='Results'!K${r}`];});
  tableHeader(All,'A5:N5'); resultsTableStyle(All,`A6:N${5+MOCK_CASE.surveyStations.length}`); addHeatmapConditionalFormatting(All.getRange(`E6:M${5+MOCK_CASE.surveyStations.length}`));

  const operationSpecs=[['PUW',5,9],['SOW',6,10],['BKR',6,7],['SLD',6,8],['ROT',6,9],['DRLG',6,10]];
  for(const [name,axialCol,torqueCol] of operationSpecs){
    const sheet=sheets[name];
    const opLast=5+MOCK_CASE.surveyStations.length;
    sectionHeader(sheet,'A3:H3',`${name} operation depth profile`);
    sheet.getRange('A5:H5').values=[['MD m','Axial load N','Torque N-m','Sinusoidal limit N','Helical limit N','Axial margin N','Buckling status','Record ID']];
    sheet.getRange('A5:F5').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Axial load "&'Unit Map'!$H$14`,`="Torque "&'Unit Map'!$H$16`,`="Sinusoidal limit "&'Unit Map'!$H$14`,`="Helical limit "&'Unit Map'!$H$14`,`="Axial margin "&'Unit Map'!$H$14`]];
    sheet.getRange(`A6:H${opLast}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>{const r=6+i; const axialLetter=String.fromCharCode(64+axialCol); const torqueLetter=String.fromCharCode(64+torqueCol); return [`='ALL'!A${r}`,`='ALL'!${axialLetter}${r}`,`='ALL'!${torqueLetter}${r}`,`='ALL'!K${r}`,`='ALL'!L${r}`,`=B${r}-E${r}`,`='ALL'!M${r}`,`='ALL'!N${r}`];});
    tableHeader(sheet,'A5:H5'); resultsTableStyle(sheet,`A6:H${opLast}`);
    const helperStart=opLast+3;
    sheet.getRange(`A${helperStart}:D${helperStart}`).values=[['MD m','Axial load N','Sinusoidal limit N','Helical limit N']];
    sheet.getRange(`A${helperStart}:D${helperStart}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="Axial load "&'Unit Map'!$H$14`,`="Sinusoidal limit "&'Unit Map'!$H$14`,`="Helical limit "&'Unit Map'!$H$14`]];
    sheet.getRange(`A${helperStart+1}:D${helperStart+MOCK_CASE.surveyStations.length}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=A${i+6}`,`=B${i+6}`,`=D${i+6}`,`=E${i+6}`]);
    sheet.getRange(`F${helperStart}:G${helperStart}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="Torque "&'Unit Map'!$H$16`]];
    sheet.getRange(`F${helperStart+1}:G${helperStart+MOCK_CASE.surveyStations.length}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`=A${i+6}`,`=C${i+6}`]);
    addDepthProfileChart(sheet,sheet,`A${helperStart}:D${helperStart+MOCK_CASE.surveyStations.length}`,`${name} axial load and buckling vs MD`,'J3','R18',{seriesNames:['Axial load','Sinusoidal limit','Helical limit'],xTitle:'Axial load (N)',depthTitle:'MD (m)'});
    addDepthProfileChart(sheet,sheet,`F${helperStart}:G${helperStart+MOCK_CASE.surveyStations.length}`,`${name} torque vs MD`,'J20','R35',{seriesNames:[`${name} torque`],xTitle:'Torque (N-m)',depthTitle:'MD (m)'});
  }

  sectionHeader(OperationCharts,'A3:N3','Operation-specific hookload, torque and buckling charts');
  let chartTop=5;
  for(const [name] of operationSpecs){
    const helperStart=chartTop;
    const helperEnd=helperStart+MOCK_CASE.surveyStations.length;
    OperationCharts.getRange(`A${helperStart}:E${helperStart}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="Axial load "&'Unit Map'!$H$14`,`="MD "&'Unit Map'!$H$8`,`="Torque "&'Unit Map'!$H$16`,`="Buckling margin "&'Unit Map'!$H$14`]];
    OperationCharts.getRange(`A${helperStart+1}:E${helperEnd}`).formulas=Array.from({length:MOCK_CASE.surveyStations.length},(_,i)=>[`='${name}'!A${i+6}`,`='${name}'!B${i+6}`,`='${name}'!A${i+6}`,`='${name}'!C${i+6}`,`='${name}'!F${i+6}`]);
    addDepthProfileChart(OperationCharts,OperationCharts,`A${helperStart}:B${helperEnd}`,`${name} axial load vs MD`,'G'+chartTop,'L'+(chartTop+15),{seriesNames:[name],xTitle:'Axial load (N)',depthTitle:'MD (m)'});
    addDepthProfileChart(OperationCharts,OperationCharts,`C${helperStart}:D${helperEnd}`,`${name} torque vs MD`,'M'+chartTop,'R'+(chartTop+15),{seriesNames:[name],xTitle:'Torque (N-m)',depthTitle:'MD (m)'});
    chartTop+=17;
  }
  addTorqueDragIndustryDashboard(sheets, firstDataRow, lastDataRow);
  addExchangeSheets(workbook, 'torqueDrag');
  applyTwoDecimalDisplayPrecision(workbook);
  return workbook;
}

function addTorqueDragIndustryDashboard(sheets, firstDataRow, lastDataRow) {
  const dashboard=sheets['Engineering Dashboard'];
  const observed=sheets['Observed Data'];
  const settings=sheets['Chart Settings'];
  const count=lastDataRow-firstDataRow+1;
  const helperHeader=60;
  const helperLast=helperHeader+count;
  settings.getRange('B6').values=[[MOCK_CASE.surveyStations[Math.floor(MOCK_CASE.surveyStations.length/2)].mdM]];

  sectionHeader(observed,'A3:F3','Mock observed data — replace with validated EDR or rig measurements before operational use');
  observed.getRange('A5:D5').values=[['MD m (canonical SI)','Observed hookload N (canonical SI)','Observed torque N-m (canonical SI)','Provenance']];
  observed.getRange(`A6:D${5+count}`).formulas=Array.from({length:count},(_,i)=>{const r=firstDataRow+i; return [`='Survey'!A${r}`,`='Calc'!G${r}*(1+0.018*SIN(${i+1}))`,`='Calc'!J${r}*(1+0.012*COS(${i+1}))`,`="MOCK — replace with EDR"`];});
  tableHeader(observed,'A5:D5'); inputTableStyle(observed,`A6:D${5+count}`);
  observed.getRange('A:A').format.columnWidth=22; observed.getRange('B:C').format.columnWidth=28; observed.getRange('D:D').format.columnWidth=28;

  sectionHeader(dashboard,'A3:N3','Integrated torque-and-drag engineering review — operations, observations, limits and well context');
  dashboard.getRange('A5:B5').values=[['Selected MD','']];
  dashboard.getRange('B5').formulas=[[`='Chart Settings'!B6*'Unit Map'!$I$8`]];
  dashboard.getRange('C5').formulas=[[`='Unit Map'!$H$8`]];
  inputTableStyle(dashboard,'B5');
  dashboard.getRange('A8:B17').values=[['Nearest station MD',''],['Inclination',''],['PUW',''],['SOW',''],['Observed hookload',''],['Tension margin',''],['ROT torque',''],['Observed torque',''],['Torsional margin',''],['Governing state','']];
  const selectedRow=`MATCH($B$5,$A$${helperHeader+1}:$A$${helperLast},1)`;
  dashboard.getRange('B8:B17').formulas=[
    [`=INDEX($A$${helperHeader+1}:$A$${helperLast},${selectedRow})`],
    [`=INDEX($U$${helperHeader+1}:$U$${helperLast},${selectedRow})`],
    [`=INDEX($B$${helperHeader+1}:$B$${helperLast},${selectedRow})`],
    [`=INDEX($C$${helperHeader+1}:$C$${helperLast},${selectedRow})`],
    [`=INDEX($H$${helperHeader+1}:$H$${helperLast},${selectedRow})`],
    [`=INDEX($I$${helperHeader+1}:$I$${helperLast},${selectedRow})-INDEX($H$${helperHeader+1}:$H$${helperLast},${selectedRow})`],
    [`=INDEX($O$${helperHeader+1}:$O$${helperLast},${selectedRow})`],
    [`=INDEX($Q$${helperHeader+1}:$Q$${helperLast},${selectedRow})`],
    [`=INDEX($R$${helperHeader+1}:$R$${helperLast},${selectedRow})-INDEX($Q$${helperHeader+1}:$Q$${helperLast},${selectedRow})`],
    [`=IF(OR(B13<0,B16<0),"REVIEW","WITHIN LIMITS")`],
  ];
  tableHeader(dashboard,'A8:A17'); resultsTableStyle(dashboard,'B8:B17');
  dashboard.getRange('A19:C22').values=[['Well context','Top MD','Bottom MD'],...MOCK_CASE.holeSections.map((section)=>[section.name,section.topMdM,section.bottomMdM]),['Drillstring extent',0,Object.values(MOCK_CASE.tubular).reduce((sum,item)=>sum+item.lengthM,0)],['Calculation range',MOCK_CASE.surveyStations[0].mdM,MOCK_CASE.surveyStations.at(-1).mdM]];
  tableHeader(dashboard,'A19:C19'); resultsTableStyle(dashboard,'A20:C22');

  dashboard.getRange(`A${helperHeader}:K${helperHeader}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="PUW "&'Unit Map'!$H$14`,`="SOW "&'Unit Map'!$H$14`,`="BKR "&'Unit Map'!$H$14`,`="SLD "&'Unit Map'!$H$14`,`="ROT "&'Unit Map'!$H$14`,`="DRLG "&'Unit Map'!$H$14`,`="Observed hookload "&'Unit Map'!$H$14`,`="Tension rating "&'Unit Map'!$H$14`,`="Sinusoidal buckling "&'Unit Map'!$H$14`,`="Helical buckling "&'Unit Map'!$H$14`]];
  dashboard.getRange(`A${helperHeader+1}:K${helperLast}`).formulas=Array.from({length:count},(_,i)=>{const r=firstDataRow+i; const o=6+i; return [
    `='Results'!A${r}`,`='Results'!B${r}`,`='Results'!C${r}`,`=('Calc'!G${r}+'Inputs'!$B$7*0.35)*'Unit Map'!$I$14`,`=('Calc'!H${r}-'Inputs'!$B$7)*'Unit Map'!$I$14`,`=('Calc'!H${r}-'Inputs'!$B$7*0.25)*'Unit Map'!$I$14`,`=('Calc'!H${r}-'Inputs'!$B$7*0.5)*'Unit Map'!$I$14`,`='Observed Data'!B${o}*'Unit Map'!$I$14`,`='Inputs'!$B$13*'Unit Map'!$I$14`,`='Results'!G${r}`,`='Results'!H${r}`,
  ];});
  dashboard.getRange(`M${helperHeader}:R${helperHeader}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="BKR "&'Unit Map'!$H$16`,`="ROT "&'Unit Map'!$H$16`,`="DRLG "&'Unit Map'!$H$16`,`="Observed torque "&'Unit Map'!$H$16`,`="Torsional rating "&'Unit Map'!$H$16`]];
  dashboard.getRange(`M${helperHeader+1}:R${helperLast}`).formulas=Array.from({length:count},(_,i)=>{const r=firstDataRow+i; const o=6+i; return [`='Results'!A${r}`,`='Results'!F${r}`,`='Results'!E${r}`,`='Results'!E${r}*1.1`,`='Observed Data'!C${o}*'Unit Map'!$I$16`,`='Inputs'!$B$14*'Unit Map'!$I$16`];});
  dashboard.getRange(`T${helperHeader}:U${helperHeader}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="Inclination "&'Unit Map'!$H$18`]];
  dashboard.getRange(`T${helperHeader+1}:U${helperLast}`).formulas=Array.from({length:count},(_,i)=>{const r=firstDataRow+i; return [`='Results'!A${r}`,`='Survey'!B${r}*'Unit Map'!$I$18`];});
  dashboard.getRange(`W${helperHeader}:Z${helperHeader}`).formulas=[[`="MD "&'Unit Map'!$H$8`,`="Low friction PUW "&'Unit Map'!$H$14`,`="Base friction PUW "&'Unit Map'!$H$14`,`="High friction PUW "&'Unit Map'!$H$14`]];
  dashboard.getRange(`W${helperHeader+1}:Z${helperLast}`).formulas=Array.from({length:count},(_,i)=>{const r=firstDataRow+i; const neutral=`('Inputs'!$B$7+'Calc'!D${r}*'Calc'!A${r})`; return [`='Results'!A${r}`,`=(${neutral}+('Calc'!G${r}-${neutral})*'Chart Settings'!$B$8)*'Unit Map'!$I$14`,`='Results'!B${r}`,`=(${neutral}+('Calc'!G${r}-${neutral})*'Chart Settings'!$B$10)*'Unit Map'!$I$14`];});

  const axialStyles=['#0F766E','#D97706','#6B7280','#7C3AED','#2563EB','#15803D','#111827','#B91C1C','#D97706','#DC2626'].map((color)=>({color,weight:2}));
  addDepthProfileChart(dashboard,dashboard,`A${helperHeader}:K${helperLast}`,'Axial load — model / actual / limits','D4','N20',{seriesNames:['PUW','SOW','BKR','SLD','ROT','DRLG','Observed hookload','Tension rating','Sinusoidal buckling','Helical buckling'],seriesStyles:axialStyles,xTitle:'Axial load',depthTitle:'MD'});
  addDepthProfileChart(dashboard,dashboard,`M${helperHeader}:R${helperLast}`,'Torque — model / actual / limits','D22','N38',{seriesNames:['BKR','ROT','DRLG','Observed torque','Torsional rating'],seriesStyles:['#6B7280','#2563EB','#15803D','#111827','#B91C1C'].map((color)=>({color,weight:2})),xTitle:'Torque',depthTitle:'MD'});
  addDepthProfileChart(dashboard,dashboard,`T${helperHeader}:U${helperLast}`,'Inclination and well geometry vs MD','D40','N56',{seriesNames:['Inclination'],seriesStyles:[{color:'#0F766E',weight:2}],xTitle:'Inclination',depthTitle:'MD'});
  addDepthProfileChart(dashboard,dashboard,`W${helperHeader}:Z${helperLast}`,'Friction sensitivity — PUW vs MD','O40','Y56',{seriesNames:['Low friction','Base friction','High friction'],seriesStyles:[{color:'#6B7280',weight:2},{color:'#0F766E',weight:2},{color:'#B91C1C',weight:2}],xTitle:'Axial load',depthTitle:'MD'});
  dashboard.getRange('A:A').format.columnWidth=24; dashboard.getRange('B:C').format.columnWidth=18;
}

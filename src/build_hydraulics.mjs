import { createSuiteWorkbook, tableHeader, inputTableStyle, resultsTableStyle, addDepthProfileChart, addScatterChart, addHeatmapConditionalFormatting } from './workbook.mjs';
import { applyTwoDecimalDisplayPrecision, DISPLAY_PERCENT_FORMAT, sectionHeader } from './common.mjs';
import { MOCK_CASE } from './shared_mock_case.mjs';
import { addExchangeSheets } from './exchange/add_exchange_sheets.mjs';

export function hydraulicsFormulaPlan() {
  return { velocity: '=Inputs!$B$8/C6', sectionPressureLoss: '=F6*B6/C6*(Inputs!$B$9*D6^2/2)', nozzleArea: '=PI()/4*(Inputs!$B$13^2)*Inputs!$B$12', candidateScore: '=SUM(F6:H6)' };
}

export function buildHydraulicsWorkbook() {
  const { workbook, sheets } = createSuiteWorkbook('Steady-State Hydraulics and Nozzle Optimization — SI', { extraSheetNames: ['Fluid Model', 'Flow Path', 'Nozzle Cases', 'Pressure Profile', 'Hydraulics Charts', 'Flow Cases', 'Hydraulics Dashboard'] });
  const { Summary, Inputs, Results, Graphs, Calc } = sheets;
  const FluidModel=sheets['Fluid Model']; const FlowPath=sheets['Flow Path']; const NozzleCases=sheets['Nozzle Cases']; const PressureProfile=sheets['Pressure Profile']; const HydraulicsCharts=sheets['Hydraulics Charts'];
  const baseNozzle = MOCK_CASE.pumpNozzle.nozzles.find(({ id }) => id === MOCK_CASE.pumpNozzle.baseNozzleId);
  sectionHeader(Inputs, 'A3:H3', 'SI operating inputs and full tube-section flow path');
  Inputs.getRange('A5:B15').values = [['Rig preset',MOCK_CASE.rig.preset],['Surface pressure limit Pa',MOCK_CASE.hydraulics.surfacePressureLimitPa],['Pump efficiency',MOCK_CASE.rig.pumpEfficiency],['Flow rate m3/s',MOCK_CASE.hydraulics.flowRateM3S],['Mud density kg/m3',MOCK_CASE.fluid.densityKgM3],['Apparent viscosity Pa-s',MOCK_CASE.fluid.apparentViscosityPaS],['Bit nozzles count',MOCK_CASE.pumpNozzle.nozzleCount],['Base nozzle diameter m',baseNozzle.diameterM],['Nozzle Cd',MOCK_CASE.pumpNozzle.dischargeCoefficient],['Max ECD screen kg/m3',MOCK_CASE.rig.ecdScreenDensityKgM3],['Minimum annular velocity screen m/s',0.50]];
  inputTableStyle(Inputs, 'B5:B15');
  Inputs.getRange('D5:H5').values = [['Tube section', 'Length m', 'Flow ID m', 'Annular / pipe', 'Hydraulic diameter m']];
  Inputs.getRange('D6:H13').values = MOCK_CASE.hydraulics.flowPath.map((section) => [section.name, section.lengthM, section.flowIdM, section.flowType, section.hydraulicDiameterM]);
  Inputs.getRange('I5:I13').values = [['Exchange record ID'], ...MOCK_CASE.hydraulics.flowPath.map(({ id }) => [id])];
  tableHeader(Inputs, 'D5:H5'); inputTableStyle(Inputs, 'D6:H13');

  sectionHeader(Calc, 'A3:J3', 'Formula-driven pressure-loss calculations');
  Calc.getRange('A5:J5').values = [['Tube section','Length m','Hydraulic dia. m','Velocity m/s','Reynolds','Friction factor','Pressure loss Pa','Cumulative Pa','Pressure % limit','Status']];
  for (let r=6;r<=13;r+=1) {
    const ir=r;
    Calc.getRange(`A${r}:J${r}`).formulas = [[`=Inputs!D${ir}`,`=Inputs!E${ir}`,`=Inputs!H${ir}`,`=Inputs!$B$8/IF(Inputs!G${ir}="Annulus",PI()/4*((Inputs!F${ir}+Inputs!H${ir})^2-Inputs!F${ir}^2),PI()/4*C${r}^2)`,`=Inputs!$B$9*D${r}*C${r}/Inputs!$B$10`,`=IF(E${r}>4000,0.3164/E${r}^0.25,64/E${r})`,`=F${r}*B${r}/C${r}*(Inputs!$B$9*D${r}^2/2)`,`=SUM($G$6:G${r})`,`=H${r}/Inputs!$B$6`,`=IF(I${r}<=1,"PASS","REVIEW")`]];
  }
  Calc.getRange('L5:Q5').values = [['Nozzle d m','Total area m2','Nozzle velocity m/s','Bit drop Pa','Total SPP Pa','Score']];
  Calc.getRange('L6:L10').values = MOCK_CASE.pumpNozzle.nozzles.map(({ diameterM }) => [diameterM]);
  Calc.getRange('R5:R10').values = [['Exchange record ID'], ...MOCK_CASE.pumpNozzle.nozzles.map(({ id }) => [id])];
  for (let r=6;r<=10;r+=1) Calc.getRange(`M${r}:Q${r}`).formulas = [[`=PI()/4*L${r}^2*Inputs!$B$11`,`=Inputs!$B$8/M${r}`,`=Inputs!$B$9/2*(N${r}/Inputs!$B$13)^2`,`=H13+O${r}`,`=ABS(P${r}-Inputs!$B$6)/Inputs!$B$6`]];
  tableHeader(Calc,'A5:J5'); resultsTableStyle(Calc,'A6:J13'); tableHeader(Calc,'L5:Q5'); resultsTableStyle(Calc,'L6:Q10');

  sectionHeader(Results,'A3:H3','Hydraulics result and nozzle recommendation');
  Results.getRange('A5:E5').values = [['Metric','SI result','Display unit','Limit / target','Status']];
  Results.getRange('A6:E10').values = [['Total flow-path loss',null,'Pa',null,null],['Recommended nozzle diameter',null,'m','User-select',null],['Recommended surface pressure',null,'Pa',null,null],['Nozzle velocity',null,'m/s','Screening only','INFO'],['ECD screening input',null,'kg/m3','Screening only','INFO']];
  Results.getRange('B6').formulas = [[`=Calc!H13*'Unit Map'!$I$15`]]; Results.getRange('C6').formulas = [[`='Unit Map'!$H$15`]]; Results.getRange('D6').formulas = [[`=Inputs!B6*'Unit Map'!$I$15`]]; Results.getRange('E6').formulas = [['=IF(B6<=D6,"PASS","REVIEW")']];
  Results.getRange('B7').formulas = [[`=INDEX(Calc!L6:L10,MATCH(MIN(Calc!Q6:Q10),Calc!Q6:Q10,0))*'Unit Map'!$I$9`]]; Results.getRange('C7').formulas = [[`='Unit Map'!$H$9`]]; Results.getRange('E7').formulas = [['=IF(B7>0,"PASS","REVIEW")']];
  Results.getRange('B8').formulas = [[`=INDEX(Calc!P6:P10,MATCH(MIN(Calc!Q6:Q10),Calc!Q6:Q10,0))*'Unit Map'!$I$15`]]; Results.getRange('C8').formulas = [[`='Unit Map'!$H$15`]]; Results.getRange('D8').formulas = [[`=Inputs!B6*'Unit Map'!$I$15`]]; Results.getRange('E8').formulas = [['=IF(B8<=D8,"PASS","REVIEW")']];
  Results.getRange('B9').formulas = [[`=INDEX(Calc!N6:N10,MATCH(MIN(Calc!Q6:Q10),Calc!Q6:Q10,0))*'Unit Map'!$I$19`]]; Results.getRange('C9').formulas = [[`='Unit Map'!$H$19`]]; Results.getRange('B10').formulas = [[`=Inputs!B14*'Unit Map'!$I$13`]]; Results.getRange('C10').formulas = [[`='Unit Map'!$H$13`]];
  Results.getRange('B6:B10').format.numberFormat = '#,##0.00';
  tableHeader(Results,'A5:E5'); resultsTableStyle(Results,'A6:E10');
  Summary.getRange('A3:D3').merge(); Summary.getRange('A3').values=[['Hydraulics decision — pressure limit and selected nozzle']];
  Summary.getRange('A5:B7').values=[['Metric','Result'],['Surface pressure status',''],['Selected nozzle diameter m','']];
  Summary.getRange('B6').formulas=[['=Results!E8']]; Summary.getRange('B7').formulas=[['=Results!B7']]; tableHeader(Summary,'A5:B5'); resultsTableStyle(Summary,'A6:B7');
  Graphs.getRange('A3:C3').values=[['Tube section','Cumulative pressure Pa','Pressure loss Pa']];
  Graphs.getRange('B3:C3').formulas=[[`="Cumulative pressure "&'Unit Map'!$H$15`,`="Pressure loss "&'Unit Map'!$H$15`]];
  Graphs.getRange('A4:C11').formulas=[['=Calc!A6',`=Calc!H6*'Unit Map'!$I$15`,`=Calc!G6*'Unit Map'!$I$15`],['=Calc!A7',`=Calc!H7*'Unit Map'!$I$15`,`=Calc!G7*'Unit Map'!$I$15`],['=Calc!A8',`=Calc!H8*'Unit Map'!$I$15`,`=Calc!G8*'Unit Map'!$I$15`],['=Calc!A9',`=Calc!H9*'Unit Map'!$I$15`,`=Calc!G9*'Unit Map'!$I$15`],['=Calc!A10',`=Calc!H10*'Unit Map'!$I$15`,`=Calc!G10*'Unit Map'!$I$15`],['=Calc!A11',`=Calc!H11*'Unit Map'!$I$15`,`=Calc!G11*'Unit Map'!$I$15`],['=Calc!A12',`=Calc!H12*'Unit Map'!$I$15`,`=Calc!G12*'Unit Map'!$I$15`],['=Calc!A13',`=Calc!H13*'Unit Map'!$I$15`,`=Calc!G13*'Unit Map'!$I$15`]];
  const sectionLoss = Graphs.charts.add('bar', Graphs.getRange('A3:C11'));
  sectionLoss.title = 'Pressure loss by flow-path section'; sectionLoss.hasLegend = true; sectionLoss.setPosition('E3','N19');
  Graphs.getRange('A14:C14').values=[['Section','Base Pa','Increment Pa']];
  Graphs.getRange('B14:C14').formulas=[[`="Base "&'Unit Map'!$H$15`,`="Increment "&'Unit Map'!$H$15`]];
  Graphs.getRange('A15:C22').formulas=Array.from({length:8},(_,i)=>[`=Calc!A${i+6}`,i===0?'=0':`=Calc!H${i+5}*'Unit Map'!$I$15`,`=Calc!G${i+6}*'Unit Map'!$I$15`]);
  const waterfall = Graphs.charts.add('bar', Graphs.getRange('A14:C22'));
  waterfall.title = 'Waterfall-style pressure-loss breakdown'; waterfall.hasLegend = true; waterfall.setPosition('E21','N37');
  Graphs.getRange('E40:H40').values=[['Nozzle diameter m','Total SPP Pa','Surface limit Pa','Candidate score']];
  Graphs.getRange('E40:G40').formulas=[[`="Nozzle diameter "&'Unit Map'!$H$9`,`="Total SPP "&'Unit Map'!$H$15`,`="Surface limit "&'Unit Map'!$H$15`]];
  Graphs.getRange('E41:H45').formulas=Array.from({length:5},(_,i)=>[`=Calc!L${i+6}*'Unit Map'!$I$9`,`=Calc!P${i+6}*'Unit Map'!$I$15`,`=Inputs!$B$6*'Unit Map'!$I$15`,`=Calc!Q${i+6}`]);
  addScatterChart(Graphs,'E40:G45','Nozzle optimization — SPP versus surface limit','E39','N54');

  sectionHeader(FluidModel,'A3:H3','Fluid and rheology model — SI canonical inputs');
  FluidModel.getRange('A5:D5').values=[['Property','Value','SI unit','Role']];
  FluidModel.getRange('A6:D13').values=[
    ['Mud density',MOCK_CASE.fluid.densityKgM3,'kg/m3','Hydrostatic and inertia'],['Apparent viscosity',MOCK_CASE.fluid.apparentViscosityPaS,'Pa-s','Newtonian screen'],['Flow behaviour index',0.72,'fraction','Power-law exponent'],['Consistency index',0.32,'Pa-s^n','Power-law consistency'],
    ['Yield point',8.5,'Pa','Herschel-Bulkley screen'],['Plastic viscosity',0.024,'Pa-s','Bingham screen'],['Temperature',333.15,'K','Reference condition'],['Compressibility',0.00000000045,'1/Pa','Screening input'],
  ];
  tableHeader(FluidModel,'A5:D5'); inputTableStyle(FluidModel,'B6:B13');
  FluidModel.getRange('F5:H5').values=[['Model','Selected','Applicability']]; FluidModel.getRange('F6:H8').values=[['Newtonian','No','Initial screening'],['Power law','Yes','Section pressure loss'],['Bingham plastic','No','Cross-check']]; tableHeader(FluidModel,'F5:H5'); inputTableStyle(FluidModel,'G6:G8');

  sectionHeader(FlowPath,'A3:N3','Complete hydraulic flow path and regime results');
  FlowPath.getRange('A5:N5').values=[['Record ID','Section','Type','Length m','Hydraulic dia. m','Area m2','Velocity m/s','Reynolds','Regime','Friction factor','Loss Pa','Cumulative Pa','% limit','Status']];
  FlowPath.getRange('D5:G5').formulas=[[`="Length "&'Unit Map'!$H$8`,`="Hydraulic dia. "&'Unit Map'!$H$9`,`="Area "&'Unit Map'!$H$10`,`="Velocity "&'Unit Map'!$H$19`]];
  FlowPath.getRange('K5:L5').formulas=[[`="Loss "&'Unit Map'!$H$15`,`="Cumulative "&'Unit Map'!$H$15`]];
  for(let i=0;i<8;i+=1){const r=6+i; const cr=6+i; FlowPath.getRange(`A${r}:N${r}`).formulas=[[
    `='Inputs'!I${cr}`,`='Calc'!A${cr}`,`='Inputs'!G${cr}`,`='Calc'!B${cr}*'Unit Map'!$I$8`,`='Calc'!C${cr}*'Unit Map'!$I$9`,`=IF(C${r}="Annulus",PI()/4*(('Inputs'!F${cr}+'Calc'!C${cr})^2-'Inputs'!F${cr}^2),PI()/4*'Calc'!C${cr}^2)*'Unit Map'!$I$10`,`='Calc'!D${cr}*'Unit Map'!$I$19`,`='Calc'!E${cr}`,`=IF(H${r}<2100,"LAMINAR",IF(H${r}<4000,"TRANSITION","TURBULENT"))`,`='Calc'!F${cr}`,`='Calc'!G${cr}*'Unit Map'!$I$15`,`='Calc'!H${cr}*'Unit Map'!$I$15`,`='Calc'!I${cr}`,`='Calc'!J${cr}`,
  ]];}
  tableHeader(FlowPath,'A5:N5'); resultsTableStyle(FlowPath,'A6:N13'); addHeatmapConditionalFormatting(FlowPath.getRange('M6:M13'));

  sectionHeader(NozzleCases,'A3:L3','Bit nozzle candidate envelope and optimisation');
  NozzleCases.getRange('A5:L5').values=[['Record ID','Diameter m','Count','Total area m2','Velocity m/s','Bit drop Pa','Flow-path loss Pa','SPP Pa','Pressure margin Pa','Hydraulic power W','HSI W/m2','Rank']];
  NozzleCases.getRange('B5:I5').formulas=[[`="Diameter "&'Unit Map'!$H$9`,`="Count"`,`="Total area "&'Unit Map'!$H$10`,`="Velocity "&'Unit Map'!$H$19`,`="Bit drop "&'Unit Map'!$H$15`,`="Flow-path loss "&'Unit Map'!$H$15`,`="SPP "&'Unit Map'!$H$15`,`="Pressure margin "&'Unit Map'!$H$15`]];
  for(let i=0;i<5;i+=1){const r=6+i; const cr=6+i; NozzleCases.getRange(`A${r}:L${r}`).formulas=[[
    `='Calc'!R${cr}`,`='Calc'!L${cr}*'Unit Map'!$I$9`,`='Inputs'!$B$11`,`='Calc'!M${cr}*'Unit Map'!$I$10`,`='Calc'!N${cr}*'Unit Map'!$I$19`,`='Calc'!O${cr}*'Unit Map'!$I$15`,`='Calc'!$H$13*'Unit Map'!$I$15`,`='Calc'!P${cr}*'Unit Map'!$I$15`,`=('Inputs'!$B$6-'Calc'!P${cr})*'Unit Map'!$I$15`,`='Calc'!O${cr}*'Inputs'!$B$8`,`=J${r}/(PI()/4*0.216^2)`,`=RANK(ABS(I${r}),$I$6:$I$10,1)`,
  ]];}
  tableHeader(NozzleCases,'A5:L5'); resultsTableStyle(NozzleCases,'A6:L10'); addHeatmapConditionalFormatting(NozzleCases.getRange('H6:I10'));

  sectionHeader(PressureProfile,'A3:J3','Pressure, ECD and annular velocity profile');
  PressureProfile.getRange('A5:J5').values=[['Section','Depth / length m','Velocity m/s','Section loss Pa','Cumulative loss Pa','Hydrostatic Pa','Dynamic pressure Pa','ECD kg/m3','Pressure % limit','Status']];
  PressureProfile.getRange('B5:H5').formulas=[[`="Depth / length "&'Unit Map'!$H$8`,`="Velocity "&'Unit Map'!$H$19`,`="Section loss "&'Unit Map'!$H$15`,`="Cumulative loss "&'Unit Map'!$H$15`,`="Hydrostatic "&'Unit Map'!$H$15`,`="Dynamic pressure "&'Unit Map'!$H$15`,`="ECD "&'Unit Map'!$H$13`]];
  for(let i=0;i<8;i+=1){const r=6+i; const cr=6+i; PressureProfile.getRange(`A${r}:J${r}`).formulas=[[
    `='Calc'!A${cr}`,`=SUM('Calc'!$B$6:'Calc'!B${cr})*'Unit Map'!$I$8`,`='Calc'!D${cr}*'Unit Map'!$I$19`,`='Calc'!G${cr}*'Unit Map'!$I$15`,`='Calc'!H${cr}*'Unit Map'!$I$15`,`='Inputs'!$B$9*9.80665*SUM('Calc'!$B$6:'Calc'!B${cr})*'Unit Map'!$I$15`,`=('Inputs'!$B$9*9.80665*SUM('Calc'!$B$6:'Calc'!B${cr})+'Calc'!H${cr})*'Unit Map'!$I$15`,`=('Inputs'!$B$9+IF('Inputs'!G${cr}="Annulus",'Calc'!G${cr}/(9.80665*MAX('Calc'!B${cr},1)),0))*'Unit Map'!$I$13`,`='Calc'!H${cr}/'Inputs'!$B$6`,`=IF(I${r}<=1,"PASS","REVIEW")`,
  ]];}
  tableHeader(PressureProfile,'A5:J5'); resultsTableStyle(PressureProfile,'A6:J13'); addHeatmapConditionalFormatting(PressureProfile.getRange('H6:I13'));

  sectionHeader(HydraulicsCharts,'A3:N3','Hydraulics engineering plots');
  HydraulicsCharts.getRange('A:A').format.columnWidth = 18;
  HydraulicsCharts.getRange('B:E').format.columnWidth = 22;
  HydraulicsCharts.getRange('A5:E5').values=[['MD m','Section loss Pa','Cumulative loss Pa','Hydrostatic Pa','Dynamic pressure Pa']];
  HydraulicsCharts.getRange('A5:E5').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Section loss "&'Unit Map'!$H$15`,`="Cumulative loss "&'Unit Map'!$H$15`,`="Hydrostatic "&'Unit Map'!$H$15`,`="Dynamic pressure "&'Unit Map'!$H$15`]];
  HydraulicsCharts.getRange('A6:E13').formulas=Array.from({length:8},(_,i)=>[`='Pressure Profile'!B${i+6}`,`='Pressure Profile'!D${i+6}`,`='Pressure Profile'!E${i+6}`,`='Pressure Profile'!F${i+6}`,`='Pressure Profile'!G${i+6}`]);
  addDepthProfileChart(HydraulicsCharts,HydraulicsCharts,'A5:E13','Pressure roadmap vs MD','F5','N22',{seriesNames:['Section loss','Cumulative loss','Hydrostatic','Dynamic pressure'],xTitle:'Pressure (Pa)',depthTitle:'MD (m)'});
  HydraulicsCharts.getRange('A25:D25').values=[['MD m','Static mud density kg/m3','ECD kg/m3','ECD screen kg/m3']];
  HydraulicsCharts.getRange('A25:D25').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Static mud density "&'Unit Map'!$H$13`,`="ECD "&'Unit Map'!$H$13`,`="ECD screen "&'Unit Map'!$H$13`]];
  HydraulicsCharts.getRange('A26:D33').formulas=Array.from({length:8},(_,i)=>[`='Pressure Profile'!B${i+6}`,`='Inputs'!$B$9*'Unit Map'!$I$13`,`='Pressure Profile'!H${i+6}`,`='Inputs'!$B$14*'Unit Map'!$I$13`]);
  addDepthProfileChart(HydraulicsCharts,HydraulicsCharts,'A25:D33','ECD window vs MD','F24','N41',{seriesNames:['Static mud density','ECD','ECD screen'],xTitle:'Density (kg/m3)',depthTitle:'MD (m)'});
  HydraulicsCharts.getRange('A44:C44').values=[['MD m','Flow velocity m/s','Minimum annular velocity m/s']];
  HydraulicsCharts.getRange('A44:C44').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Flow velocity "&'Unit Map'!$H$19`,`="Minimum annular velocity "&'Unit Map'!$H$19`]];
  HydraulicsCharts.getRange('A45:C52').formulas=Array.from({length:8},(_,i)=>[`='Pressure Profile'!B${i+6}`,`='Pressure Profile'!C${i+6}`,`='Inputs'!$B$15*'Unit Map'!$I$19`]);
  addDepthProfileChart(HydraulicsCharts,HydraulicsCharts,'A44:C52','Velocity and transport screen vs MD','F43','N60',{seriesNames:['Flow velocity','Minimum annular velocity'],xTitle:'Velocity (m/s)',depthTitle:'MD (m)'});
  HydraulicsCharts.getRange('A63:D63').values=[['Nozzle diameter m','SPP Pa','Surface limit Pa','Bit drop Pa']];
  HydraulicsCharts.getRange('A63:D63').formulas=[[`="Nozzle diameter "&'Unit Map'!$H$9`,`="SPP "&'Unit Map'!$H$15`,`="Surface limit "&'Unit Map'!$H$15`,`="Bit drop "&'Unit Map'!$H$15`]];
  HydraulicsCharts.getRange('A64:D68').formulas=Array.from({length:5},(_,i)=>[`=Calc!L${i+6}*'Unit Map'!$I$9`,`=Calc!P${i+6}*'Unit Map'!$I$15`,`=Inputs!$B$6*'Unit Map'!$I$15`,`=Calc!O${i+6}*'Unit Map'!$I$15`]);
  addScatterChart(HydraulicsCharts,'A63:D68','Nozzle pressure envelope','F62','N79');
  addHydraulicsIndustryDashboard(sheets);
  addExchangeSheets(workbook, 'hydraulics');
  applyTwoDecimalDisplayPrecision(workbook);
  for (const [sheetName, chartIndex] of [['Graphs',2],['Hydraulics Charts',3],['Hydraulics Dashboard',3]]) {
    const chart=sheets[sheetName].charts.items[chartIndex];
    if (chart?.xAxis) { chart.xAxis.numberFormatCode='0.000'; chart.xAxis.numberFormatSourceLinked=false; }
  }
  sheets['Flow Cases'].getRange('D6:D8').format.numberFormat='0.000';
  Calc.getRange('I6:I13').format.numberFormat = DISPLAY_PERCENT_FORMAT;
  Calc.getRange('Q6:Q10').format.numberFormat = DISPLAY_PERCENT_FORMAT;
  return workbook;
}

function addHydraulicsIndustryDashboard(sheets) {
  const dashboard=sheets['Hydraulics Dashboard'];
  const flowCases=sheets['Flow Cases'];
  const settings=sheets['Chart Settings'];
  settings.getRange('B6').values=[[MOCK_CASE.hydraulics.flowPath.reduce((sum,section)=>sum+section.lengthM,0)/2]];

  sectionHeader(flowCases,'A3:F3','Hydraulic sensitivity cases — editable multipliers of the canonical base flow rate');
  flowCases.getRange('A5:E5').values=[['Case','Multiplier','Label','Flow rate m3/s (canonical SI)','Enabled']];
  flowCases.getRange('A6:E8').values=[['FLOW-LOW',0.85,'Low',null,'Yes'],['FLOW-BASE',1,'Base',null,'Yes'],['FLOW-HIGH',1.15,'High',null,'Yes']];
  flowCases.getRange('D6:D8').formulas=[[`='Inputs'!$B$8*B6`],[`='Inputs'!$B$8*B7`],[`='Inputs'!$B$8*B8`]];
  tableHeader(flowCases,'A5:E5'); inputTableStyle(flowCases,'B6:B8'); resultsTableStyle(flowCases,'D6:D8');
  flowCases.getRange('E6:E8').dataValidation={rule:{type:'list',values:['Yes','No']}};
  flowCases.getRange('A:A').format.columnWidth=20; flowCases.getRange('C:D').format.columnWidth=26;

  sectionHeader(dashboard,'A3:Y3','Integrated hydraulics review — pressure, ECD, transport, flow sensitivity and nozzle envelope');
  dashboard.getRange('A5:C5').values=[['Selected MD','',null]];
  dashboard.getRange('B5').formulas=[[`='Chart Settings'!B6*'Unit Map'!$I$8`]];
  dashboard.getRange('C5').formulas=[[`='Unit Map'!$H$8`]];
  inputTableStyle(dashboard,'B5');
  dashboard.getRange('A8:B16').values=[['Nearest station MD',''],['Flow case','Base'],['Total dynamic pressure',''],['Pressure margin',''],['ECD',''],['ECD margin',''],['Annular velocity',''],['Transport margin',''],['Governing state','']];
  const pressureMatch=`MATCH($B$5,$A$46:$A$53,1)`;
  const ecdMatch=`MATCH($B$5,$H$46:$H$53,1)`;
  const velocityMatch=`MATCH($B$5,$O$46:$O$53,1)`;
  dashboard.getRange('B8:B16').formulas=[
    [`=INDEX($A$46:$A$53,${pressureMatch})`],['="Base"'],[`=INDEX($D$46:$D$53,${pressureMatch})`],[`=INDEX($F$46:$F$53,${pressureMatch})-INDEX($D$46:$D$53,${pressureMatch})`],[`=INDEX($J$46:$J$53,${ecdMatch})`],[`=INDEX($M$46:$M$53,${ecdMatch})-INDEX($J$46:$J$53,${ecdMatch})`],[`=INDEX($Q$46:$Q$53,${velocityMatch})`],[`=INDEX($Q$46:$Q$53,${velocityMatch})-INDEX($S$46:$S$53,${velocityMatch})`],[`=IF(OR(B11<0,B13<0,B15<0),"REVIEW","WITHIN LIMITS")`],
  ];
  tableHeader(dashboard,'A8:A16'); resultsTableStyle(dashboard,'B8:B16');
  let runningDepth=0;
  const flowContext=MOCK_CASE.hydraulics.flowPath.map((section)=>{const row=[section.name,runningDepth,runningDepth+section.lengthM]; runningDepth+=section.lengthM; return row;}).slice(-3);
  dashboard.getRange('A18:C22').values=[['Flow-path context','Top / start','Bottom / end'],...flowContext,['Profile basis','Surface equipment','Open-hole annulus']];
  tableHeader(dashboard,'A18:C18'); resultsTableStyle(dashboard,'A19:C22');

  dashboard.getRange('A45:F45').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Low flow pressure "&'Unit Map'!$H$15`,`="Base flow pressure "&'Unit Map'!$H$15`,`="High flow pressure "&'Unit Map'!$H$15`,`="Hydrostatic "&'Unit Map'!$H$15`,`="Pressure limit "&'Unit Map'!$H$15`]];
  dashboard.getRange('A46:F53').formulas=Array.from({length:8},(_,i)=>{const r=6+i; return [`='Pressure Profile'!B${r}`,`=('Pressure Profile'!F${r}+('Pressure Profile'!G${r}-'Pressure Profile'!F${r})*'Flow Cases'!B6^1.75)`,`='Pressure Profile'!G${r}`,`=('Pressure Profile'!F${r}+('Pressure Profile'!G${r}-'Pressure Profile'!F${r})*'Flow Cases'!B8^1.75)`,`='Pressure Profile'!F${r}`,`='Inputs'!$B$6*'Unit Map'!$I$15`];});
  dashboard.getRange('H45:M45').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Low flow ECD "&'Unit Map'!$H$13`,`="Base flow ECD "&'Unit Map'!$H$13`,`="High flow ECD "&'Unit Map'!$H$13`,`="Static density "&'Unit Map'!$H$13`,`="ECD limit "&'Unit Map'!$H$13`]];
  dashboard.getRange('H46:M53').formulas=Array.from({length:8},(_,i)=>{const r=6+i; return [`='Pressure Profile'!B${r}`,`=('Inputs'!$B$9+('Pressure Profile'!H${r}/'Unit Map'!$I$13-'Inputs'!$B$9)*'Flow Cases'!B6^1.75)*'Unit Map'!$I$13`,`='Pressure Profile'!H${r}`,`=('Inputs'!$B$9+('Pressure Profile'!H${r}/'Unit Map'!$I$13-'Inputs'!$B$9)*'Flow Cases'!B8^1.75)*'Unit Map'!$I$13`,`='Inputs'!$B$9*'Unit Map'!$I$13`,`='Inputs'!$B$14*'Unit Map'!$I$13`];});
  dashboard.getRange('O45:S45').formulas=[[`="MD "&'Unit Map'!$H$8`,`="Low flow velocity "&'Unit Map'!$H$19`,`="Base flow velocity "&'Unit Map'!$H$19`,`="High flow velocity "&'Unit Map'!$H$19`,`="Minimum transport velocity "&'Unit Map'!$H$19`]];
  dashboard.getRange('O46:S53').formulas=Array.from({length:8},(_,i)=>{const r=6+i; return [`='Pressure Profile'!B${r}`,`='Pressure Profile'!C${r}*'Flow Cases'!B6`,`='Pressure Profile'!C${r}`,`='Pressure Profile'!C${r}*'Flow Cases'!B8`,`='Inputs'!$B$15*'Unit Map'!$I$19`];});
  dashboard.getRange('U45:X45').formulas=[[`="Nozzle diameter "&'Unit Map'!$H$9`,`="SPP "&'Unit Map'!$H$15`,`="Surface limit "&'Unit Map'!$H$15`,`="Bit drop "&'Unit Map'!$H$15`]];
  dashboard.getRange('U46:X50').formulas=Array.from({length:5},(_,i)=>[`='Hydraulics Charts'!A${64+i}`,`='Hydraulics Charts'!B${64+i}`,`='Hydraulics Charts'!C${64+i}`,`='Hydraulics Charts'!D${64+i}`]);

  addDepthProfileChart(dashboard,dashboard,'A45:F53','Pressure flow families and operating limit vs MD','D4','N20',{seriesNames:['Low flow pressure','Base flow pressure','High flow pressure','Hydrostatic','Pressure limit'],seriesStyles:[{color:'#6B7280',weight:2},{color:'#0F766E',weight:2},{color:'#15803D',weight:2},{color:'#2563EB',weight:2},{color:'#B91C1C',weight:2}],xTitle:'Pressure',depthTitle:'MD'});
  addDepthProfileChart(dashboard,dashboard,'H45:M53','ECD flow-rate window vs MD','O4','Y20',{seriesNames:['Low flow ECD','Base flow ECD','High flow ECD','Static density','ECD limit'],seriesStyles:[{color:'#6B7280',weight:2},{color:'#0F766E',weight:2},{color:'#15803D',weight:2},{color:'#2563EB',weight:2},{color:'#B91C1C',weight:2}],xTitle:'Density',depthTitle:'MD'});
  addDepthProfileChart(dashboard,dashboard,'O45:S53','Annular velocity flow families vs MD','D22','N38',{seriesNames:['Low flow velocity','Base flow velocity','High flow velocity','Minimum transport velocity'],seriesStyles:[{color:'#6B7280',weight:2},{color:'#0F766E',weight:2},{color:'#15803D',weight:2},{color:'#B91C1C',weight:2}],xTitle:'Velocity',depthTitle:'MD'});
  addScatterChart(dashboard,'U45:X50','Nozzle pressure envelope','O22','Y38');
  dashboard.getRange('A:A').format.columnWidth=24; dashboard.getRange('B:C').format.columnWidth=18;
}

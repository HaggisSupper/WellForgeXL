import { createSuiteWorkbook, tableHeader, inputTableStyle, resultsTableStyle, addLineChart, addScatterChart, addPolarScatterChart, addHeatmapConditionalFormatting } from './workbook.mjs';
import { applyTwoDecimalDisplayPrecision, sectionHeader } from './common.mjs';
import { MOCK_CASE } from './shared_mock_case.mjs';
import { addExchangeSheets } from './exchange/add_exchange_sheets.mjs';
import fs from 'node:fs';

const RUST_FIXTURE = JSON.parse(fs.readFileSync(new URL('../engine/fixtures/expected/release-one-minimal.result.json', import.meta.url), 'utf8'));

export function bhaFormulaPlan() {
  return { firstFrequency: '=1/(2*PI())*SQRT(K6/M6)', bendingStress: '=M6*D6/(2*I6)', polarX: '=R6*COS(Q6)', polarY: '=R6*SIN(Q6)' };
}

export function buildBhaWorkbook() {
  const { workbook, sheets } = createSuiteWorkbook('BHA Vibration, Bending and Drill-Ahead Tendency — SI', { extraSheetNames: ['BHA Assembly', 'Vibration Modes', 'Bending Response', 'BHA Geometry View', 'Tendency Matrix', 'Polar Plot', 'Rust Engine', 'Rust Engine Results', 'Rust Calc'] });
  const { Summary, Inputs, Results, Graphs, Calc } = sheets;
  const BhaAssembly=sheets['BHA Assembly']; const VibrationModes=sheets['Vibration Modes']; const BendingResponse=sheets['Bending Response']; const BhaGeometry=sheets['BHA Geometry View']; const TendencyMatrix=sheets['Tendency Matrix']; const PolarPlot=sheets['Polar Plot'];
  const RustEngine=sheets['Rust Engine']; const RustResults=sheets['Rust Engine Results']; const RustCalc=sheets['Rust Calc']; RustCalc.visibility='hidden';
  RustEngine.getRange('A1').values=[['WellForge | Rust Engine | Value-only calculation client']];
  RustResults.getRange('A1').values=[['WellForge | Rust Engine Results | Value-only decision surface']];
  RustCalc.getRange('A1').values=[['WellForge | Rust Calc | Value-only helper arrays']];
  sectionHeader(Inputs,'A3:H3','SI BHA, operating and WOB/toolface-case inputs');
  Inputs.getRange('A5:B10').values=[['RPM',MOCK_CASE.operation.rotarySpeedRpm],['Flow rate m3/s',MOCK_CASE.hydraulics.flowRateM3S],['Young modulus',MOCK_CASE.material.youngModulusPa/1E9],['Steel density kg/m3',MOCK_CASE.material.steelDensityKgM3],['WOB case 1 N',MOCK_CASE.operation.lowWobN],['WOB case 2 N',MOCK_CASE.operation.wobN]];
  Inputs.getRange('C7').values=[['GPa']];
  Inputs.getRange('C7').dataValidation={rule:{type:'list',values:['GPa','MPa','Pa','Mpsi','psi']}};
  Inputs.getRange('A11:B15').values=[['Hole diameter m',MOCK_CASE.holeSections[0].holeIdM],['Projection plane','Highside'],['Deflection display scale',0.001],['Fluid density kg/m3',1200],['BHA inclination deg',MOCK_CASE.surveyStations.at(-1).inclinationRad*180/Math.PI]];
  Inputs.getRange('B12').dataValidation={rule:{type:'list',values:['Highside','Lowside']}};
  Inputs.getRange('C11:C15').values=[['Canonical SI'],['Display sign'],['Dimensionless'],['Canonical SI'],['WITSML trajectory projection']];
  inputTableStyle(Inputs,'B5:B15'); inputTableStyle(Inputs,'C7');
  Inputs.getRange('B13').format.numberFormat='0.0000';
  const youngModulusPa=`(Inputs!$B$7*IF(Inputs!$C$7="GPa",1E9,IF(Inputs!$C$7="MPa",1E6,IF(Inputs!$C$7="Pa",1,IF(Inputs!$C$7="Mpsi",6894757293.168,IF(Inputs!$C$7="psi",6894.757293168,NA()))))))`;
  Inputs.getRange('D5:H5').values=[['BHA component','Length m','OD m','ID m','Support factor']];
  Inputs.getRange('D6:H11').values=MOCK_CASE.bha.map(({ name, lengthM, odM, idM, supportFactor }) => [name, lengthM, odM, idM, supportFactor]);
  Inputs.getRange('I5:I11').values=[['Exchange record ID'],...MOCK_CASE.bha.map(({id})=>[id])];
  tableHeader(Inputs,'D5:H5'); inputTableStyle(Inputs,'D6:H11');
  sectionHeader(Calc,'A3:N3','BHA static bending, first-mode screen and tendency response');
  Calc.getRange('A5:N5').values=[['Component','Length m','OD m','ID m','Area m2','I m4','Mass kg','Stiffness N/m','First mode Hz','Bending moment Nm','Bending stress Pa','WOB 1 tendency','WOB 2 tendency','Vibration screen']];
  for(let r=6;r<=11;r+=1) Calc.getRange(`A${r}:N${r}`).formulas=[[`=Inputs!D${r}`,`=Inputs!E${r}`,`=Inputs!F${r}`,`=Inputs!G${r}`,`=PI()/4*(C${r}^2-D${r}^2)`,`=PI()/64*(C${r}^4-D${r}^4)`,`=E${r}*B${r}*Inputs!$B$8`,`=48*${youngModulusPa}*F${r}*Inputs!H${r}/B${r}^3`,`=1/(2*PI())*SQRT(H${r}/G${r})`,`=Inputs!$B$9*B${r}*Inputs!H${r}/4`,`=J${r}*C${r}/(2*F${r})`,`=Inputs!$B$9/H${r}*0.0001`,`=Inputs!$B$10/H${r}*0.0001`,`=IF(I${r}>Inputs!$B$5/60*1.2,"CLEAR","REVIEW")`]];
  tableHeader(Calc,'A5:N5'); resultsTableStyle(Calc,'A6:N11');
  Calc.getRange('P5:AD5').values=[['Distance from bit SI','Distance from bit','Component','Local fraction','Estimated centreline SI','Projected centreline','Hole high wall','Hole low wall','BHA OD high','BHA OD low','BHA ID high','BHA ID low','Projected radial clearance','Zero clearance','Geometry flag']];
  Calc.getRange('AE5:AI5').values=[['Distance from bit','Bending moment','Bending stress','Stress screen','Projected centreline']];
  for(let component=0;component<6;component+=1){
    const inputRow=6+component;
    for(let sample=0;sample<5;sample+=1){
      const row=6+component*5+sample;
      const fraction=sample/4;
      const priorLength=component===0?'0':`SUM(Inputs!$E$6:E${inputRow-1})`;
      const deflection=`Calc!J${inputRow}*Calc!B${inputRow}^2/(8*${youngModulusPa}*Calc!F${inputRow})`;
      Calc.getRange(`P${row}:AD${row}`).formulas=[[
        `=${priorLength}+Inputs!E${inputRow}*${fraction}`,
        `=P${row}*'Unit Map'!$I$8`,
        `=Inputs!D${inputRow}`,
        `=${fraction}`,
        `=IF(Inputs!$B$12="Lowside",-1,1)*(${deflection})*Inputs!$B$13*SIN(PI()*S${row})`,
        `=T${row}*'Unit Map'!$I$9`,
        `=Inputs!$B$11/2*'Unit Map'!$I$9`,
        `=-Inputs!$B$11/2*'Unit Map'!$I$9`,
        `=(T${row}+Inputs!F${inputRow}/2)*'Unit Map'!$I$9`,
        `=(T${row}-Inputs!F${inputRow}/2)*'Unit Map'!$I$9`,
        `=(T${row}+Inputs!G${inputRow}/2)*'Unit Map'!$I$9`,
        `=(T${row}-Inputs!G${inputRow}/2)*'Unit Map'!$I$9`,
        `=(Inputs!$B$11/2-ABS(T${row})-Inputs!F${inputRow}/2)*'Unit Map'!$I$9`,
        `=0`,
        `=IF(AB${row}<0,"OVERLAP INDICATION","CLEARANCE")`,
      ]];
      Calc.getRange(`AE${row}:AI${row}`).formulas=[[
        `=Q${row}`,`=Calc!J${inputRow}*'Unit Map'!$I$16`,`=Calc!K${inputRow}*'Unit Map'!$I$17`,`=350000000*'Unit Map'!$I$17`,`=U${row}`,
      ]];
    }
  }
  tableHeader(Calc,'P5:AI5'); resultsTableStyle(Calc,'P6:AI35');
  sectionHeader(Results,'A3:I3','BHA response and toolface/WOB rose data');
  Results.getRange('A5:E5').values=[['Component','First mode Hz','Bending stress Pa','WOB 1 tendency','WOB 2 tendency']];
  for(let r=6;r<=11;r+=1) Results.getRange(`A${r}:E${r}`).formulas=[[`=Calc!A${r}`,`=Calc!I${r}`,`=Calc!K${r}*'Unit Map'!$I$17`,`=Calc!L${r}`,`=Calc!M${r}`]];
  Results.getRange('F5').values=[['Exchange record ID']]; Results.getRange('F6:F11').formulas=Array.from({length:6},(_,index)=>[`=Inputs!I${index+6}`]);
  Results.getRange('C4').formulas = [[`='Unit Map'!$H$17`]];
  Results.getRange('G5:K5').values=[['Toolface rad','WOB 1 magnitude','WOB 1 X','WOB 1 Y','WOB 2 magnitude']];
  Results.getRange('G5').formulas=[[`="Toolface "&'Unit Map'!$H$18`]];
  Results.getRange('L5:M5').values=[['WOB 2 X','WOB 2 Y']];
  for(let r=6;r<=17;r+=1){const angle=(r-6)*Math.PI/6; Results.getRange(`G${r}`).formulas=[[`=${angle}*'Unit Map'!$I$18`]]; Results.getRange(`H${r}:M${r}`).formulas=[[`=AVERAGE($D$6:$D$11)*(1+0.35*COS(G${r}/'Unit Map'!$I$18))`,`=H${r}*COS(G${r}/'Unit Map'!$I$18)`,`=H${r}*SIN(G${r}/'Unit Map'!$I$18)`,`=AVERAGE($E$6:$E$11)*(1+0.35*COS(G${r}/'Unit Map'!$I$18))`,`=K${r}*COS(G${r}/'Unit Map'!$I$18)`,`=K${r}*SIN(G${r}/'Unit Map'!$I$18)`]];}
  Results.getRange('N5:N17').values=[['Exchange record ID'],...Array.from({length:12},(_,index)=>[`toolface-${String(index*30).padStart(3,'0')}deg`])];
  tableHeader(Results,'A5:E5'); resultsTableStyle(Results,'A6:E11'); tableHeader(Results,'G5:M5'); resultsTableStyle(Results,'G6:M17');
  sectionHeader(Summary,'A3:E3','BHA decision summary');
  Summary.getRange('A5:B8').values=[['Metric','Result'],['Lowest first mode Hz',''],['Peak bending stress Pa',''],['Vibration screening','']];
  Summary.getRange('B6').formulas=[['=MIN(Results!B6:B11)']]; Summary.getRange('B7').formulas=[['=MAX(Results!C6:C11)']]; Summary.getRange('B8').formulas=[['=IF(COUNTIF(Calc!N6:N11,"REVIEW")>0,"REVIEW","CLEAR")']];
  tableHeader(Summary,'A5:B5'); resultsTableStyle(Summary,'A6:B8');
  Graphs.getRange('A:A').format.columnWidth = 24;
  Graphs.getRange('B:C').format.columnWidth = 20;
  Graphs.getRange('D:D').format.columnWidth = 24;
  Graphs.getRange('E:E').format.columnWidth = 20;
  Graphs.getRange('A3:C3').values=[['Distance from bit','First mode Hz','Bending stress Pa']];
  Graphs.getRange('A3').formulas=[[`="Distance from bit "&'Unit Map'!$H$8`]];
  Graphs.getRange('C3').formulas=[[`="Bending stress "&'Unit Map'!$H$17`]];
  Graphs.getRange('A4:C9').formulas=Array.from({length:6},(_,i)=>[`=('BHA Assembly'!C${i+6}+'BHA Assembly'!D${i+6})/2`,`=Results!B${i+6}`,`=Calc!K${i+6}*'Unit Map'!$I$17`]);
  const modeDistanceChart=addScatterChart(Graphs,'A3:B9','BHA first mode versus distance from bit','E3','N16');
  modeDistanceChart.xAxis.title={text:'Distance from bit'}; modeDistanceChart.yAxis.title={text:'Natural frequency (Hz)'};
  Graphs.getRange('D3:E3').values=[['Distance from bit','Bending stress']]; Graphs.getRange('D3').formulas=[[`="Distance from bit "&'Unit Map'!$H$8`]]; Graphs.getRange('E3').formulas=[[`="Bending stress "&'Unit Map'!$H$17`]]; Graphs.getRange('D4:E9').formulas=Array.from({length:6},(_,i)=>[`=('BHA Assembly'!C${i+6}+'BHA Assembly'!D${i+6})/2`,`=Calc!K${i+6}*'Unit Map'!$I$17`]);
  const stressDistanceChart=addScatterChart(Graphs,'D3:E9','BHA bending stress versus distance from bit','E18','N33');
  stressDistanceChart.xAxis.title={text:'Distance from bit'}; stressDistanceChart.yAxis.title={text:'Bending stress'};
  Graphs.getRange('A20:D20').values=[['WOB 1 X','WOB 1 Y','WOB 2 X','WOB 2 Y']];
  Graphs.getRange('A21:D33').formulas=Array.from({length:13},(_,i)=>{const j=i===12?0:i; return [`=Results!I${j+6}`,`=Results!J${j+6}`,`=Results!L${j+6}`,`=Results!M${j+6}`];});
  Graphs.getRange('H35:J35').values=[['Toolface rad','WOB 1 magnitude','WOB 2 magnitude']]; Graphs.getRange('H35').formulas=[[`="Toolface "&'Unit Map'!$H$18`]];
  Graphs.getRange('H36:J47').formulas=Array.from({length:12},(_,i)=>[`=Results!G${i+6}`,`=Results!H${i+6}`,`=Results!K${i+6}`]);
  // PolarPlotter2010 construction: radar grid layer + true XY scatter data.
  Graphs.getRange('H65:L65').values=[['Angle','Ring 25%','Ring 50%','Ring 75%','Ring 100%']];
  Graphs.getRange('H66:L77').formulas=Array.from({length:12},(_,i)=>[`=${i*30}`,`=MAX($I$36:$J$47)*0.25`,`=MAX($I$36:$J$47)*0.50`,`=MAX($I$36:$J$47)*0.75`,`=MAX($I$36:$J$47)`]);
  const polarGrid = Graphs.charts.add('radar', Graphs.getRange('H65:L77'));
  polarGrid.title = 'Polar grid'; polarGrid.hasLegend = false; polarGrid.setPosition('E35','N53');
  // True polar geometry (0° at north), overlaid on the radar grid.
  addPolarScatterChart(Graphs, [
    { name:'WOB 1', xRange:"='Graphs'!$A$21:$A$33", yRange:"='Graphs'!$B$21:$B$33", lineColor:'#0F766E', transparency:35 },
    { name:'WOB 2', xRange:"='Graphs'!$C$21:$C$33", yRange:"='Graphs'!$D$21:$D$33", lineColor:'#D97706', transparency:35 },
  ], 'Toolface / WOB rose plot — true XY polar overlay', 'E35','N53', { xTitle:'East component', yTitle:'North component', scatterStyle:'lineMarker' });
  // Compact severity heatmap across components and the two selected WOB cases.
  Graphs.getRange('A55:C55').values=[['Component','WOB 1 tendency','WOB 2 tendency']];
  Graphs.getRange('A56:C61').formulas=Array.from({length:6},(_,i)=>[`=Results!A${i+6}`,`=Results!D${i+6}`,`=Results!E${i+6}`]);
  addHeatmapConditionalFormatting(Graphs.getRange('B56:C61'));
  tableHeader(Graphs,'A55:C55'); resultsTableStyle(Graphs,'A56:C61');

  sectionHeader(BhaAssembly,'A3:M3','Bottom-hole assembly component register and section properties');
  BhaAssembly.getRange('A5:M5').values=[['Record ID','Component','Top m','Bottom m','Length m','OD m','ID m','Area m2','I m4','Mass kg','Support factor','Connection / role','Status']];
  BhaAssembly.getRange('C5:H5').formulas=[[`="Top "&'Unit Map'!$H$8`,`="Bottom "&'Unit Map'!$H$8`,`="Length "&'Unit Map'!$H$8`,`="OD "&'Unit Map'!$H$9`,`="ID "&'Unit Map'!$H$9`,`="Area "&'Unit Map'!$H$10`]];
  for(let i=0;i<6;i+=1){const r=6+i; const ir=6+i; BhaAssembly.getRange(`A${r}:M${r}`).formulas=[[
    `='Inputs'!I${ir}`,`='Inputs'!D${ir}`,i===0?`=0*'Unit Map'!$I$8`:`=D${r-1}`,i===0?`='Inputs'!E${ir}*'Unit Map'!$I$8`:`=C${r}+'Inputs'!E${ir}*'Unit Map'!$I$8`,`='Inputs'!E${ir}*'Unit Map'!$I$8`,`='Inputs'!F${ir}*'Unit Map'!$I$9`,`='Inputs'!G${ir}*'Unit Map'!$I$9`,`=PI()/4*('Inputs'!F${ir}^2-'Inputs'!G${ir}^2)*'Unit Map'!$I$10`,`='Calc'!F${ir}`,`='Calc'!G${ir}`,`='Inputs'!H${ir}`,i===0?'="Bit / formation interface"':'="BHA component"',`=IF(AND('Inputs'!F${ir}>'Inputs'!G${ir},'Inputs'!E${ir}>0),"PASS","REVIEW")`,
  ]];}
  tableHeader(BhaAssembly,'A5:M5'); resultsTableStyle(BhaAssembly,'A6:M11');

  sectionHeader(VibrationModes,'A3:K3','Axial, lateral and torsional modal screening');
  VibrationModes.getRange('A4:K4').merge();
  VibrationModes.getRange('A4').values=[['Screening only: component beam frequencies and rotary orders are uncoupled; no rigid-body contact, damping calibration or forced-response amplitude is solved.']];
  VibrationModes.getRange('A4:K4').format={fill:'#FFF4D6',font:{italic:true,color:'#7C2D12'},wrapText:true};
  VibrationModes.getRange('A5:K5').values=[['Component','Mode','Natural frequency Hz','Operating excitation Hz','Frequency ratio','Separation %','Mass kg','Stiffness N/m','Damping ratio','Status','Record ID']];
  for(let i=0;i<18;i+=1){const r=6+i; const component=Math.floor(i/3); const mode=(i%3)+1; const cr=6+component; VibrationModes.getRange(`A${r}:K${r}`).formulas=[[
    `='Calc'!A${cr}`,`=${mode}`,`='Calc'!I${cr}*B${r}`,`='Inputs'!$B$5/60`,`=D${r}/C${r}`,`=ABS(C${r}-D${r})/MAX(C${r},0.000001)`,`='Calc'!G${cr}`,`='Calc'!H${cr}`,`=0.03+0.01*(B${r}-1)`,`=IF(F${r}<0.20,"REVIEW","CLEAR")`,`='Inputs'!I${cr}`,
  ]];}
  tableHeader(VibrationModes,'A5:K5'); resultsTableStyle(VibrationModes,'A6:K23'); addHeatmapConditionalFormatting(VibrationModes.getRange('E6:F23'));
  VibrationModes.getRange('M5:V5').values=[['RPM','1× rotary','3× rotary','5× rotary',...MOCK_CASE.bha.map(({name})=>`${name} first mode`)]];
  for(let i=0;i<10;i+=1){const row=6+i; const rpm=i*30; VibrationModes.getRange(`M${row}:V${row}`).formulas=[[
    `=${rpm}`,`=M${row}/60`,`=3*M${row}/60`,`=5*M${row}/60`,...Array.from({length:6},(_,component)=>`=Calc!I${component+6}`),
  ]];}
  VibrationModes.getRange('X5:AA5').values=[['Distance from bit','Mode 1','Mode 2','Mode 3']];
  VibrationModes.getRange('X5').formulas=[[`="Distance from bit "&'Unit Map'!$H$8`]];
  VibrationModes.getRange('X6:AA11').formulas=Array.from({length:6},(_,i)=>[
    `=('BHA Assembly'!C${i+6}+'BHA Assembly'!D${i+6})/2`,`=Calc!I${i+6}`,`=2*Calc!I${i+6}`,`=3*Calc!I${i+6}`,
  ]);
  tableHeader(VibrationModes,'M5:V5'); resultsTableStyle(VibrationModes,'M6:V15'); tableHeader(VibrationModes,'X5:AA5'); resultsTableStyle(VibrationModes,'X6:AA11');
  const campbell=addPolarScatterChart(VibrationModes,[
    {name:'1× rotary',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$N$6:$N$15",lineColor:'#2563EB'},
    {name:'3× rotary',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$O$6:$O$15",lineColor:'#D97706'},
    {name:'5× rotary',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$P$6:$P$15",lineColor:'#DC2626'},
    {name:'Motor / RSS first mode',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$S$6:$S$15",lineColor:'#0F766E'},
    {name:'MWD / LWD first mode',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$T$6:$T$15",lineColor:'#7C3AED'},
    {name:'Drill collar first mode',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$U$6:$U$15",lineColor:'#64748B'},
    {name:'HWDP transition first mode',xRange:"='Vibration Modes'!$M$6:$M$15",yRange:"='Vibration Modes'!$V$6:$V$15",lineColor:'#0891B2'},
  ],'Screening Campbell diagram — operational low-frequency modes','M18','V35',{xTitle:'Rotary speed (RPM)',yTitle:'Frequency (Hz)'});
  campbell.hasLegend=true;
  const modalProfile=addPolarScatterChart(VibrationModes,[
    {name:'Mode 1',xRange:"='Vibration Modes'!$X$8:$X$11",yRange:"='Vibration Modes'!$Y$8:$Y$11",lineColor:'#2563EB'},
    {name:'Mode 2',xRange:"='Vibration Modes'!$X$8:$X$11",yRange:"='Vibration Modes'!$Z$8:$Z$11",lineColor:'#D97706'},
    {name:'Mode 3',xRange:"='Vibration Modes'!$X$8:$X$11",yRange:"='Vibration Modes'!$AA$8:$AA$11",lineColor:'#0F766E'},
  ],'Component modal frequencies — low-frequency distance screening view','X18','AG35',{xTitle:'Distance from bit',yTitle:'Frequency (Hz)'});
  modalProfile.hasLegend=true;

  sectionHeader(BendingResponse,'A3:L3','Static bending response by BHA component');
  BendingResponse.getRange('A5:L5').values=[['Component','Length m','Bending moment N-m','Curvature 1/m','Outer fibre strain','Stress Pa','Estimated deflection m','WOB 1 tendency','WOB 2 tendency','Support factor','Status','Record ID']];
  BendingResponse.getRange('B5:C5').formulas=[[`="Length "&'Unit Map'!$H$8`,`="Bending moment "&'Unit Map'!$H$16`]];
  BendingResponse.getRange('F5:G5').formulas=[[`="Stress "&'Unit Map'!$H$17`,`="Estimated deflection "&'Unit Map'!$H$8`]];
  for(let i=0;i<6;i+=1){const r=6+i; const cr=6+i; BendingResponse.getRange(`A${r}:L${r}`).formulas=[[
    `='Calc'!A${cr}`,`='Calc'!B${cr}*'Unit Map'!$I$8`,`='Calc'!J${cr}*'Unit Map'!$I$16`,`='Calc'!J${cr}/(${youngModulusPa}*'Calc'!F${cr})`,`=('Calc'!J${cr}/(${youngModulusPa}*'Calc'!F${cr}))*'Calc'!C${cr}/2`,`='Calc'!K${cr}*'Unit Map'!$I$17`,`=('Calc'!J${cr}*'Calc'!B${cr}^2/(8*${youngModulusPa}*'Calc'!F${cr}))*'Unit Map'!$I$8`,`='Calc'!L${cr}`,`='Calc'!M${cr}`,`='Inputs'!H${cr}`,`=IF('Calc'!K${cr}>350000000,"REVIEW","PASS")`,`='Inputs'!I${cr}`,
  ]];}
  tableHeader(BendingResponse,'A5:L5'); resultsTableStyle(BendingResponse,'A6:L11'); addHeatmapConditionalFormatting(BendingResponse.getRange('F6:I11'));

  sectionHeader(BhaGeometry,'A3:P3','Static bending geometry projection and contact-screening indication');
  BhaGeometry.getRange('A5:P5').merge();
  BhaGeometry.getRange('A5').values=[['GEOMETRIC INTERFERENCE INDICATION — projected OD through the wellbore; not solved contact or reaction force.']];
  BhaGeometry.getRange('A6:P6').merge();
  BhaGeometry.getRange('A6').values=[['Centreline displacement is scaled by the explicit Inputs setting. Hole and BHA diameters remain dimensional; use this page to locate review regions, not to approve contact loads.']];
  BhaGeometry.getRange('A7:P7').merge();
  BhaGeometry.getRange('A7').values=[['A future rigid-body/FE engine must solve equilibrium, contact reactions, friction and coupled modes before any force claim is made.']];
  BhaGeometry.getRange('A5:P7').format={fill:'#FFF4D6',font:{bold:true,color:'#7C2D12'},wrapText:true};
  BhaGeometry.getRange('A9:H9').values=[['Decision metric','Value','Status','Interpretation','','','','']];
  BhaGeometry.getRange('A10:D13').values=[['Projection plane','','',''],['Display deflection scale','','',''],['Minimum projected clearance','','',''],['Overlap indication count','','','']];
  BhaGeometry.getRange('B10').formulas=[['=Inputs!B12']]; BhaGeometry.getRange('B11').formulas=[['=Inputs!B13']];
  BhaGeometry.getRange('B12').formulas=[[`=MIN(Calc!AB6:AB35)`]]; BhaGeometry.getRange('C12').formulas=[[`='Unit Map'!$H$9`]];
  BhaGeometry.getRange('B13').formulas=[['=COUNTIF(Calc!AD6:AD35,"OVERLAP INDICATION")']];
  BhaGeometry.getRange('C13').formulas=[['=IF(B13>0,"REVIEW","CLEAR")']];
  BhaGeometry.getRange('D10:D13').values=[['Selected projection sign only'],['Applied to estimated centreline displacement'],['Negative means projected OD crosses the hole wall'],['Indication only; no reaction or contact force calculated']];
  tableHeader(BhaGeometry,'A9:D9'); resultsTableStyle(BhaGeometry,'A10:D13'); addHeatmapConditionalFormatting(BhaGeometry.getRange('B12:B13'));
  BhaGeometry.getRange('B11').format.numberFormat='0.0000';
  BhaGeometry.getRange('B12').format.numberFormat='0.000';
  const geometryChart=addPolarScatterChart(BhaGeometry,[
    {name:'Hole high wall',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$V$6:$V$35",lineColor:'#334155'},
    {name:'Hole low wall',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$W$6:$W$35",lineColor:'#334155'},
    {name:'BHA OD high',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$X$6:$X$35",lineColor:'#D97706'},
    {name:'BHA OD low',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$Y$6:$Y$35",lineColor:'#D97706'},
    {name:'BHA ID high',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$Z$6:$Z$35",lineColor:'#0F766E'},
    {name:'BHA ID low',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$AA$6:$AA$35",lineColor:'#0F766E'},
    {name:'Estimated centreline',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$U$6:$U$35",lineColor:'#2563EB'},
  ],'BHA geometry projection — hole, OD and ID','A15','H31',{xTitle:'Distance from bit',yTitle:'Projected lateral position'});
  geometryChart.hasLegend=true;
  addPolarScatterChart(BhaGeometry,[
    {name:'Projected radial clearance',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$AB$6:$AB$35",lineColor:'#0F766E'},
    {name:'Zero clearance',xRange:"='Calc'!$Q$6:$Q$35",yRange:"='Calc'!$AC$6:$AC$35",lineColor:'#DC2626'},
  ],'Projected radial clearance — indication only','I15','P31',{xTitle:'Distance from bit',yTitle:'Radial clearance'});
  addPolarScatterChart(BhaGeometry,[{name:'Bending moment',xRange:"='Calc'!$AE$6:$AE$35",yRange:"='Calc'!$AF$6:$AF$35",lineColor:'#D97706'}],
    'Bending moment versus distance from bit','A33','H49',{xTitle:'Distance from bit',yTitle:'Bending moment'});
  addPolarScatterChart(BhaGeometry,[
    {name:'Bending stress',xRange:"='Calc'!$AE$6:$AE$35",yRange:"='Calc'!$AG$6:$AG$35",lineColor:'#2563EB'},
    {name:'Screening limit',xRange:"='Calc'!$AE$6:$AE$35",yRange:"='Calc'!$AH$6:$AH$35",lineColor:'#DC2626'},
  ],'Bending stress versus distance from bit','I33','P49',{xTitle:'Distance from bit',yTitle:'Bending stress'});
  BhaGeometry.getRange('A:A').format.columnWidth=25; BhaGeometry.getRange('B:C').format.columnWidth=18; BhaGeometry.getRange('D:D').format.columnWidth=45;

  sectionHeader(TendencyMatrix,'A3:H3','Toolface/WOB drill-ahead tendency matrix');
  TendencyMatrix.getRange('A5:H5').values=[['Toolface deg','Toolface rad','WOB 1 N','WOB 1 tendency','WOB 2 N','WOB 2 tendency','Build/turn quadrant','Status']];
  TendencyMatrix.getRange('B5:C5').formulas=[[`="Toolface "&'Unit Map'!$H$18`,`="WOB 1 "&'Unit Map'!$H$14`]];
  TendencyMatrix.getRange('E5').formulas=[[`="WOB 2 "&'Unit Map'!$H$14`]];
  for(let i=0;i<12;i+=1){const r=6+i; TendencyMatrix.getRange(`A${r}:H${r}`).formulas=[[
    `=${i*30}`,`=RADIANS(A${r})*'Unit Map'!$I$18`,`='Inputs'!$B$9*'Unit Map'!$I$14`,`=AVERAGE('Results'!$D$6:$D$11)*(1+0.35*COS(RADIANS(A${r})))`,`='Inputs'!$B$10*'Unit Map'!$I$14`,`=AVERAGE('Results'!$E$6:$E$11)*(1+0.35*COS(RADIANS(A${r})))`,`=IF(A${r}<90,"BUILD / RIGHT",IF(A${r}<180,"DROP / RIGHT",IF(A${r}<270,"DROP / LEFT","BUILD / LEFT")))`,`=IF(MAX(D${r},F${r})>0.05,"REVIEW","SCREEN")`,
  ]];}
  tableHeader(TendencyMatrix,'A5:H5'); resultsTableStyle(TendencyMatrix,'A6:H17'); addHeatmapConditionalFormatting(TendencyMatrix.getRange('D6:F17'));

  sectionHeader(PolarPlot,'A3:N3','PolarPlotter-style toolface response — radar grid plus XY data');
  PolarPlot.getRange('A5:E5').values=[['Toolface deg','WOB 1 X','WOB 1 Y','WOB 2 X','WOB 2 Y']];
  PolarPlot.getRange('A6:E18').formulas=Array.from({length:13},(_,i)=>{const j=i===12?0:i; const r=6+j; return [`='Tendency Matrix'!A${r}`,`='Tendency Matrix'!D${r}*SIN('Tendency Matrix'!B${r})`,`='Tendency Matrix'!D${r}*COS('Tendency Matrix'!B${r})`,`='Tendency Matrix'!F${r}*SIN('Tendency Matrix'!B${r})`,`='Tendency Matrix'!F${r}*COS('Tendency Matrix'!B${r})`];});
  PolarPlot.getRange('H5:L5').values=[['Angle','Ring 25%','Ring 50%','Ring 75%','Ring 100%']]; PolarPlot.getRange('H6:L17').formulas=Array.from({length:12},(_,i)=>[`=${i*30}`,`=MAX($B$6:$E$18)*0.25`,`=MAX($B$6:$E$18)*0.50`,`=MAX($B$6:$E$18)*0.75`,`=MAX($B$6:$E$18)`]);
  const polarBase=PolarPlot.charts.add('radar',PolarPlot.getRange('H5:L17')); polarBase.title='Polar grid'; polarBase.hasLegend=false; polarBase.setPosition('G20','N37');
  // Keep the XY overlay in a distinct drawing anchor.  Identical anchors can
  // be serialized as an invalid radar/scatter combination by the exporter.
  addPolarScatterChart(PolarPlot,[{name:'WOB 1',xRange:"='Polar Plot'!$B$6:$B$18",yRange:"='Polar Plot'!$C$6:$C$18",lineColor:'#0F766E',transparency:35},{name:'WOB 2',xRange:"='Polar Plot'!$D$6:$D$18",yRange:"='Polar Plot'!$E$6:$E$18",lineColor:'#D97706',transparency:35}],'WOB/toolface polar response','G20','N38',{xTitle:'East component',yTitle:'North component'});

  sectionHeader(RustEngine,'A3:H3','Rust BHA engine contract, source identity and execution state');
  RustEngine.getRange('A5:B13').values=[
    ['Contract version','1.0.0'],['Engine executable','wellforge-bha.exe'],['Execution mode','RUST REQUIRED — NO VBA FALLBACK'],
    ['Request path',''],['Result path',''],['Request SHA-256',''],['Result SHA-256',''],['Engine version',RUST_FIXTURE.evidence.engine_version],['State','FIXTURE PREVIEW'],
  ];
  tableHeader(RustEngine,'A5:B5'); resultsTableStyle(RustEngine,'A6:B13');
  RustEngine.getRange('D5:I5').values=[['Object type','UUID','URI','Content hash','Citation','Source system']];
  RustEngine.getRange('D6:I11').values=RUST_FIXTURE.sources.map(source=>[source.object_type,source.uuid,source.uri,source.content_hash,source.citation_name,source.source_system]);
  tableHeader(RustEngine,'D5:I5'); resultsTableStyle(RustEngine,'D6:I11');
  RustEngine.getRange('K5:K11').values=[['Component UUID'],...Array.from({length:6},(_,index)=>[`00000000-0000-0000-0000-${(1000+index).toString().padStart(12,'0')}`])];
  tableHeader(RustEngine,'K5:K5'); resultsTableStyle(RustEngine,'K6:K11');
  RustEngine.getRange('A15:I17').merge(); RustEngine.getRange('A15').values=[['The checked-in values on Rust Engine Results are a synthetic deterministic fixture preview. Desktop Excel replaces them with value-only results from the colocated, hash-verified Rust executable. Projected negative clearance is an interference indication only; Release 1 does not report contact force.']];
  RustEngine.getRange('A15:I17').format={fill:'#FFF4D6',font:{color:'#7C2D12',bold:true},wrapText:true};
  RustEngine.getRange('A:A').format.columnWidth=26; RustEngine.getRange('B:B').format.columnWidth=34;
  RustEngine.getRange('D:D').format.columnWidth=22; RustEngine.getRange('E:F').format.columnWidth=38;
  RustEngine.getRange('G:G').format.columnWidth=34; RustEngine.getRange('H:I').format.columnWidth=26; RustEngine.getRange('K:K').format.columnWidth=40;

  sectionHeader(RustResults,'A3:N3','Rust Release 1 — static projection, bending, modes, FRF and Campbell results');
  RustResults.getRange('A5:D5').values=[['Decision metric','Value','Unit','State']];
  const minClearance=Math.min(...RUST_FIXTURE.static_nodes.map(node=>node.projected_clearance_m));
  const peakStress=Math.max(...RUST_FIXTURE.static_nodes.map(node=>node.bending_stress_pa));
  RustResults.getRange('A6:D10').values=[
    ['Minimum projected clearance',minClearance,'m',minClearance<0?'REVIEW':'CLEAR'],
    ['Peak bending stress',peakStress,'Pa','CALCULATED'],
    ['First lateral natural frequency',RUST_FIXTURE.modes[0].natural_frequency_hz,'Hz','CALCULATED'],
    ['Nearest 1x–3x modal margin',Math.min(...RUST_FIXTURE.campbell.map(point=>point.nearest_mode_margin_hz)),'Hz','REVIEW'],
    ['Contact force','Not calculated','—','INDICATION ONLY'],
  ];
  tableHeader(RustResults,'A5:D5'); resultsTableStyle(RustResults,'A6:D10'); addHeatmapConditionalFormatting(RustResults.getRange('B6:B9'));
  RustResults.getRange('A12:D12').values=[['Mode','Natural frequency Hz','Critical speed RPM','Status']];
  RustResults.getRange(`A13:D${12+RUST_FIXTURE.modes.length}`).values=RUST_FIXTURE.modes.map(mode=>[mode.mode_number,mode.natural_frequency_hz,mode.critical_speed_rpm,'CALCULATED']);
  tableHeader(RustResults,'A12:D12'); resultsTableStyle(RustResults,`A13:D${12+RUST_FIXTURE.modes.length}`);
  RustResults.getRange('A22:N24').merge(); RustResults.getRange('A22').values=[['Applicability: linear small-deflection lateral beam model with buoyancy and compressive geometric stiffness. OD/hole crossing is an interference indication, not a solved normal-contact reaction. DAT calibration and nonlinear impact dynamics are outside Release 1.']];
  RustResults.getRange('A22:N24').format={fill:'#FFF4D6',font:{color:'#7C2D12',bold:true},wrapText:true};
  sectionHeader(RustCalc,'A3:O3','Value-only Rust result arrays used by charts');
  RustCalc.getRange('A5:K5').values=[['MD m','Centreline X m','Centreline Y m','OD radius m','ID radius m','Hole radius m','Projected clearance m','Bending moment N.m','Bending stress Pa','State','Hole low m']];
  const staticRows=RUST_FIXTURE.static_nodes.map(node=>[node.md_m,node.x_m,node.y_m,node.od_radius_m,node.id_radius_m,node.hole_radius_m,node.projected_clearance_m,node.bending_moment_n_m,node.bending_stress_pa,node.projected_clearance_m<0?'OVERLAP INDICATION':'CLEARANCE',-node.hole_radius_m]);
  RustCalc.getRange(`A6:K${5+staticRows.length}`).values=staticRows;
  tableHeader(RustCalc,'A5:K5'); resultsTableStyle(RustCalc,`A6:K${5+staticRows.length}`); addHeatmapConditionalFormatting(RustCalc.getRange(`G6:G${5+staticRows.length}`));
  RustCalc.getRange('L5:O5').values=[['Mode','Natural frequency Hz','Critical speed RPM','Status']];
  RustCalc.getRange(`L6:O${5+RUST_FIXTURE.modes.length}`).values=RUST_FIXTURE.modes.map(mode=>[mode.mode_number,mode.natural_frequency_hz,mode.critical_speed_rpm,'CALCULATED']);
  tableHeader(RustCalc,'L5:O5'); resultsTableStyle(RustCalc,`L6:O${5+RUST_FIXTURE.modes.length}`);
  const modeShapeStart=250;
  RustCalc.getRange(`L${modeShapeStart}:O${modeShapeStart}`).values=[['MD m','Mode 1','Mode 2','Mode 3']];
  const shapeRows=RUST_FIXTURE.static_nodes.map((node,index)=>[node.md_m,...[0,1,2].map(mode=>RUST_FIXTURE.modes[mode]?.normalized_shape[index]??null)]);
  RustCalc.getRange(`L${modeShapeStart+1}:O${modeShapeStart+shapeRows.length}`).values=shapeRows;
  tableHeader(RustCalc,`L${modeShapeStart}:O${modeShapeStart}`); resultsTableStyle(RustCalc,`L${modeShapeStart+1}:O${modeShapeStart+shapeRows.length}`);
  const frfStart=40;
  RustCalc.getRange(`A${frfStart}:C${frfStart}`).values=[['Frequency Hz','Receptance m/N','Phase deg']];
  RustCalc.getRange(`A${frfStart+1}:C${frfStart+RUST_FIXTURE.frequency_response.length}`).values=RUST_FIXTURE.frequency_response.map(point=>[point.frequency_hz,point.receptance_m_n,point.phase_deg]);
  tableHeader(RustCalc,`A${frfStart}:C${frfStart}`); resultsTableStyle(RustCalc,`A${frfStart+1}:C${frfStart+RUST_FIXTURE.frequency_response.length}`);
  const campbellStart=40;
  RustCalc.getRange(`E${campbellStart}:H${campbellStart}`).values=[['Order','RPM','Excitation Hz','Nearest-mode margin Hz']];
  RustCalc.getRange(`E${campbellStart+1}:H${campbellStart+RUST_FIXTURE.campbell.length}`).values=RUST_FIXTURE.campbell.map(point=>[point.order,point.rpm,point.excitation_frequency_hz,point.nearest_mode_margin_hz]);
  tableHeader(RustCalc,`E${campbellStart}:H${campbellStart}`); resultsTableStyle(RustCalc,`E${campbellStart+1}:H${campbellStart+RUST_FIXTURE.campbell.length}`); addHeatmapConditionalFormatting(RustCalc.getRange(`H${campbellStart+1}:H${campbellStart+RUST_FIXTURE.campbell.length}`));
  const staticEnd=5+staticRows.length;
  const rustModeChart=addPolarScatterChart(RustResults,[
    {name:'Hole high',xRange:`='Rust Calc'!$A$6:$A$${staticEnd}`,yRange:`='Rust Calc'!$F$6:$F$${staticEnd}`,lineColor:'#334155'},
    {name:'Hole low',xRange:`='Rust Calc'!$A$6:$A$${staticEnd}`,yRange:`='Rust Calc'!$K$6:$K$${staticEnd}`,lineColor:'#334155'},
    {name:'Centreline',xRange:`='Rust Calc'!$A$6:$A$${staticEnd}`,yRange:`='Rust Calc'!$B$6:$B$${staticEnd}`,lineColor:'#2563EB'},
  ],'Rust static centreline and hole radius','F5','N19',{xTitle:'MD (m)',yTitle:'Lateral position / radius (m)'});
  addPolarScatterChart(RustResults,[
    {name:'Mode 1',xRange:`='Rust Calc'!$M$${modeShapeStart+1}:$M$${modeShapeStart+shapeRows.length}`,yRange:`='Rust Calc'!$L$${modeShapeStart+1}:$L$${modeShapeStart+shapeRows.length}`,lineColor:'#2563EB'},
    {name:'Mode 2',xRange:`='Rust Calc'!$N$${modeShapeStart+1}:$N$${modeShapeStart+shapeRows.length}`,yRange:`='Rust Calc'!$L$${modeShapeStart+1}:$L$${modeShapeStart+shapeRows.length}`,lineColor:'#D97706'},
    {name:'Mode 3',xRange:`='Rust Calc'!$O$${modeShapeStart+1}:$O$${modeShapeStart+shapeRows.length}`,yRange:`='Rust Calc'!$L$${modeShapeStart+1}:$L$${modeShapeStart+shapeRows.length}`,lineColor:'#0F766E'},
  ],'Rust lateral mode shapes — MD increases downward','F21','N35',{xTitle:'Normalized lateral amplitude',yTitle:'MD (m)'});
  rustModeChart.yAxis.orientation='maxMin';
  addPolarScatterChart(RustResults,[{name:'Receptance',xRange:`='Rust Calc'!$A$${frfStart+1}:$A$${frfStart+RUST_FIXTURE.frequency_response.length}`,yRange:`='Rust Calc'!$B$${frfStart+1}:$B$${frfStart+RUST_FIXTURE.frequency_response.length}`,lineColor:'#7C3AED'}],
    'Direct frequency response — unit lateral force','A26','E40',{xTitle:'Frequency (Hz)',yTitle:'Receptance (m/N)'});
  RustResults.getRange('A:A').format.columnWidth=32; RustResults.getRange('B:D').format.columnWidth=18;
  RustCalc.getRange('A:A').format.columnWidth=16; RustCalc.getRange('B:K').format.columnWidth=18; RustCalc.getRange('L:O').format.columnWidth=18;
  addExchangeSheets(workbook, 'bha');
  applyTwoDecimalDisplayPrecision(workbook);
  Inputs.getRange('B13').format.numberFormat='0.0000';
  BhaGeometry.getRange('B11').format.numberFormat='0.0000';
  BhaGeometry.getRange('B12').format.numberFormat='0.000';
  return workbook;
}

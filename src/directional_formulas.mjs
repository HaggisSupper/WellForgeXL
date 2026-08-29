const q = (sheet, cell) => `'${sheet}'!${cell}`;

export function doglegAngleFormula(row) {
  const previous = row - 1;
  return `=ACOS(MAX(-1,MIN(1,COS(D${previous})*COS(D${row})+SIN(D${previous})*SIN(D${row})*COS(MOD(E${row}-E${previous}+PI(),2*PI())-PI()))))`;
}

export function ratioFactorFormula(row) {
  return `=IF(ABS(G${row})<1E-9,1+G${row}^2/12+G${row}^4/120,2*TAN(G${row}/2)/G${row})`;
}

export function deltaTvdFormula(row) { return `=F${row}/2*(COS(D${row - 1})+COS(D${row}))*H${row}`; }
export function deltaNorthFormula(row) { return `=F${row}/2*(SIN(D${row - 1})*COS(E${row - 1})+SIN(D${row})*COS(E${row}))*H${row}`; }
export function deltaEastFormula(row) { return `=F${row}/2*(SIN(D${row - 1})*SIN(E${row - 1})+SIN(D${row})*SIN(E${row}))*H${row}`; }
export function doglegSeverityFormula(row) { return `=IF(F${row}>0,G${row}/F${row},"")`; }

function slerpFormula(row, component, priorComponent, nextComponent) {
  const beta = q('Calc', `$G${row}`);
  const fraction = q('Calc', `$AA${row}`);
  const pN = q('Calc', `$AB${row}`); const pE = q('Calc', `$AC${row}`); const pV = q('Calc', `$AD${row}`);
  const nN = q('Calc', `$AE${row}`); const nE = q('Calc', `$AF${row}`); const nV = q('Calc', `$AG${row}`);
  const blend = (a, b) => `((1-${fraction})*${a}+${fraction}*${b})`;
  const norm = `SQRT(${blend(pN, nN)}^2+${blend(pE, nE)}^2+${blend(pV, nV)}^2)`;
  const spherical = `(SIN((1-${fraction})*${beta})/SIN(${beta}))*${priorComponent}+(SIN(${fraction}*${beta})/SIN(${beta}))*${nextComponent}`;
  return `=IF(ABS(${beta})<1E-9,${blend(priorComponent, nextComponent)}/${norm},${spherical})`;
}

export function slerpNorthFormula(row) { return slerpFormula(row, 'north', q('Calc', `$AB${row}`), q('Calc', `$AE${row}`)); }
export function slerpEastFormula(row) { return slerpFormula(row, 'east', q('Calc', `$AC${row}`), q('Calc', `$AF${row}`)); }
export function slerpVerticalFormula(row) { return slerpFormula(row, 'vertical', q('Calc', `$AD${row}`), q('Calc', `$AG${row}`)); }

export function partialPositionFormula(row) {
  return `=${q('Calc', `$M${row - 1}`)}+${q('Calc', `$AH${row}`)}/2*(${q('Calc', `$AB${row}`)}+${q('Calc', `$AI${row}`)})*${q('Calc', `$AJ${row}`)}`;
}

export function crosslineErrorFormula(row) {
  return `=-${q('Calc', `$Q${row}`)}*SIN(${q('Inputs', '$B$16')})+${q('Calc', `$R${row}`)}*COS(${q('Inputs', '$B$16')})`;
}

export function error3dFormula(row) { return `=SQRT(${q('Calc', `$Q${row}`)}^2+${q('Calc', `$R${row}`)}^2+${q('Calc', `$P${row}`)}^2)`; }

export function effectiveTurnFormula(row) {
  return `=IF(F${row}>0,(MOD(E${row}-E${row - 1}+PI(),2*PI())-PI())*SIN((D${row - 1}+D${row})/2)/F${row},"")`;
}

export function responseToolfaceFormula(row) {
  return `=MOD(ATAN2(${q('Calc', `$P${row}`)},${q('Calc', `$Q${row}`)}),2*PI())`;
}

export function targetEnvelopeFormula(row) {
  const s = (cell) => q('Targets', `$${cell}${row}`);
  return `=LET(dN,${q('Calc', `$AN${row}`)}-${s('C')},dE,${q('Calc', `$AO${row}`)}-${s('D')},theta,${s('I')},localMajor,dN*COS(theta)+dE*SIN(theta),localMinor,-dN*SIN(theta)+dE*COS(theta),SWITCH(${s('F')},"Point",SQRT(dN^2+dE^2)/${s('G')},"Circle",SQRT(dN^2+dE^2)/${s('G')},"Ellipse",SQRT((localMajor/${s('G')})^2+(localMinor/${s('H')})^2),"Box",MAX(ABS(localMajor)/${s('G')},ABS(localMinor)/${s('H')}),NA()))`;
}

export function formationHighLowFormula(row) {
  return `=IF(OR(${q('Formation Tops', `$C${row}`)}="",${q('Formation Tops', `$G${row}`)}=""),"",${q('Formation Tops', `$C${row}`)}-${q('Formation Tops', `$G${row}`)})`;
}

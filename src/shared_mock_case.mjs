import { directionalReferenceData } from './directional_reference_data.mjs';

const TUBULAR = Object.freeze({
  drillPipe: Object.freeze({ lengthM: 2100, odM: 0.127, idM: 0.1086 }),
  hwdp: Object.freeze({ lengthM: 300, odM: 0.14, idM: 0.076 }),
  drillCollar: Object.freeze({ lengthM: 210, odM: 0.171, idM: 0.071 }),
  mwdLwd: Object.freeze({ lengthM: 30, odM: 0.171, idM: 0.057 }),
  motorRss: Object.freeze({ lengthM: 20, odM: 0.171, idM: 0.057 }),
  bitSub: Object.freeze({ lengthM: 10, odM: 0.171, idM: 0.051 }),
});

const HOLE_SECTIONS = Object.freeze([
  Object.freeze({ id: 'hole-01', name: '8-1/2 in open hole', topMdM: 0, bottomMdM: 2660, holeIdM: 0.216 }),
]);

export function buildHydraulicsFlowPath({ tubular, holeSections }) {
  const openHole = holeSections.find(({ id }) => id === 'hole-01');
  return Object.freeze([
    Object.freeze({ id: 'flow-standpipe', name: 'Standpipe', lengthM: 35, flowIdM: 0.102, hydraulicDiameterM: 0.102, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-rotary-hose', name: 'Rotary hose', lengthM: 18, flowIdM: 0.076, hydraulicDiameterM: 0.076, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-top-drive', name: 'Top drive / swivel', lengthM: 8, flowIdM: 0.076, hydraulicDiameterM: 0.076, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-drill-pipe', name: 'Drill pipe', lengthM: tubular.drillPipe.lengthM, flowIdM: tubular.drillPipe.idM, hydraulicDiameterM: tubular.drillPipe.idM, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-hwdp', name: 'HWDP', lengthM: tubular.hwdp.lengthM, flowIdM: tubular.hwdp.idM, hydraulicDiameterM: tubular.hwdp.idM, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-drill-collar', name: 'Drill collar', lengthM: tubular.drillCollar.lengthM, flowIdM: tubular.drillCollar.idM, hydraulicDiameterM: tubular.drillCollar.idM, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-motor-mwd', name: 'Motor / MWD', lengthM: tubular.motorRss.lengthM + tubular.mwdLwd.lengthM, flowIdM: tubular.motorRss.idM, hydraulicDiameterM: tubular.motorRss.idM, flowType: 'Pipe' }),
    Object.freeze({ id: 'flow-open-hole-annulus', name: 'Open-hole annulus', lengthM: openHole.bottomMdM - openHole.topMdM, flowIdM: tubular.drillCollar.odM, hydraulicDiameterM: 0.044, flowType: 'Annulus' }),
  ]);
}

// Suite-wide mock operating case.  Values are canonical SI and are used only
// where the same physical item or operating condition appears in a workbook.
export const MOCK_CASE = Object.freeze({
  fluid: Object.freeze({ densityKgM3: 1200, apparentViscosityPaS: 0.035 }),
  // Conventional drill-string steel modulus: 30.00 Mpsi = 206.842718795 GPa.
  material: Object.freeze({ steelDensityKgM3: 7850, youngModulusPa: 206842718795.04 }),
  hydraulics: Object.freeze({
    flowRateM3S: 0.044,
    surfacePressureLimitPa: 35000000,
    flowPath: buildHydraulicsFlowPath({ tubular: TUBULAR, holeSections: HOLE_SECTIONS }),
  }),
  operation: Object.freeze({ wobN: 120000, lowWobN: 80000, surfaceTorqueNm: 26000, frictionFactor: 0.24, rotarySpeedRpm: 120 }),
  tubular: TUBULAR,
  holeSections: HOLE_SECTIONS,
  bha: Object.freeze([
    Object.freeze({ id: 'bha-bit', name: 'Bit', lengthM: 0.3, odM: 0.216, idM: 0.05, supportFactor: 1 }),
    Object.freeze({ id: 'bha-near-bit-stabilizer', name: 'Near-bit stabilizer', lengthM: 1.5, odM: 0.203, idM: 0.07, supportFactor: 0.85 }),
    Object.freeze({ id: 'bha-motor-rss', name: 'Motor / RSS', lengthM: 8, odM: 0.171, idM: 0.057, supportFactor: 0.7 }),
    Object.freeze({ id: 'bha-mwd-lwd', name: 'MWD / LWD', lengthM: 12, odM: 0.171, idM: 0.057, supportFactor: 0.65 }),
    Object.freeze({ id: 'bha-drill-collar', name: 'Drill collar', lengthM: 90, odM: 0.171, idM: 0.071, supportFactor: 0.55 }),
    Object.freeze({ id: 'bha-hwdp-transition', name: 'HWDP transition', lengthM: 60, odM: 0.14, idM: 0.076, supportFactor: 0.4 }),
  ]),
  rig: Object.freeze({ preset: 'Land rig', pumpEfficiency: 0.92, ecdScreenDensityKgM3: 1400, hookloadLimitN: 1050000 }),
  pumpNozzle: Object.freeze({
    nozzleCount: 3,
    dischargeCoefficient: 0.95,
    baseNozzleId: 'nozzle-010',
    nozzles: Object.freeze([
      Object.freeze({ id: 'nozzle-008', diameterM: 0.008 }),
      Object.freeze({ id: 'nozzle-009', diameterM: 0.009 }),
      Object.freeze({ id: 'nozzle-010', diameterM: 0.01 }),
      Object.freeze({ id: 'nozzle-011', diameterM: 0.011 }),
      Object.freeze({ id: 'nozzle-012', diameterM: 0.012 }),
    ]),
  }),
  api7g: Object.freeze({
    designUtilisationLimit: 0.9,
    sections: Object.freeze([
      Object.freeze({ id: 'api7g-dp-01', name: 'Drill Pipe', tubularKey: 'drillPipe', axialLoadN: 850000, tensionLimitN: 1800000, operatingTorqueNm: 22000 }),
      Object.freeze({ id: 'api7g-hwdp-01', name: 'HWDP', tubularKey: 'hwdp', axialLoadN: 920000, tensionLimitN: 2300000, operatingTorqueNm: 30000 }),
      Object.freeze({ id: 'api7g-dc-01', name: 'Drill Collar', tubularKey: 'drillCollar', axialLoadN: 980000, tensionLimitN: 3000000, operatingTorqueNm: 45000 }),
      Object.freeze({ id: 'api7g-mwd-lwd-01', name: 'MWD / LWD', tubularKey: 'mwdLwd', axialLoadN: 1000000, tensionLimitN: 2200000, operatingTorqueNm: 35000 }),
      Object.freeze({ id: 'api7g-motor-rss-01', name: 'Motor / RSS', tubularKey: 'motorRss', axialLoadN: 1020000, tensionLimitN: 2200000, operatingTorqueNm: 35000 }),
      Object.freeze({ id: 'api7g-bit-sub-01', name: 'Bit / Sub', tubularKey: 'bitSub', axialLoadN: 1050000, tensionLimitN: 1600000, operatingTorqueNm: 30000 }),
    ]),
  }),
  surveyStations: Object.freeze(directionalReferenceData.survey.map((station) => Object.freeze({
    id: `survey-${String(station.station).padStart(3, '0')}`,
    station: station.station,
    mdM: station.mdFt * 0.3048,
    inclinationRad: station.incDeg * Math.PI / 180,
    azimuthRad: station.aziDeg * Math.PI / 180,
    holeIdM: 0.216,
  }))),
});

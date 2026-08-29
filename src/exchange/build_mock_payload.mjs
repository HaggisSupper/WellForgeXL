import fs from 'node:fs/promises';
import { directionalReferenceData } from '../directional_reference_data.mjs';
import { MOCK_CASE } from '../shared_mock_case.mjs';
import { SCHEMA_VERSION, quantity } from './schema_contract.mjs';
import { validateExchangePayload } from './schema_validator.mjs';

const formulaAnalysis = () => ({
  calculationState: 'notCalculated',
  method: 'WellForge workbook formula model',
  results: [],
});

const stationId = (prefix, station) => `${prefix}-${String(station).padStart(3, '0')}`;
const TUBULAR_IDS = Object.freeze({ drillPipe: 'dp-01', hwdp: 'hwdp-01', drillCollar: 'dc-01', mwdLwd: 'mwd-lwd-01', motorRss: 'motor-rss-01', bitSub: 'bit-sub-01' });
const payloadTrajectoryIds = (kind) => kind === 'survey'
  ? MOCK_CASE.surveyStations.map(({ id }) => id)
  : directionalReferenceData[kind].map(({ station }) => stationId(kind, station));

function referenceMetadata() {
  return Object.fromEntries(directionalReferenceData.metadata.map(({ key, value }) => [key, value]));
}

function buildTrajectory() {
  return {
    plan: directionalReferenceData.plan.map((station) => ({
      id: stationId('plan', station.station),
      station: station.station,
      md: quantity(station.mdFt, 'ft'),
      inclination: quantity(station.incDeg, 'deg'),
      azimuth: quantity(station.aziDeg, 'deg'),
      source: 'Directional reference plan',
    })),
    survey: directionalReferenceData.survey.map((station, index) => ({
      id: MOCK_CASE.surveyStations[index].id,
      station: station.station,
      md: quantity(station.mdFt, 'ft'),
      inclination: quantity(station.incDeg, 'deg'),
      azimuth: quantity(station.aziDeg, 'deg'),
      source: 'Directional reference survey',
      holeDiameter: quantity(MOCK_CASE.surveyStations[index].holeIdM, 'm'),
    })),
    targets: directionalReferenceData.targets.map((target, index) => ({
      id: target.id,
      name: target.id,
      md: quantity([6125, 11125, 16125][index], 'ft'),
      centerTvd: quantity(target.centerTvdFt, 'ft'),
      centerNorth: quantity(target.centerNorthFt, 'ft'),
      centerEast: quantity(target.centerEastFt, 'ft'),
      radius: quantity(target.radiusFt, 'ft'),
      type: 'Circle',
      major: quantity(target.radiusFt, 'ft'),
      minor: quantity(target.radiusFt, 'ft'),
      rotation: quantity(0, 'deg'),
      verticalTolerance: quantity(target.radiusFt, 'ft'),
      entryInclination: quantity(target.entryIncDeg, 'deg'),
      entryAzimuth: quantity(target.entryAziDeg, 'deg'),
      note: target.note,
    })),
    slideIntervals: directionalReferenceData.slideIntervals.map((interval) => ({
      id: `slide-${String(interval.stand).padStart(2, '0')}`,
      stand: interval.stand,
      date: quantity(interval.dateSerial, 'd'),
      mdIn: quantity(interval.mdInFt, 'ft'),
      mdOut: quantity(interval.mdOutFt, 'ft'),
      slideLength: quantity(interval.slideFt, 'ft'),
      rotateLength: quantity(interval.rotateFt, 'ft'),
      commandedToolface: quantity(interval.commandedToolfaceDeg, 'deg'),
      source: 'Sanitized reference fixture',
    })),
    formationTops: directionalReferenceData.formationTops.map((top) => ({
      id: top.id,
      name: top.name,
      prognosedMd: quantity(top.prognosedMdFt, 'ft'),
      prognosedTvd: quantity(top.prognosedTvdFt, 'ft'),
      localDip: quantity(top.localDipDeg, 'deg'),
      note: `Local dip ${top.localDipDeg} deg; sanitized reference fixture`,
    })),
  };
}

function buildTubulars() {
  const records = [
    ['dp-01', 'Drill Pipe', MOCK_CASE.tubular.drillPipe],
    ['hwdp-01', 'HWDP', MOCK_CASE.tubular.hwdp],
    ['dc-01', 'Drill Collar', MOCK_CASE.tubular.drillCollar],
    ['mwd-lwd-01', 'MWD / LWD', MOCK_CASE.tubular.mwdLwd],
    ['motor-rss-01', 'Motor / RSS', MOCK_CASE.tubular.motorRss],
    ['bit-sub-01', 'Bit / Sub', MOCK_CASE.tubular.bitSub],
  ];
  return records.map(([id, name, tubular]) => ({
    id,
    name,
    length: quantity(tubular.lengthM, 'm'),
    outerDiameter: quantity(tubular.odM, 'm'),
    innerDiameter: quantity(tubular.idM, 'm'),
    materialDensity: quantity(MOCK_CASE.material.steelDensityKgM3, 'kg/m3'),
    youngModulus: quantity(MOCK_CASE.material.youngModulusPa, 'Pa'),
  }));
}

function buildApi7gInputs() {
  return MOCK_CASE.api7g.sections.map((section) => {
    const tubular = MOCK_CASE.tubular[section.tubularKey];
    return {
      id: section.id,
      tubularId: TUBULAR_IDS[section.tubularKey],
      name: section.name,
      length: quantity(tubular.lengthM, 'm'),
      outerDiameter: quantity(tubular.odM, 'm'),
      innerDiameter: quantity(tubular.idM, 'm'),
      fluidDensity: quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'),
      axialLoad: quantity(section.axialLoadN, 'N'),
      tensionLimit: quantity(section.tensionLimitN, 'N'),
      operatingTorque: quantity(section.operatingTorqueNm, 'N*m'),
    };
  });
}

function directionalInputValues() {
  const inputs = Object.fromEntries(directionalReferenceData.inputs.map(({ key, value }) => [key, value]));
  return {
    planLengthUnit: inputs.planLengthUnit,
    planAngleUnit: inputs.planAngleUnit,
    surveyLengthUnit: inputs.surveyLengthUnit,
    surveyAngleUnit: inputs.surveyAngleUnit,
    motorYield: quantity(inputs.fieldVerifiedMotorYieldDegPer100Ft, 'deg/100ft'),
    rotaryBuildTendency: quantity(inputs.rotaryBuildTendencyDegPer100Ft, 'deg/100ft'),
    slideCalibrationWindow: quantity(inputs.slideCalibrationWindowStands, '1'),
  };
}

export function buildMockExchangePayload() {
  const metadata = referenceMetadata();
  const payload = {
    schemaVersion: SCHEMA_VERSION,
    caseId: 'wellforge-mock-case',
    createdAt: '2026-08-27T00:00:00.000Z',
    producer: { name: 'WellForge', version: '1.0.0' },
    metadata: {
      well: metadata.wellName,
      field: metadata.fieldPad,
      pad: metadata.fieldPad,
      rig: metadata.rig,
      datum: metadata.datum,
      northReference: metadata.northReference,
      surfaceNorth: quantity(metadata.surfaceNorthFt, 'ft'),
      surfaceEast: quantity(metadata.surfaceEastFt, 'ft'),
      groundElevation: quantity(metadata.groundElevationFt, 'ft'),
      verticalSectionAzimuth: quantity(metadata.verticalSectionAzimuthDeg, 'deg'),
    },
    unitPreferences: {
      length: 'ft', diameter: 'in', density: 'ppg', force: 'klbf', pressure: 'psi',
      torque: 'ft-lbf', angle: 'deg', flowRate: 'gpm', viscosity: 'cP',
    },
    trajectory: buildTrajectory(),
    holeSections: MOCK_CASE.holeSections.map((section) => ({
      id: section.id,
      name: section.name,
      topMd: quantity(section.topMdM, 'm'),
      bottomMd: quantity(section.bottomMdM, 'm'),
      holeDiameter: quantity(section.holeIdM, 'm'),
    })),
    tubulars: buildTubulars(),
    bhaComponents: MOCK_CASE.bha.map((component) => ({
      id: component.id,
      name: component.name,
      length: quantity(component.lengthM, 'm'),
      outerDiameter: quantity(component.odM, 'm'),
      innerDiameter: quantity(component.idM, 'm'),
      supportFactor: quantity(component.supportFactor, '1'),
    })),
    fluids: [{
      id: 'fluid-01',
      name: 'WellForge water-based mud',
      density: quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'),
      apparentViscosity: quantity(MOCK_CASE.fluid.apparentViscosityPaS, 'Pa*s'),
    }],
    operatingPoint: {
      wob: quantity(MOCK_CASE.operation.wobN, 'N'),
      surfaceTorque: quantity(MOCK_CASE.operation.surfaceTorqueNm, 'N*m'),
      flowRate: quantity(MOCK_CASE.hydraulics.flowRateM3S, 'm3/s'),
      frictionFactor: quantity(MOCK_CASE.operation.frictionFactor, '1'),
      rotarySpeed: quantity(MOCK_CASE.operation.rotarySpeedRpm, 'rpm'),
    },
    rigLimits: {
      surfacePressure: quantity(MOCK_CASE.hydraulics.surfacePressureLimitPa, 'Pa'),
      ecdScreenDensity: quantity(MOCK_CASE.rig.ecdScreenDensityKgM3, 'kg/m3'),
      hookload: quantity(MOCK_CASE.rig.hookloadLimitN, 'N'),
    },
    pumpNozzle: {
      pumps: [{
        id: 'pump-01',
        name: 'Land rig pump set',
        efficiency: quantity(MOCK_CASE.rig.pumpEfficiency, '1'),
        flowRate: quantity(MOCK_CASE.hydraulics.flowRateM3S, 'm3/s'),
      }],
      nozzles: MOCK_CASE.pumpNozzle.nozzles.map((nozzle) => ({
        id: nozzle.id,
        diameter: quantity(nozzle.diameterM, 'm'),
        count: quantity(MOCK_CASE.pumpNozzle.nozzleCount, '1'),
        dischargeCoefficient: quantity(MOCK_CASE.pumpNozzle.dischargeCoefficient, '1'),
      })),
    },
    analyses: {
      api7g: {
        ...formulaAnalysis(),
        fluidId: 'fluid-01',
        materialDensity: quantity(MOCK_CASE.material.steelDensityKgM3, 'kg/m3'),
        sections: buildApi7gInputs(),
        controls: {
          surfaceTorque: quantity(MOCK_CASE.operation.surfaceTorqueNm, 'N*m'),
          hookload: quantity(MOCK_CASE.rig.hookloadLimitN, 'N'),
          designUtilisationLimit: quantity(MOCK_CASE.api7g.designUtilisationLimit, '1'),
        },
      },
      bha: {
        ...formulaAnalysis(),
        componentIds: MOCK_CASE.bha.map(({ id }) => id),
        rotarySpeed: quantity(MOCK_CASE.operation.rotarySpeedRpm, 'rpm'),
        flowRate: quantity(MOCK_CASE.hydraulics.flowRateM3S, 'm3/s'),
        youngModulus: quantity(MOCK_CASE.material.youngModulusPa, 'Pa'),
        materialDensity: quantity(MOCK_CASE.material.steelDensityKgM3, 'kg/m3'),
        lowWob: quantity(MOCK_CASE.operation.lowWobN, 'N'),
        operatingCases: [
          { id: 'bha-low-wob', wob: quantity(MOCK_CASE.operation.lowWobN, 'N') },
          { id: 'bha-primary-wob', wob: quantity(MOCK_CASE.operation.wobN, 'N') },
        ],
      },
      directional: {
        ...formulaAnalysis(),
        inputs: directionalInputValues(),
        trajectoryInputIds: {
          plan: payloadTrajectoryIds('plan'),
          survey: payloadTrajectoryIds('survey'),
          targets: directionalReferenceData.targets.map(({ id }) => id),
          slideIntervals: directionalReferenceData.slideIntervals.map(({ stand }) => `slide-${String(stand).padStart(2, '0')}`),
          formationTops: directionalReferenceData.formationTops.map(({ id }) => id),
        },
      },
      hydraulics: {
        ...formulaAnalysis(),
        fluidId: 'fluid-01',
        flowRate: quantity(MOCK_CASE.hydraulics.flowRateM3S, 'm3/s'),
        surfacePressureLimit: quantity(MOCK_CASE.hydraulics.surfacePressureLimitPa, 'Pa'),
        pumpId: 'pump-01',
        nozzleIds: MOCK_CASE.pumpNozzle.nozzles.map(({ id }) => id),
        pumpEfficiency: quantity(MOCK_CASE.rig.pumpEfficiency, '1'),
        fluidDensity: quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'),
        apparentViscosity: quantity(MOCK_CASE.fluid.apparentViscosityPaS, 'Pa*s'),
        nozzleCount: quantity(MOCK_CASE.pumpNozzle.nozzleCount, '1'),
        baseNozzleDiameter: quantity(MOCK_CASE.pumpNozzle.nozzles.find(({ id }) => id === MOCK_CASE.pumpNozzle.baseNozzleId).diameterM, 'm'),
        nozzleDischargeCoefficient: quantity(MOCK_CASE.pumpNozzle.dischargeCoefficient, '1'),
        flowPath: MOCK_CASE.hydraulics.flowPath.map((section) => ({
          id: section.id,
          name: section.name,
          length: quantity(section.lengthM, 'm'),
          flowId: quantity(section.flowIdM, 'm'),
          hydraulicDiameter: quantity(section.hydraulicDiameterM, 'm'),
          flowType: section.flowType,
        })),
      },
      torqueDrag: {
        ...formulaAnalysis(),
        inputs: {
          fluidId: 'fluid-01',
          surveyIds: payloadTrajectoryIds('survey'),
          holeSectionIds: MOCK_CASE.holeSections.map(({ id }) => id),
          tubularIds: Object.values(TUBULAR_IDS),
          materialDensity: quantity(MOCK_CASE.material.steelDensityKgM3, 'kg/m3'),
          youngModulus: quantity(MOCK_CASE.material.youngModulusPa, 'Pa'),
          wob: quantity(MOCK_CASE.operation.wobN, 'N'),
          surfaceTorque: quantity(MOCK_CASE.operation.surfaceTorqueNm, 'N*m'),
          frictionFactor: quantity(MOCK_CASE.operation.frictionFactor, '1'),
          fluidDensity: quantity(MOCK_CASE.fluid.densityKgM3, 'kg/m3'),
          outerDiameter: quantity(MOCK_CASE.tubular.drillPipe.odM, 'm'),
          innerDiameter: quantity(MOCK_CASE.tubular.drillPipe.idM, 'm'),
        },
      },
    },
    provenance: {
      notes: [
        'Deterministic mock exchange payload derived from the WellForge shared mock case.',
        'Analysis results are intentionally absent until workbook formulas are evaluated.',
      ],
      sources: ['src/shared_mock_case.mjs', 'src/directional_reference_data.mjs'],
    },
    warnings: ['Formula-derived analysis results are not calculated in this fixture.'],
  };
  const validation = validateExchangePayload(payload);
  if (!validation.valid) throw new Error(`Mock exchange payload is invalid: ${validation.errors.join('; ')}`);
  return payload;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const payload = buildMockExchangePayload();
  await fs.writeFile('data/wellforge-mock-case.json', `${JSON.stringify(payload, null, 2)}\n`);
}

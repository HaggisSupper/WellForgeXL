const TWO_PI = 2 * Math.PI;

function clamp(value, lower, upper) {
  return Math.max(lower, Math.min(upper, value));
}

function wrapPositive(angle) {
  return ((angle % TWO_PI) + TWO_PI) % TWO_PI;
}

function wrapSigned(angle) {
  const wrapped = wrapPositive(angle);
  return wrapped > Math.PI ? wrapped - TWO_PI : wrapped;
}

function directionVector({ inc, azi }) {
  return {
    north: Math.sin(inc) * Math.cos(azi),
    east: Math.sin(inc) * Math.sin(azi),
    vertical: Math.cos(inc),
  };
}

function doglegBetween(first, second) {
  const firstDirection = directionVector(first);
  const secondDirection = directionVector(second);
  const dot = firstDirection.north * secondDirection.north
    + firstDirection.east * secondDirection.east
    + firstDirection.vertical * secondDirection.vertical;
  return Math.acos(clamp(dot, -1, 1));
}

function ratioFactor(dogleg) {
  return Math.abs(dogleg) < 1e-9
    ? 1 + dogleg ** 2 / 12 + dogleg ** 4 / 120
    : 2 * Math.tan(dogleg / 2) / dogleg;
}

function displacement(first, second, length) {
  const dogleg = doglegBetween(first, second);
  const ratio = ratioFactor(dogleg);
  return {
    dogleg,
    ratio,
    north: length / 2 * (Math.sin(first.inc) * Math.cos(first.azi) + Math.sin(second.inc) * Math.cos(second.azi)) * ratio,
    east: length / 2 * (Math.sin(first.inc) * Math.sin(first.azi) + Math.sin(second.inc) * Math.sin(second.azi)) * ratio,
    tvd: length / 2 * (Math.cos(first.inc) + Math.cos(second.inc)) * ratio,
  };
}

function normalizedLinearDirection(first, second, fraction) {
  const north = first.north + fraction * (second.north - first.north);
  const east = first.east + fraction * (second.east - first.east);
  const vertical = first.vertical + fraction * (second.vertical - first.vertical);
  const magnitude = Math.hypot(north, east, vertical);
  return { north: north / magnitude, east: east / magnitude, vertical: vertical / magnitude };
}

function slerpDirection(firstStation, secondStation, fraction) {
  const first = directionVector(firstStation);
  const second = directionVector(secondStation);
  const totalDogleg = Math.acos(clamp(first.north * second.north + first.east * second.east + first.vertical * second.vertical, -1, 1));
  if (totalDogleg < 1e-9) return normalizedLinearDirection(first, second, fraction);
  const sine = Math.sin(totalDogleg);
  const firstWeight = Math.sin((1 - fraction) * totalDogleg) / sine;
  const secondWeight = Math.sin(fraction * totalDogleg) / sine;
  return {
    north: firstWeight * first.north + secondWeight * second.north,
    east: firstWeight * first.east + secondWeight * second.east,
    vertical: firstWeight * first.vertical + secondWeight * second.vertical,
  };
}

function stationFromDirection(md, direction) {
  return {
    md,
    inc: Math.acos(clamp(direction.vertical, -1, 1)),
    azi: wrapPositive(Math.atan2(direction.east, direction.north)),
  };
}

export function minimumCurvature(stations) {
  if (!Array.isArray(stations) || stations.length === 0) return [];
  let north = 0;
  let east = 0;
  let tvd = 0;
  return stations.map((station, index) => {
    if (index === 0) return { ...station, north, east, tvd, dmd: 0, dogleg: 0, ratioFactor: 1, dls: 0 };
    const previous = stations[index - 1];
    const dmd = station.md - previous.md;
    if (!(dmd > 0)) throw new RangeError('Minimum-curvature stations must have strictly increasing MD.');
    const interval = displacement(previous, station, dmd);
    north += interval.north;
    east += interval.east;
    tvd += interval.tvd;
    return { ...station, north, east, tvd, dmd, dogleg: interval.dogleg, ratioFactor: interval.ratio, dls: interval.dogleg / dmd };
  });
}

export function interpolateMinimumCurvature(stations, md) {
  if (!Array.isArray(stations) || stations.length === 0) return { status: 'NO_STATIONS', md };
  if (md < stations[0].md) return { status: 'BEFORE START', md };
  if (md > stations.at(-1).md) return { status: 'BEYOND TD', md };
  const exact = stations.find((station) => station.md === md);
  if (exact) return { ...exact, status: 'OK' };
  let lowerIndex = 0;
  while (lowerIndex + 1 < stations.length && stations[lowerIndex + 1].md < md) lowerIndex += 1;
  const lower = stations[lowerIndex];
  const upper = stations[lowerIndex + 1];
  const fraction = (md - lower.md) / (upper.md - lower.md);
  const direction = slerpDirection(lower, upper, fraction);
  const partial = stationFromDirection(md, direction);
  const interval = displacement(lower, partial, md - lower.md);
  return {
    ...partial,
    status: 'OK',
    north: lower.north + interval.north,
    east: lower.east + interval.east,
    tvd: lower.tvd + interval.tvd,
    dmd: md - lower.md,
    dogleg: interval.dogleg,
    ratioFactor: interval.ratio,
    dls: interval.dogleg / (md - lower.md),
  };
}

export function positionError(actual, planned, vsAzimuth) {
  const north = actual.north - planned.north;
  const east = actual.east - planned.east;
  const tvd = actual.tvd - planned.tvd;
  const alongTrack = north * Math.cos(vsAzimuth) + east * Math.sin(vsAzimuth);
  const crossline = -north * Math.sin(vsAzimuth) + east * Math.cos(vsAzimuth);
  const horizontal = Math.hypot(north, east);
  return { north, east, tvd, verticalSection: alongTrack, alongTrack, crossline, horizontal, error3d: Math.hypot(horizontal, tvd) };
}

export function slideVector(interval) {
  const courseLength = interval.mdOut - interval.mdIn;
  const slideLength = interval.slideLength;
  if (!(courseLength > 0) || !(slideLength > 0)) return { status: 'INVALID_SLIDE_LENGTH' };
  const averageInc = (interval.startInc + interval.endInc) / 2;
  if (averageInc < (interval.lowInclinationThreshold ?? 0)) return { status: 'LOW_INCLINATION' };
  const build = (interval.endInc - interval.startInc) / courseLength;
  const effectiveTurn = wrapSigned(interval.endAzi - interval.startAzi) * Math.sin(averageInc) / courseLength;
  const residualBuild = (build - (interval.rotaryBuild ?? 0)) * courseLength / slideLength;
  const residualTurn = (effectiveTurn - (interval.rotaryEffectiveTurn ?? 0)) * courseLength / slideLength;
  const yieldRate = Math.hypot(residualBuild, residualTurn);
  const responseToolface = wrapPositive(Math.atan2(residualTurn, residualBuild));
  return {
    status: 'OK',
    build,
    effectiveTurn,
    residualBuild,
    residualTurn,
    yield: yieldRate,
    responseToolface,
    toolfaceError: wrapSigned(responseToolface - (interval.commandedToolface ?? 0)),
  };
}

export function targetEnvelopeStatus(target, position) {
  const type = target.type;
  const major = target.major;
  const minor = target.minor ?? major;
  const verticalTolerance = target.verticalTolerance;
  if (!['Point', 'Circle', 'Ellipse', 'Box'].includes(type) || !(major > 0) || !(minor > 0) || !(verticalTolerance >= 0)) return { status: 'INVALID_GEOMETRY' };
  const deltaNorth = position.north - target.north;
  const deltaEast = position.east - target.east;
  const rotation = target.rotation ?? 0;
  const localMajor = deltaNorth * Math.cos(rotation) + deltaEast * Math.sin(rotation);
  const localMinor = -deltaNorth * Math.sin(rotation) + deltaEast * Math.cos(rotation);
  let horizontalUtilization;
  if (type === 'Point' || type === 'Circle') horizontalUtilization = Math.hypot(deltaNorth, deltaEast) / major;
  if (type === 'Ellipse') horizontalUtilization = Math.hypot(localMajor / major, localMinor / minor);
  if (type === 'Box') horizontalUtilization = Math.max(Math.abs(localMajor) / major, Math.abs(localMinor) / minor);
  const verticalDifference = position.tvd - target.tvd;
  const verticalUtilization = verticalTolerance === 0 ? (verticalDifference === 0 ? 0 : Infinity) : Math.abs(verticalDifference) / verticalTolerance;
  return {
    status: horizontalUtilization <= 1 && verticalUtilization <= 1 ? 'HIT' : 'MISS',
    horizontalUtilization,
    verticalUtilization,
    localMajor,
    localMinor,
    verticalDifference,
  };
}

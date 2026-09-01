const DATA_URL = "../data/wellforge-mock-case.json";
const FALLBACK = {
  schemaVersion: "1.0.0",
  caseId: "local-fixture-unavailable",
  metadata: { wellName: "WellForge sample well", fieldName: "Local fixture" },
  trajectory: { plan: [], survey: [], targets: [], slideIntervals: [], formationTops: [] },
  tubulars: [], bhaComponents: [], fluids: [], operatingPoint: {}, rigLimits: {}, pumpNozzle: { pumps: [], nozzles: [] },
  analyses: { directional: {}, bha: {}, hydraulics: {}, torqueDrag: {}, api7g: {} }, warnings: ["Fixture could not be loaded."]
};

let caseData = FALLBACK;
const grids = new Map();

const $ = (id) => document.getElementById(id);
const val = (obj, key, fallback = 0) => Number(obj?.[key]?.value ?? fallback);
const fmt = (number, digits = 0) => Number.isFinite(Number(number)) ? Number(number).toLocaleString(undefined, { maximumFractionDigits: digits }) : "—";
const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({"&":"&amp;","<":"&lt;",">":"&gt;","\"":"&quot;","'":"&#039;"}[char]));

function mountGrid(id, data, columns, options = {}) {
  const element = $(id);
  if (!element) return;
  const previous = grids.get(id);
  previous?.destroy?.();
  element.replaceChildren();
  if (typeof window.Tabulator === "function") {
    grids.set(id, new window.Tabulator(element, {
      data,
      layout: "fitColumns",
      height: options.height || "270px",
      movableColumns: true,
      resizableRows: true,
      pagination: true,
      paginationSize: options.paginationSize || 8,
      paginationSizeSelector: [8, 16, 32],
      placeholder: "No rows in fixture",
      columns
    }));
    return;
  }
  // The CDN is optional for file previews; keep a readable fallback while preserving the same data contract.
  const headers = columns.map((column) => `<th>${esc(column.title)}</th>`).join("");
  const rows = data.map((row) => `<tr>${columns.map((column) => `<td>${esc(row[column.field])}</td>`).join("")}</tr>`).join("");
  element.innerHTML = `<table class="data-table"><thead><tr>${headers}</tr></thead><tbody>${rows}</tbody></table>`;
}

function setView(view) {
  document.querySelectorAll(".tab").forEach((tab) => {
    const active = tab.dataset.view === view;
    tab.classList.toggle("is-active", active);
    tab.setAttribute("aria-selected", String(active));
  });
  document.querySelectorAll(".view").forEach((panel) => {
    const active = panel.dataset.panel === view;
    panel.classList.toggle("is-visible", active);
    panel.hidden = !active;
  });
}

function scale(values, start, end) {
  const min = Math.min(...values), max = Math.max(...values);
  return (value) => min === max ? (start + end) / 2 : start + ((value - min) / (max - min)) * (end - start);
}

function linePath(points, x, y) { return points.map((point, index) => `${index ? "L" : "M"}${x(point.x).toFixed(1)},${y(point.y).toFixed(1)}`).join(" "); }

function drawScatter(svg, series, options = {}) {
  const width = 720, height = 360, pad = { left: 52, right: 18, top: 16, bottom: 34 };
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  const allX = series.flatMap((s) => s.points.map((p) => p.x));
  const allY = series.flatMap((s) => s.points.map((p) => p.y));
  if (!allX.length || !allY.length) { svg.innerHTML = `<text class="axis-label" x="${width / 2}" y="${height / 2}" text-anchor="middle">No points in fixture</text>`; return; }
  const x = scale(allX, pad.left, width - pad.right);
  const yNormal = scale(allY, pad.top, height - pad.bottom);
  const y = (value) => options.reverseY ? height - pad.bottom - (yNormal(value) - pad.top) : yNormal(value);
  const xTicks = 4, yTicks = 4;
  let markup = "";
  for (let i = 0; i <= xTicks; i++) { const value = Math.min(...allX) + (Math.max(...allX) - Math.min(...allX)) * i / xTicks; const px = x(value); markup += `<line class="grid-line" x1="${px}" y1="${pad.top}" x2="${px}" y2="${height-pad.bottom}"/><text class="axis-label" x="${px}" y="${height-11}" text-anchor="middle">${fmt(value, options.xDigits ?? 0)}</text>`; }
  for (let i = 0; i <= yTicks; i++) { const value = Math.min(...allY) + (Math.max(...allY) - Math.min(...allY)) * i / yTicks; const py = y(value); markup += `<line class="grid-line" x1="${pad.left}" y1="${py}" x2="${width-pad.right}" y2="${py}"/><text class="axis-label" x="${pad.left-9}" y="${py+3}" text-anchor="end">${fmt(value, options.yDigits ?? 0)}</text>`; }
  markup += `<line class="axis" x1="${pad.left}" y1="${height-pad.bottom}" x2="${width-pad.right}" y2="${height-pad.bottom}"/><line class="axis" x1="${pad.left}" y1="${pad.top}" x2="${pad.left}" y2="${height-pad.bottom}"/>`;
  markup += `<text class="axis-label" x="${width/2}" y="${height-1}" text-anchor="middle">${esc(options.xLabel || "Metric")}</text><text class="axis-label" transform="translate(11 ${height/2}) rotate(-90)" text-anchor="middle">${esc(options.yLabel || "Measured depth")}</text>`;
  series.forEach((item) => { const points = item.points.filter((p) => Number.isFinite(p.x) && Number.isFinite(p.y)); const path = linePath(points, (v) => x(v), (v) => y(v)); markup += `<path d="${path}" fill="none" stroke="${item.color}" stroke-width="${item.width || 2}" ${item.dash ? `stroke-dasharray="${item.dash}"` : ""} opacity=".95"/>`; if (item.markers) points.forEach((point) => { markup += `<circle cx="${x(point.x)}" cy="${y(point.y)}" r="3.2" fill="${item.color}"/>`; }); });
  svg.innerHTML = markup;
}

function drawTrajectory(svg, compact = false) {
  const plan = (caseData.trajectory?.plan || []).map((row) => ({ x: val(row, "inclination") * Math.sin(val(row, "azimuth") * Math.PI / 180), y: val(row, "md") }));
  const survey = (caseData.trajectory?.survey || []).map((row) => ({ x: val(row, "inclination") * Math.sin(val(row, "azimuth") * Math.PI / 180), y: val(row, "md") }));
  drawScatter(svg, [{ name: "Plan", points: plan, color: "#c3f36b", markers: !compact }, { name: "Survey", points: survey, color: "#6ed6d0", markers: !compact }], { reverseY: true, xLabel: "Horizontal departure (relative)", yLabel: "MD (ft)", xDigits: 2, yDigits: 0 });
}

function bulletMarkup(label, actual, target, limit, unit = "") {
  const max = Math.max(limit, actual, target, 1), actualPct = Math.min(100, Math.max(0, actual / max * 100)), targetPct = Math.min(100, Math.max(0, target / max * 100)), limitPct = Math.min(100, Math.max(0, limit / max * 100));
  return `<div class="bullet"><div class="bullet-top"><span>${esc(label)}</span><strong>${fmt(actual, 2)} ${esc(unit)}</strong></div><div class="bullet-track"><span class="bullet-band good" style="width:${targetPct}%"></span><span class="bullet-value" style="width:${actualPct}%"></span><i class="bullet-marker" style="left:${targetPct}%"></i><i class="bullet-marker limit" style="left:${limitPct}%"></i></div><div class="bullet-key"><span>target ${fmt(target, 2)}</span><span>limit ${fmt(limit, 2)} ${esc(unit)}</span></div></div>`;
}

function renderOverview() {
  const plan = caseData.trajectory?.plan || [], survey = caseData.trajectory?.survey || [], components = caseData.bhaComponents || [], flow = caseData.analyses?.hydraulics?.flowPath || [];
  $("case-name").textContent = caseData.metadata?.wellName || caseData.caseId || "Local fixture";
  $("schema-version").textContent = caseData.schemaVersion || "—";
  $("overview-kpis").innerHTML = [
    ["Plan stations", fmt(plan.length), "stations", "trajectory input"],
    ["Survey stations", fmt(survey.length), "stations", "measured input"],
    ["BHA components", fmt(components.length), "items", "geometry loaded"],
    ["Flow path", fmt(flow.length), "segments", "hydraulics input"]
  ].map(([label, value, unit, foot]) => `<article class="panel kpi"><div class="kpi-label">${label}</div><div class="kpi-value">${value}<span class="kpi-unit">${unit}</span></div><div class="kpi-foot">${foot}</div></article>`).join("");
  drawTrajectory($("overview-trajectory"), true);
  const op = caseData.operatingPoint || {}, limits = caseData.rigLimits || {};
  $("overview-bullets").innerHTML = [bulletMarkup("Surface pressure", val(limits, "surfacePressure") / 1e6, 28, val(limits, "surfacePressure") / 1e6, "MPa"), bulletMarkup("Hookload", val(op, "wob") / 1000, val(limits, "hookload") / 1000, val(limits, "hookload") / 1000, "kN"), bulletMarkup("Flow rate", val(op, "flowRate") * 1000, 40, 50, "L/s")].join("");
  const engines = [["Directional", caseData.analyses?.directional], ["BHA", caseData.analyses?.bha], ["Hydraulics", caseData.analyses?.hydraulics], ["Torque & drag", caseData.analyses?.torqueDrag], ["API 7G", caseData.analyses?.api7g]];
  mountGrid("engine-table", engines.map(([name, engine]) => ({ engine: name, state: engine?.calculationState || "not calculated", inputs: `${Object.keys(engine || {}).length} fields` })), [{ title: "Engine", field: "engine", widthGrow: 1.4 }, { title: "State", field: "state", cssClass: "ready" }, { title: "Inputs", field: "inputs", cssClass: "muted" }], { height: "248px", paginationSize: 6 });
  $("rail-well").textContent = caseData.metadata?.wellName || caseData.caseId || "local fixture";
  $("rail-contract").textContent = caseData.schemaVersion || "—";
}

function renderTrajectory() {
  drawTrajectory($("trajectory-chart"));
  const targets = caseData.trajectory?.targets || [], slides = caseData.trajectory?.slideIntervals || [], formations = caseData.trajectory?.formationTops || [];
  $("trajectory-status").textContent = `${targets.length} targets · ${slides.length} slides`;
  $("trajectory-insights").innerHTML = [["Targets", `${targets.length} target windows loaded`, "ready"], ["Slide intervals", `${slides.length} steering intervals loaded`, "ready"], ["Formation tops", `${formations.length} formation markers loaded`, "ready"], ["Plan / survey", `${caseData.analyses?.directional?.calculationState || "not calculated"}`, "fixture"]].map(([title, copy, state]) => `<div class="insight"><div class="insight-copy"><strong>${title}</strong><span>${copy}</span></div><span class="insight-state">${state}</span></div>`).join("");
  const survey = caseData.trajectory?.survey || [];
  $("survey-count").textContent = `${survey.length} rows`;
  mountGrid("survey-table", survey.map((row) => ({ station: row.id, md: `${fmt(val(row, "md"), 1)} ${row.md?.unit || ""}`, inclination: `${fmt(val(row, "inclination"), 2)}°`, azimuth: `${fmt(val(row, "azimuth"), 2)}°` })), [{ title: "Station", field: "station" }, { title: "MD", field: "md", hozAlign: "right" }, { title: "Inclination", field: "inclination", hozAlign: "right" }, { title: "Azimuth", field: "azimuth", hozAlign: "right" }], { height: "330px", paginationSize: 10 });
}

function renderBha() {
  const components = caseData.bhaComponents || [], svg = $("bha-schematic"), width = 450, height = 460, total = components.reduce((sum, item) => sum + val(item, "length"), 0) || 1;
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  let y = 24, markup = `<line class="axis" x1="225" y1="16" x2="225" y2="${height-16}"/>`;
  [...components].reverse().forEach((item, index) => { const h = Math.max(24, val(item, "length") / total * 390); const w = 78 + Math.min(72, val(item, "outerDiameter") * 280); const x = 225 - w / 2; const color = index === components.length - 1 ? "#c3f36b" : (index % 2 ? "#6b9aaa" : "#83b7c5"); markup += `<rect x="${x}" y="${y}" width="${w}" height="${h-4}" rx="7" fill="${color}" opacity=".9"/><text class="point-label" x="${225 + w/2 + 13}" y="${y + Math.min(h/2, 16)}">${esc(item.name)}</text>`; y += h; });
  markup += `<text class="axis-label" x="225" y="${height-2}" text-anchor="middle">bit depth ↑</text>`; svg.innerHTML = markup;
  $("bha-list").innerHTML = components.map((item, index) => `<div class="component"><span class="component-index">${String(index + 1).padStart(2, "0")}</span><div><div class="component-name">${esc(item.name)}</div><div class="component-detail">OD ${fmt(val(item, "outerDiameter") * 1000, 0)} mm · ID ${fmt(val(item, "innerDiameter") * 1000, 0)} mm</div></div><span class="component-length">${fmt(val(item, "length"), 2)} m</span></div>`).join("");
}

function renderHydraulics() {
  const h = caseData.analyses?.hydraulics || {}, flow = h.flowPath || [], nozzles = caseData.pumpNozzle?.nozzles || [];
  $("hydraulics-kpis").innerHTML = [["Flow rate", `${fmt(val(h, "flowRate") * 1000, 1)}`, "L/s", "shared operating point"], ["Fluid density", `${fmt(val(h, "fluidDensity"), 0)}`, "kg/m³", "input"], ["Pump efficiency", `${fmt(val(h, "pumpEfficiency") * 100, 1)}`, "%", "input"], ["Pressure limit", `${fmt(val(h, "surfacePressureLimit") / 1e6, 1)}`, "MPa", "rig limit"]].map(([label, value, unit, foot]) => `<article class="panel kpi"><div class="kpi-label">${label}</div><div class="kpi-value">${value}<span class="kpi-unit">${unit}</span></div><div class="kpi-foot">${foot}</div></article>`).join("");
  const svg = $("flow-chart"), width = 720, height = 360, pad = 52, maxLen = Math.max(...flow.map((item) => val(item, "length")), 1); svg.setAttribute("viewBox", `0 0 ${width} ${height}`); let y = 20, markup = ""; flow.forEach((item, index) => { const hPx = Math.max(18, val(item, "length") / maxLen * 270); const w = 200 + index * 52; markup += `<rect x="${pad}" y="${y}" width="${w}" height="${hPx-3}" rx="6" fill="${index % 2 ? "#6ed6d0" : "#c3f36b"}" opacity=".8"/><text class="point-label" x="${pad + w + 10}" y="${y + Math.min(16, hPx/2)}">${esc(item.name)}</text>`; y += hPx; }); markup += `<text class="axis-label" x="${pad}" y="${height-8}">relative segment length</text>`; svg.innerHTML = markup;
  mountGrid("nozzle-table", nozzles.map((item) => ({ diameter: `${fmt(val(item, "diameter") * 1000, 1)} mm`, count: fmt(val(item, "count"), 0), cd: fmt(val(item, "dischargeCoefficient"), 2), state: val(item, "diameter") === val(h, "baseNozzleDiameter") ? "base" : "candidate" })), [{ title: "Diameter", field: "diameter", hozAlign: "right" }, { title: "Count", field: "count", hozAlign: "right" }, { title: "Cd", field: "cd", hozAlign: "right" }, { title: "State", field: "state" }], { height: "305px", paginationSize: 8 });
}

function renderTorque() {
  const depth = Array.from({ length: 14 }, (_, i) => i * 200), op = caseData.operatingPoint || {}, baseForce = val(op, "wob") / 1000;
  const axial = depth.map((y, i) => ({ x: baseForce * (1 - i * .018), y })), drag = depth.map((y, i) => ({ x: baseForce * .19 + i * 1.1, y })), limit = depth.map((y) => ({ x: baseForce * .9, y }));
  drawScatter($("torque-chart"), [{ points: axial, color: "#c3f36b", markers: true }, { points: drag, color: "#6ed6d0", markers: true }, { points: limit, color: "#ff8e83", dash: "6 5" }], { reverseY: true, xLabel: "Force (kN)", yLabel: "MD (m)", xDigits: 0, yDigits: 0 });
  const inputs = caseData.analyses?.torqueDrag?.inputs || {}; $("torque-inputs").innerHTML = [["WOB", val(inputs, "wob") / 1000, "kN"], ["Surface torque", val(inputs, "surfaceTorque") / 1000, "kN·m"], ["Friction factor", val(inputs, "frictionFactor"), ""], ["Young modulus", val(inputs, "youngModulus") / 1e9, "GPa"]].map(([name, value, unit]) => `<div class="input-item"><span>${name}</span><strong>${fmt(value, 2)} ${unit}</strong></div>`).join("");
  $("torque-bullets").innerHTML = [bulletMarkup("Axial load", baseForce, baseForce * .75, baseForce * .9, "kN"), bulletMarkup("Surface torque", val(op, "surfaceTorque") / 1000, 24, 32, "kN·m"), bulletMarkup("Friction factor", val(inputs, "frictionFactor"), .2, .3, "")].join("");
}

function renderJson() { const json = JSON.stringify(caseData, null, 2); $("json-preview").textContent = json; $("json-summary").textContent = `${json.length.toLocaleString()} characters · ${Object.keys(caseData).length} top-level fields`; $("download-json").onclick = () => { const blob = new Blob([json], { type: "application/json" }); const link = document.createElement("a"); link.href = URL.createObjectURL(blob); link.download = `${caseData.caseId || "wellforge-case"}.json`; link.click(); URL.revokeObjectURL(link.href); }; $("copy-json").onclick = async () => { await navigator.clipboard?.writeText(json); $("copy-json").textContent = "Copied"; setTimeout(() => { $("copy-json").textContent = "Copy visible JSON"; }, 1200); }; }

function renderAll() { renderOverview(); renderTrajectory(); renderBha(); renderHydraulics(); renderTorque(); renderJson(); $("last-refresh").textContent = `Refreshed ${new Date().toLocaleTimeString()}`; }

async function loadData() { try { const response = await fetch(DATA_URL, { cache: "no-store" }); if (!response.ok) throw new Error(`HTTP ${response.status}`); caseData = await response.json(); } catch { caseData = FALLBACK; } renderAll(); }

document.querySelectorAll(".tab").forEach((tab) => tab.addEventListener("click", () => setView(tab.dataset.view)));
document.querySelectorAll("[data-rail-view]").forEach((link) => link.addEventListener("click", () => { setView(link.dataset.railView); document.querySelectorAll("[data-rail-view]").forEach((item) => item.classList.toggle("is-active", item === link)); }));
$("view-filter").addEventListener("input", (event) => { const query = event.target.value.trim().toLowerCase(); document.querySelectorAll(".tab").forEach((tab) => { tab.hidden = Boolean(query) && !tab.textContent.toLowerCase().includes(query); }); document.querySelectorAll("[data-rail-view]").forEach((link) => { link.hidden = Boolean(query) && !link.textContent.toLowerCase().includes(query); }); });
$("reload-data").addEventListener("click", loadData);
loadData();

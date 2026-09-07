import { assembleRigState, type RigStateAssemblyInput, type Severity } from "../contracts/rig-state.ts";

export interface RenderGlyphOptions {
  instanceId?: string;
  compact?: boolean;
}

const severityColor: Record<Severity, string> = {
  normal: "#4e6470",
  advisory: "#93713c",
  warning: "#bf7a20",
  critical: "#d0473d",
};

const escapeXml = (value: string): string => value
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll('"', "&quot;")
  .replaceAll("'", "&#39;");

const marker = (id: string, color: string): string => `
  <marker id="${id}" viewBox="0 0 10 10" refX="7.5" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
    <path d="M 0 0 L 10 5 L 0 10 z" fill="${color}" />
  </marker>`;

const layerSvg = (layerId: string, ids: Record<string, string>): string => {
  const wrapped = (content: string, animClass: string = ""): string =>
    `<g data-layer="${layerId}" class="glyph-layer-group ${animClass}">${content}</g>`;
  switch (layerId) {
    case "PIPE":
      return wrapped(`
        <path d="M120 42V278" fill="none" stroke="#d8e0e4" stroke-width="12" stroke-linecap="round" />
        <path d="M120 42V278" fill="none" stroke="#6f818a" stroke-width="2" stroke-linecap="round" />
        <path d="M93 58H147" stroke="#8b9da5" stroke-width="2" />
        <path d="M93 78H147" stroke="#8b9da5" stroke-width="2" />`);
    case "BIT":
      return wrapped(`<path d="M97 278H143L151 297L134 313H106L89 297Z" fill="#cbd5d9" stroke="#6f818a" stroke-width="2" />
        <path d="M101 294H139M108 304H132" stroke="#6f818a" stroke-width="2" stroke-linecap="round" />`);
    case "CORE":
      return wrapped(`<path d="M96 278V312H61L78 295L61 278Z" fill="#d8e0e4" stroke="#6f818a" stroke-width="2" />
        <path d="M77 295H48" stroke="#d8e0e4" stroke-width="3" marker-end="url(#${ids.arrowLight})" />`);
    case "ROTATION":
      return wrapped(`
        <path d="M161 111A55 55 0 1 1 79 91" fill="none" stroke="#93b8a8" stroke-width="4" stroke-linecap="round" marker-end="url(#${ids.arrowGreen})" />
        <path d="M79 137A55 55 0 0 1 161 157" fill="none" stroke="#93b8a8" stroke-width="4" stroke-linecap="round" marker-end="url(#${ids.arrowGreen})" />`,
        "anim-rotate"
      );
    case "SLIDE_KINK":
      return wrapped(`<path d="M120 212L120 266L145 286" fill="none" stroke="#c98c4a" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M146 286L151 297L134 313" fill="none" stroke="#c98c4a" stroke-width="3" stroke-linecap="round" />`);
    case "FLOW_LOOP":
      return wrapped(`
        <path d="M120 28V70" fill="none" stroke="#5fa8a5" stroke-width="4" stroke-linecap="round" marker-end="url(#${ids.arrowTeal})" />
        <path d="M76 283V79C76 64 86 53 99 53" fill="none" stroke="#5fa8a5" stroke-width="3" stroke-linecap="round" marker-end="url(#${ids.arrowTeal})" className="flow-path" />
        <path d="M164 283V79C164 64 154 53 141 53" fill="none" stroke="#5fa8a5" stroke-width="3" stroke-linecap="round" marker-end="url(#${ids.arrowTeal})" className="flow-path" />`,
        "anim-flow"
      );
    case "RETURNS_OBSERVE":
      return wrapped(`
        <path d="M64 78C82 58 100 58 120 78C100 98 82 98 64 78Z" fill="none" stroke="#b9c8ce" stroke-width="3" />
        <circle cx="92" cy="78" r="7" fill="#b9c8ce" className="observe-eye" />
        <path d="M70 124H98M70 136H98M70 148H98" stroke="#b9c8ce" stroke-width="2" stroke-linecap="round" stroke-dasharray="3 5" />`,
        "anim-observe"
      );
    case "MOVE_DOWN":
      return wrapped(`<path d="M47 132V222" fill="none" stroke="#8ea5ad" stroke-width="4" stroke-linecap="round" marker-end="url(#${ids.arrowSlate})" />`, "anim-move-down");
    case "MOVE_UP":
      return wrapped(`<path d="M47 222V132" fill="none" stroke="#8ea5ad" stroke-width="4" stroke-linecap="round" marker-end="url(#${ids.arrowSlate})" />`, "anim-move-up");
    case "REAM":
      return wrapped(`<path d="M149 270L172 285L149 300" fill="#d8a554" stroke="#9c6f30" stroke-width="2" stroke-linejoin="round" />
        <path d="M149 270L158 285L149 300" fill="none" stroke="#f1d5a2" stroke-width="2" />`);
    case "BACKREAM":
      return wrapped(`<path d="M91 270L68 285L91 300" fill="#d8a554" stroke="#9c6f30" stroke-width="2" stroke-linejoin="round" />
        <path d="M91 270L82 285L91 300" fill="none" stroke="#f1d5a2" stroke-width="2" />`);
    case "CEMENT":
      return wrapped(`<path d="M75 64V265C75 284 88 300 103 304" fill="none" stroke="#b7a27a" stroke-width="6" stroke-linecap="round" stroke-dasharray="4 6" />
        <path d="M165 64V265C165 284 152 300 137 304" fill="none" stroke="#b7a27a" stroke-width="6" stroke-linecap="round" stroke-dasharray="4 6" />`, "anim-flow");
    case "PACK_OFF":
      return wrapped(`<path d="M75 240H102M138 240H165" stroke="#bf7a20" stroke-width="9" stroke-linecap="round" />
        <path d="M74 228L102 252M138 228L166 252" stroke="#f0c77d" stroke-width="2" />`, "anim-pulse");
    case "WASHOUT":
      return wrapped(`<path d="M126 172C154 174 163 180 175 196" fill="none" stroke="#bf7a20" stroke-width="4" stroke-linecap="round" stroke-dasharray="2 4" />
        <circle cx="127" cy="172" r="5" fill="#bf7a20" />`, "anim-pulse");
    case "LOSS":
      return wrapped(`<path d="M172 217C194 233 196 256 180 280C164 256 166 233 172 217Z" fill="url(#${ids.lossHatch})" stroke="#d0473d" stroke-width="3" />
        <path d="M180 286V319" fill="none" stroke="#d0473d" stroke-width="4" marker-end="url(#${ids.arrowRed})" />`, "anim-hazard");
    case "INFLUX":
      return wrapped(`<path d="M67 281C46 257 48 234 65 218C81 238 83 258 67 281Z" fill="#d0473d" opacity="0.84" />
        <path d="M61 265V205" fill="none" stroke="#ffd0ca" stroke-width="3" marker-end="url(#${ids.arrowPink})" />`, "anim-hazard");
    case "WELL_CONTROL":
      return wrapped(`<path d="M72 100H168V127H72Z" fill="#d0473d" stroke="#ffc2bb" stroke-width="2" />
        <path d="M103 100V127M137 100V127" stroke="#6f2520" stroke-width="3" />
        <circle cx="120" cy="113.5" r="7" fill="#fff1ef" />`, "anim-hazard");
    case "WHIRL":
      return wrapped(`<path d="M165 196C153 174 92 174 76 196C92 218 153 218 165 196Z" fill="none" stroke="#bf7a20" stroke-width="3" stroke-dasharray="4 4" />`, "anim-pulse");
    default:
      return "";
  }
};

export function renderRigStateSvg(input: RigStateAssemblyInput, options: RenderGlyphOptions = {}): string {
  const assembly = assembleRigState(input);
  const severity: Severity = assembly.severity;
  const rawId = options.instanceId && options.instanceId.trim().length > 0 ? options.instanceId : `${assembly.state.id.toLowerCase()}-${assembly.status.kind}`;
  const suffix = rawId.replace(/[^a-zA-Z0-9_-]/g, "-");

  const ids = {
    suffix,
    arrowGreen: `arrow-green-${suffix}`,
    arrowSlate: `arrow-slate-${suffix}`,
    arrowTeal: `arrow-teal-${suffix}`,
    arrowRed: `arrow-red-${suffix}`,
    arrowPink: `arrow-pink-${suffix}`,
    arrowLight: `arrow-light-${suffix}`,
    lossHatch: `loss-hatch-${suffix}`,
  };

  const width = options.compact ? 200 : 240;
  const height = options.compact ? 300 : 360;
  const titleId = `glyph-title-${suffix}`;

  const maskPipeId = `mask-pipe-${suffix}`;
  const maskAnnulusId = `mask-annulus-${suffix}`;

  return `<svg class="rig-state-glyph" viewBox="0 0 240 360" width="${width}" height="${height}" role="img" aria-labelledby="${titleId}" data-severity="${severity}" xmlns="http://www.w3.org/2000/svg">
    <title id="${titleId}">${escapeXml(assembly.accessibleLabel)}</title>
    <defs>
      ${marker(ids.arrowGreen, "#93b8a8")}
      ${marker(ids.arrowSlate, "#8ea5ad")}
      ${marker(ids.arrowTeal, "#5fa8a5")}
      ${marker(ids.arrowRed, "#d0473d")}
      ${marker(ids.arrowPink, "#ffd0ca")}
      ${marker(ids.arrowLight, "#d8e0e4")}
      <pattern id="${ids.lossHatch}" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
        <rect width="7" height="7" fill="#472a2b" />
        <path d="M0 2H7" stroke="#f2a49a" stroke-width="2" />
      </pattern>
      <mask id="${maskPipeId}">
        <rect width="240" height="360" fill="white" />
        <rect x="112" y="40" width="16" height="240" fill="black" />
      </mask>
      <mask id="${maskAnnulusId}">
        <rect width="240" height="360" fill="white" />
        <rect x="70" y="50" width="100" height="240" fill="black" />
      </mask>
    </defs>
    <rect x="219" y="25" width="6" height="300" rx="3" fill="${severityColor[severity]}" data-severity-rail="${severity}" />
    <path d="M36 32H204" stroke="#50616a" stroke-width="1" opacity="0.55" />
    ${assembly.layers.map((layer) => layerSvg(layer.id, ids)).join("\n")}
  </svg>`;
}

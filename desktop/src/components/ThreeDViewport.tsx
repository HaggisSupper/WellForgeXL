import { useEffect, useRef, useState } from "react";
import type { SceneDocumentV1, ScenePoint } from "../lib/scene";

interface ThreeDViewportProps {
  scene: SceneDocumentV1 | null;
}

interface Rotation {
  pitch: number;
  yaw: number;
}

export function ThreeDViewport({ scene }: ThreeDViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [visibleLayers, setVisibleLayers] = useState<Record<string, boolean>>({});
  const [rotation, setRotation] = useState<Rotation>({ pitch: -0.45, yaw: 0.72 });
  const [rendererState, setRendererState] = useState("WebGL2 initializing");

  useEffect(() => {
    setVisibleLayers(Object.fromEntries((scene?.layers ?? []).map((layer) => [layer.id, layer.visibleByDefault])));
  }, [scene]);

  useEffect(() => {
    if (!scene || !canvasRef.current) return;
    const canvas = canvasRef.current;
    if (typeof WebGL2RenderingContext === "undefined") {
      setRendererState("WebGL2 unavailable");
      return;
    }
    const context = canvas.getContext("webgl2");
    if (!context) {
      setRendererState("WebGL2 unavailable");
      return;
    }

    const width = Math.max(canvas.clientWidth, 640);
    const height = Math.max(canvas.clientHeight, 360);
    canvas.width = width;
    canvas.height = height;
    context.viewport(0, 0, width, height);
    context.clearColor(0.055, 0.067, 0.082, 1);
    context.clear(context.COLOR_BUFFER_BIT | context.DEPTH_BUFFER_BIT);

    const program = makeProgram(context);
    if (!program) {
      setRendererState("WebGL2 shader initialization failed");
      return;
    }
    context.useProgram(program);
    const positionLocation = context.getAttribLocation(program, "aPosition");
    const colorLocation = context.getUniformLocation(program, "uColor");
    const buffer = context.createBuffer();
    if (!buffer || positionLocation < 0 || !colorLocation) {
      setRendererState("WebGL2 buffer initialization failed");
      context.deleteProgram(program);
      return;
    }

    context.bindBuffer(context.ARRAY_BUFFER, buffer);
    context.enableVertexAttribArray(positionLocation);
    context.vertexAttribPointer(positionLocation, 2, context.FLOAT, false, 0, 0);
    for (const layer of scene.layers) {
      if (!visibleLayers[layer.id]) continue;
      const color = hexColor(layer.color);
      context.uniform3f(colorLocation, color[0], color[1], color[2]);
      for (const primitive of layer.primitives) {
        const points = primitive.kind === "polyline" ? primitive.points : [primitive.point];
        const vertices = new Float32Array(points.flatMap((point) => project(point, scene, rotation)));
        context.bufferData(context.ARRAY_BUFFER, vertices, context.STATIC_DRAW);
        context.drawArrays(primitive.kind === "polyline" ? context.LINE_STRIP : context.POINTS, 0, points.length);
      }
    }
    context.deleteBuffer(buffer);
    context.deleteProgram(program);
    setRendererState("WebGL2 active");
  }, [scene, visibleLayers, rotation]);

  if (!scene) {
    return <section className="rounded border border-slate-700 bg-slate-950/50 p-5 text-sm text-slate-400">3Dmk is awaiting a validated scene from a Rust engineering capability.</section>;
  }

  return (
    <section className="rounded border border-slate-700 bg-slate-950/50" aria-label="3Dmk viewport">
      <header className="flex items-start justify-between border-b border-slate-800 px-4 py-3">
        <div><p className="text-sm font-semibold text-slate-100">3Dmk · {scene.title}</p><p className="mt-1 text-xs text-slate-500">{scene.coordinateFrame} · {rendererState}</p></div>
        <p className="text-right text-xs text-slate-400"><span className="block">{scene.provenance.algorithm}</span><span>{scene.provenance.profileVersion} · {scene.provenance.backend}</span></p>
      </header>
      <canvas
        className="block h-80 w-full cursor-grab touch-none bg-slate-950 active:cursor-grabbing"
        onPointerDown={(event) => {
          const start = { x: event.clientX, y: event.clientY, ...rotation };
          event.currentTarget.setPointerCapture(event.pointerId);
          const move = (moveEvent: PointerEvent) => setRotation({ yaw: start.yaw + (moveEvent.clientX - start.x) * 0.01, pitch: clamp(start.pitch + (moveEvent.clientY - start.y) * 0.01, -1.3, 0.25) });
          const end = () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", end); };
          window.addEventListener("pointermove", move);
          window.addEventListener("pointerup", end);
        }}
        ref={canvasRef}
      />
      <div className="flex flex-wrap gap-x-4 gap-y-2 border-t border-slate-800 px-4 py-3">
        {scene.layers.map((layer) => <label className="flex items-center gap-2 text-xs text-slate-300" key={layer.id}><input aria-label={layer.name} checked={visibleLayers[layer.id] ?? false} onChange={() => setVisibleLayers((current) => ({ ...current, [layer.id]: !current[layer.id] }))} type="checkbox" />{layer.name}</label>)}
      </div>
    </section>
  );
}

function makeProgram(context: WebGL2RenderingContext): WebGLProgram | null {
  const vertex = compileShader(context, context.VERTEX_SHADER, "#version 300 es\nin vec2 aPosition;\nvoid main() { gl_Position = vec4(aPosition, 0.0, 1.0); gl_PointSize = 7.0; }");
  const fragment = compileShader(context, context.FRAGMENT_SHADER, "#version 300 es\nprecision mediump float;\nuniform vec3 uColor;\nout vec4 outColor;\nvoid main() { outColor = vec4(uColor, 1.0); }");
  if (!vertex || !fragment) return null;
  const program = context.createProgram();
  if (!program) return null;
  context.attachShader(program, vertex);
  context.attachShader(program, fragment);
  context.linkProgram(program);
  context.deleteShader(vertex);
  context.deleteShader(fragment);
  return context.getProgramParameter(program, context.LINK_STATUS) ? program : null;
}

function compileShader(context: WebGL2RenderingContext, type: number, source: string): WebGLShader | null {
  const shader = context.createShader(type);
  if (!shader) return null;
  context.shaderSource(shader, source);
  context.compileShader(shader);
  return context.getShaderParameter(shader, context.COMPILE_STATUS) ? shader : null;
}

function project(point: ScenePoint, scene: SceneDocumentV1, rotation: Rotation): [number, number] {
  const centre = { x: (scene.bounds.minimum.x + scene.bounds.maximum.x) / 2, y: (scene.bounds.minimum.y + scene.bounds.maximum.y) / 2, z: (scene.bounds.minimum.z + scene.bounds.maximum.z) / 2 };
  const span = Math.max(scene.bounds.maximum.x - scene.bounds.minimum.x, scene.bounds.maximum.y - scene.bounds.minimum.y, scene.bounds.maximum.z - scene.bounds.minimum.z, 1);
  const x = (point.x - centre.x) * 1.65 / span;
  const y = (point.y - centre.y) * 1.65 / span;
  const z = (point.z - centre.z) * 1.65 / span;
  const cosYaw = Math.cos(rotation.yaw), sinYaw = Math.sin(rotation.yaw);
  const horizontal = cosYaw * x - sinYaw * y;
  const depth = sinYaw * x + cosYaw * y;
  const cosPitch = Math.cos(rotation.pitch), sinPitch = Math.sin(rotation.pitch);
  const vertical = cosPitch * -z - sinPitch * depth;
  return [horizontal, vertical];
}

function hexColor(value: string): [number, number, number] {
  return [1, 3, 5].map((offset) => Number.parseInt(value.slice(offset, offset + 2), 16) / 255) as [number, number, number];
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

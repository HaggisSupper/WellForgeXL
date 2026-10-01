import type { GlyphStatusKind } from "../contracts/rig-state.ts";

export type StatusTone = "neutral" | "positive" | "caution" | "critical";

export function formatTelemetryAge(seconds: number): string {
  if (isNaN(seconds) || !isFinite(seconds) || seconds < 0) {
    return "0 s ago";
  }
  const wholeSeconds = Math.round(seconds);
  if (wholeSeconds < 60) return `${wholeSeconds} s ago`;
  return `${Math.floor(wholeSeconds / 60)} min ${wholeSeconds % 60} s ago`;
}

export function statusPresentation(status: GlyphStatusKind): { label: string; tone: StatusTone } {
  switch (status) {
    case "confirmed":
      return { label: "Confirmed", tone: "positive" };
    case "pending":
      return { label: "Pending", tone: "caution" };
    case "stale":
      return { label: "Stale", tone: "caution" };
    case "conflict":
      return { label: "Conflict", tone: "critical" };
    case "unclassified":
      return { label: "Unclassified", tone: "neutral" };
  }
}

import { create } from "zustand";

export type PlotExportFormat = "png" | "svg" | "pdf";

export interface PlotPreferences {
  palette: string;
  showAxes: boolean;
  showRiskBands: boolean;
  riskBands: [number, number, number];
  visibleLayers: string[];
  exportFormat: PlotExportFormat;
}

interface PlotPreferencesStore extends PlotPreferences {
  update: (preferences: Partial<PlotPreferences>) => void;
}

export const usePlotPrefs = create<PlotPreferencesStore>((set) => ({
  palette: "wellforge-dark",
  showAxes: true,
  showRiskBands: true,
  riskBands: [1.5, 1.2, 1.0],
  visibleLayers: [],
  exportFormat: "png",
  update: (preferences) => set(preferences),
}));

import { create } from "zustand";

export const wellforgeModules = [
  "Project",
  "Surveys",
  "Plans",
  "AC",
  "BHA",
  "T&D",
  "Hydraulics",
  "Reports",
  "Audit",
] as const;

export type WellForgeModule = (typeof wellforgeModules)[number];

interface UiStore {
  activeModule: WellForgeModule;
  setActiveModule: (module: WellForgeModule) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  activeModule: "Project",
  setActiveModule: (activeModule) => set({ activeModule }),
}));

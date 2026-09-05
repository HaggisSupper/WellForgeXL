import { create } from "zustand";
import type { SceneDocumentV1 } from "../lib/scene";

interface SceneState {
  scene: SceneDocumentV1 | null;
  setScene: (scene: SceneDocumentV1 | null) => void;
}

export const useSceneStore = create<SceneState>((set) => ({
  scene: null,
  setScene: (scene) => set({ scene }),
}));

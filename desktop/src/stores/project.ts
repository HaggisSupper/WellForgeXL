import { create } from "zustand";
import { wellforgeIpc, type ApiError, type DocumentInspection, type ProjectDisplaySummary } from "../lib/ipc";

export type ProjectSelectionState = "idle" | "selecting" | "ready" | "cancelled" | "error";

let latestSelectionOperation = 0;

interface ProjectStore {
  activeProject: ProjectDisplaySummary | null;
  inspection: DocumentInspection | null;
  selectionState: ProjectSelectionState;
  error: ApiError | null;
  setActiveProject: (project: ProjectDisplaySummary | null) => void;
  selectAndInspectProject: () => Promise<void>;
}

export const useProjectStore = create<ProjectStore>((set) => ({
  activeProject: null,
  inspection: null,
  selectionState: "idle",
  error: null,
  setActiveProject: (activeProject) => {
    latestSelectionOperation += 1;
    set({ activeProject });
  },
  selectAndInspectProject: async () => {
    const operation = ++latestSelectionOperation;
    set({ selectionState: "selecting", error: null });
    try {
      const result = await wellforgeIpc.selectProject();
      if (operation !== latestSelectionOperation) return;
      if (!result) {
        set({ selectionState: "cancelled", error: null });
        return;
      }
      set({ activeProject: result.project, inspection: result.inspection, selectionState: "ready", error: null });
    } catch (error) {
      if (operation !== latestSelectionOperation) return;
      set({ selectionState: "error", error: displayError(error) });
    }
  },
}));

function displayError(value: unknown): ApiError {
  if (typeof value === "object" && value !== null) {
    const candidate = value as Record<string, unknown>;
    if (typeof candidate.code === "string" && typeof candidate.message === "string") {
      return { code: candidate.code, message: candidate.message, details: candidate.details };
    }
  }
  if (value instanceof Error) {
    return { code: "PROJECT_EXPLORER_FAILED", message: value.message };
  }
  return { code: "PROJECT_EXPLORER_FAILED", message: "The project request could not be completed." };
}

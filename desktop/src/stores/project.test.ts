import { beforeEach, describe, expect, it, vi } from "vitest";
import { useProjectStore } from "./project";

const { selectProject } = vi.hoisted(() => ({ selectProject: vi.fn() }));

vi.mock("../lib/ipc", () => ({ wellforgeIpc: { selectProject } }));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

const inspection = {
  documentType: "project" as const,
  rootName: "DrillProject",
  caption: null,
  surveyCount: 1,
  componentCount: 0,
  diagnostics: [],
};

describe("project selection store", () => {
  beforeEach(() => {
    selectProject.mockReset();
    useProjectStore.setState({ activeProject: null, inspection: null, selectionState: "idle", error: null });
  });

  it("keeps the newer selection when an older successful request resolves last", async () => {
    const older = deferred<{ project: { name: string }; inspection: typeof inspection }>();
    const newer = deferred<{ project: { name: string }; inspection: typeof inspection }>();
    selectProject.mockImplementationOnce(() => older.promise).mockImplementationOnce(() => newer.promise);

    const olderRequest = useProjectStore.getState().selectAndInspectProject();
    const newerRequest = useProjectStore.getState().selectAndInspectProject();
    newer.resolve({ project: { name: "newer.drillproj" }, inspection });
    await newerRequest;
    older.resolve({ project: { name: "older.drillproj" }, inspection });
    await olderRequest;

    const state = useProjectStore.getState();
    expect(state).toMatchObject({
      activeProject: { name: "newer.drillproj" },
      selectionState: "ready",
      error: null,
    });
    expect(state.activeProject).not.toHaveProperty("path");
  });

  it("keeps the newer selection when an older request fails last", async () => {
    const older = deferred<never>();
    const newer = deferred<{ project: { name: string }; inspection: typeof inspection }>();
    selectProject.mockImplementationOnce(() => older.promise).mockImplementationOnce(() => newer.promise);

    const olderRequest = useProjectStore.getState().selectAndInspectProject();
    const newerRequest = useProjectStore.getState().selectAndInspectProject();
    newer.resolve({ project: { name: "newer.drillproj" }, inspection });
    await newerRequest;
    older.reject({ code: "PROJECT_SELECTION_FAILED", message: "Older request failed" });
    await olderRequest;

    const state = useProjectStore.getState();
    expect(state).toMatchObject({
      activeProject: { name: "newer.drillproj" },
      selectionState: "ready",
      error: null,
    });
    expect(state.activeProject).not.toHaveProperty("path");
  });
});

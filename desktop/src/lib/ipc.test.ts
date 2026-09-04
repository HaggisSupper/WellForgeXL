import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { wellforgeIpc } from "./ipc";

const projectRevisionDigest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const requestDigest = "b".repeat(64);

function backendCalculationResponse() {
  return {
    result: { northM: 1, eastM: 2, tvdM: 29.8, doglegRad: 0.1, doglegSeverityRadPerM: 0.0033 },
    receipt: {
      algorithm: "minimum-curvature",
      algorithmVersion: "2026.1",
      inputRevisions: [
        {
          kind: "project_revision",
          id: "project-8a9f:revision-event:18a5e5f3dcb5f2a-1e4-0",
          contentSha256: projectRevisionDigest,
        },
        { kind: "minimum_curvature_request", id: requestDigest, contentSha256: requestDigest },
      ],
      context: {
        unitSystem: "si",
        crs: "EPSG:4979",
        backend: "cpu",
        actorId: "local-workstation",
        warnings: ["Actor identity is currently local to this workstation."],
      },
      outputSha256: "c".repeat(64),
    },
  };
}

describe("WellForge IPC", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("requests the current plot preferences from the desktop command", async () => {
    invoke.mockResolvedValue({ palette: "wellforge-dark", showRiskBands: true });

    await expect(wellforgeIpc.getPlotPreferences()).resolves.toEqual({
      palette: "wellforge-dark",
      showRiskBands: true,
    });
    expect(invoke).toHaveBeenCalledWith("get_plot_preferences");
  });

  it("accepts only metadata-only project audit facts", async () => {
    invoke.mockResolvedValue({
      revisions: [{
        id: "revision-2",
        parentRevisionId: "revision-1",
        contentSha256: "a".repeat(64),
        createdAt: "2026-08-27T12:00:00Z",
        actorId: "local-workstation",
      }],
      calculationReceipts: [{
        id: "receipt-2",
        projectRevisionId: "revision-2",
        projectRevisionContentSha256: "a".repeat(64),
        contentSha256: "b".repeat(64),
        recordedAt: "2026-08-27T12:01:00Z",
        algorithm: "minimum-curvature",
        algorithmVersion: "2026.1",
        actorId: "local-workstation",
        outputSha256: "c".repeat(64),
        warningCount: 1,
      }],
    });

    await expect(wellforgeIpc.getProjectAudit()).resolves.toMatchObject({
      revisions: [{ id: "revision-2" }],
      calculationReceipts: [{ algorithm: "minimum-curvature" }],
    });
    expect(invoke).toHaveBeenCalledWith("get_project_audit");
  });

  it("rejects an audit response that exposes a non-metadata field", async () => {
    invoke.mockResolvedValue({
      revisions: [],
      calculationReceipts: [],
      databasePath: "C:\\private\\local-authority.sqlite3",
    });

    await expect(wellforgeIpc.getProjectAudit()).rejects.toThrow("Invalid project audit response");
  });

  it("rejects an audit response that contains free-form warning text", async () => {
    invoke.mockResolvedValue({
      revisions: [],
      calculationReceipts: [{
        id: "receipt-2",
        projectRevisionId: "revision-2",
        projectRevisionContentSha256: "a".repeat(64),
        contentSha256: "b".repeat(64),
        recordedAt: "2026-08-27T12:01:00Z",
        algorithm: "minimum-curvature",
        algorithmVersion: "2026.1",
        actorId: "local-workstation",
        outputSha256: "c".repeat(64),
        warnings: ["C:\\private\\local-authority.sqlite3"],
      }],
    });

    await expect(wellforgeIpc.getProjectAudit()).rejects.toThrow("Invalid project audit response");
  });

  it("requests a read-only typed document inspection", async () => {
    invoke.mockResolvedValue({
      documentType: "project",
      rootName: "DrillProject",
      caption: null,
      surveyCount: 1,
      componentCount: 0,
      diagnostics: [],
    });

    await expect(wellforgeIpc.inspectDocument()).resolves.toMatchObject({
      documentType: "project",
      surveyCount: 1,
    });
    expect(invoke).toHaveBeenCalledWith("inspect_document");
  });

  it("rejects an invalid inspection response instead of treating it as typed data", async () => {
    invoke.mockResolvedValue({ documentType: "project" });

    await expect(wellforgeIpc.inspectDocument()).rejects.toThrow("Invalid document inspection response");
  });

  it("can request a backend-owned native project selection without supplying a path", async () => {
    invoke.mockResolvedValue({
      project: { name: "project.drillproj" },
      inspection: {
        documentType: "project",
        rootName: "DrillProject",
        caption: null,
        surveyCount: 1,
        componentCount: 0,
        diagnostics: [],
      },
    });

    const result = await wellforgeIpc.selectProject();
    expect(result).toEqual({
      project: { name: "project.drillproj" },
      inspection: expect.objectContaining({ documentType: "project" }),
    });
    expect(result).not.toHaveProperty("project.path");
    expect(invoke).toHaveBeenCalledWith("select_project");
    expect("openProject" in wellforgeIpc).toBe(false);
  });

  it("accepts the receipt returned by the current backend", async () => {
    invoke.mockResolvedValue(backendCalculationResponse());

    const calculateMinimumCurvature = (wellforgeIpc as unknown as {
      calculateMinimumCurvature: (start: unknown, end: unknown) => Promise<unknown>;
    }).calculateMinimumCurvature;
    const result = await calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    );

    expect(result).toMatchObject({ receipt: { algorithm: "minimum-curvature" } });
    expect(invoke).toHaveBeenCalledWith("calculate_minimum_curvature", {
      request: {
        start: { md_m: 0, inclination_rad: 0, azimuth_true_rad: 0 },
        end: { md_m: 30, inclination_rad: 0.1, azimuth_true_rad: 0.2 },
      },
    });
  });

  it("rejects a shape-valid calculation response with noncanonical receipt context", async () => {
    const response = backendCalculationResponse();
    response.receipt.context.unitSystem = "oilfield";
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a nonfinite result", async () => {
    const response = backendCalculationResponse();
    response.result.northM = Infinity;
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a receipt without a project revision", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions = [response.receipt.inputRevisions[1]];
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision with the wrong kind", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].kind = "portable_project_artifact";
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision with a blank opaque ID", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].id = "  ";
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision with an invalid content digest", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].contentSha256 = "not-a-digest";
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision whose opaque ID is its content digest", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].id = projectRevisionDigest;
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision whose opaque ID is a different digest", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].id = "d".repeat(64);
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a project revision whose opaque ID is a case variation of a digest", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[0].id = projectRevisionDigest.toUpperCase();
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a receipt with multiple project revisions", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions.splice(1, 0, {
      kind: "project_revision",
      id: "project-8a9f:revision-event:18a5e5f3dcb5f2a-1e4-1",
      contentSha256: "d".repeat(64),
    });
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });

  it("rejects a request revision whose identity does not match its content digest", async () => {
    const response = backendCalculationResponse();
    response.receipt.inputRevisions[1].contentSha256 = "d".repeat(64);
    invoke.mockResolvedValue(response);

    await expect(wellforgeIpc.calculateMinimumCurvature(
      { mdM: 0, inclinationRad: 0, azimuthTrueRad: 0 },
      { mdM: 30, inclinationRad: 0.1, azimuthTrueRad: 0.2 },
    )).rejects.toThrow("Invalid minimum-curvature calculation response");
  });
});

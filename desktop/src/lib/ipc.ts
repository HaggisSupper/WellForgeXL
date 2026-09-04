import { invoke } from "@tauri-apps/api/core";
import { parseSceneDocumentV1, type SceneDocumentV1 } from "./scene";

export type UnitSystem = "oilfield" | "si" | "custom";

export interface UnitPreferences {
  system: UnitSystem;
}

export interface PlotPreferences {
  palette: string;
  showRiskBands: boolean;
}

/** SI survey station values accepted by the receipt-bearing minimum-curvature command. */
export interface SurveyStation {
  mdM: number;
  inclinationRad: number;
  azimuthTrueRad: number;
}

export interface MinimumCurvatureResult {
  northM: number;
  eastM: number;
  tvdM: number;
  doglegRad: number;
  doglegSeverityRadPerM: number;
}

export interface CalculationReceipt {
  algorithm: string;
  algorithmVersion: string;
  inputRevisions: Array<{
    kind: string;
    id: string;
    contentSha256: string;
  }>;
  context: {
    unitSystem: string;
    crs: string;
    backend: "cpu" | "cuda" | "vulkan";
    actorId: string;
    warnings: string[];
  };
  outputSha256: string;
}

export interface MinimumCurvatureCalculation {
  result: MinimumCurvatureResult;
  receipt: CalculationReceipt;
}

/** Metadata-only immutable lineage returned by the desktop audit boundary. */
export interface ProjectRevisionAudit {
  id: string;
  parentRevisionId: string | null;
  contentSha256: string;
  createdAt: string;
  actorId: string;
}

/** Metadata-only calculation provenance returned by the desktop audit boundary. */
export interface CalculationReceiptAudit {
  id: string;
  projectRevisionId: string;
  projectRevisionContentSha256: string;
  contentSha256: string;
  recordedAt: string;
  algorithm: string;
  algorithmVersion: string;
  actorId: string;
  outputSha256: string;
  warningCount: number;
}

export interface ProjectAudit {
  revisions: ProjectRevisionAudit[];
  calculationReceipts: CalculationReceiptAudit[];
}

/** Safe display facts for a locally selected project. Local paths stay in the desktop backend. */
export interface ProjectDisplaySummary {
  name: string;
}

/** A selection and the inspection taken from that exact backend-owned selection. */
export interface ProjectSelectionResult {
  project: ProjectDisplaySummary;
  inspection: DocumentInspection;
}

export interface ApiError {
  code: string;
  message: string;
  details?: unknown;
}

export type InspectionSeverity = "error" | "warning";

export interface InspectionDiagnostic {
  severity: InspectionSeverity;
  code: string;
  message: string;
}

/** Read-only typed facts returned for a supported portable document. */
export interface DocumentInspection {
  documentType: "project" | "bha";
  rootName: string;
  caption: string | null;
  surveyCount: number;
  componentCount: number;
  diagnostics: InspectionDiagnostic[];
}

function isDocumentInspection(value: unknown): value is DocumentInspection {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  if (
    (candidate.documentType !== "project" && candidate.documentType !== "bha")
    || typeof candidate.rootName !== "string"
    || (typeof candidate.caption !== "string" && candidate.caption !== null)
    || typeof candidate.surveyCount !== "number"
    || typeof candidate.componentCount !== "number"
    || !Array.isArray(candidate.diagnostics)
  ) return false;

  return candidate.diagnostics.every((diagnostic) => {
    if (typeof diagnostic !== "object" || diagnostic === null) return false;
    const entry = diagnostic as Record<string, unknown>;
    return (entry.severity === "error" || entry.severity === "warning")
      && typeof entry.code === "string"
      && typeof entry.message === "string";
  });
}

async function inspectActiveDocument(): Promise<DocumentInspection> {
  const response = await invoke<unknown>("inspect_document");
  if (!isDocumentInspection(response)) {
    throw new Error("Invalid document inspection response");
  }
  return response;
}

async function selectProject(): Promise<ProjectSelectionResult | null> {
  const response = await invoke<unknown>("select_project");
  if (response === null) return null;
  if (typeof response !== "object" || response === null) throw new Error("Invalid project selection response");
  const candidate = response as Record<string, unknown>;
  const project = candidate.project;
  if (typeof project !== "object" || project === null || typeof (project as Record<string, unknown>).name !== "string") {
    throw new Error("Invalid project selection response");
  }
  const name = (project as Record<string, unknown>).name;
  if (typeof name !== "string" || name.trim().length === 0 || !isDocumentInspection(candidate.inspection)) {
    throw new Error("Invalid project selection response");
  }
  return { project: { name }, inspection: candidate.inspection };
}

function isSha256(value: unknown): value is string {
  return typeof value === "string" && /^[a-fA-F0-9]{64}$/.test(value);
}

function hasOnlyKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).every((key) => keys.includes(key));
}

function isSafeAuditText(value: unknown): value is string {
  return typeof value === "string"
    && value.trim().length > 0
    && !value.includes("\\")
    && !value.includes("/")
    && !value.includes("\u0000");
}

function isUtcTimestamp(value: unknown): value is string {
  return typeof value === "string"
    && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/.test(value)
    && Number.isFinite(Date.parse(value));
}

function isProjectRevisionAudit(value: unknown): value is ProjectRevisionAudit {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return hasOnlyKeys(candidate, ["id", "parentRevisionId", "contentSha256", "createdAt", "actorId"])
    && isSafeAuditText(candidate.id)
    && (candidate.parentRevisionId === null || isSafeAuditText(candidate.parentRevisionId))
    && isSha256(candidate.contentSha256)
    && isUtcTimestamp(candidate.createdAt)
    && isSafeAuditText(candidate.actorId);
}

function isCalculationReceiptAudit(value: unknown): value is CalculationReceiptAudit {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return hasOnlyKeys(candidate, [
    "id", "projectRevisionId", "projectRevisionContentSha256", "contentSha256", "recordedAt",
    "algorithm", "algorithmVersion", "actorId", "outputSha256", "warningCount",
  ])
    && isSafeAuditText(candidate.id)
    && isSafeAuditText(candidate.projectRevisionId)
    && isSha256(candidate.projectRevisionContentSha256)
    && isSha256(candidate.contentSha256)
    && isUtcTimestamp(candidate.recordedAt)
    && isSafeAuditText(candidate.algorithm)
    && isSafeAuditText(candidate.algorithmVersion)
    && isSafeAuditText(candidate.actorId)
    && isSha256(candidate.outputSha256)
    && typeof candidate.warningCount === "number"
    && Number.isSafeInteger(candidate.warningCount)
    && candidate.warningCount >= 0;
}

function isProjectAudit(value: unknown): value is ProjectAudit {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return hasOnlyKeys(candidate, ["revisions", "calculationReceipts"])
    && Array.isArray(candidate.revisions)
    && candidate.revisions.every(isProjectRevisionAudit)
    && Array.isArray(candidate.calculationReceipts)
    && candidate.calculationReceipts.every(isCalculationReceiptAudit);
}

async function getProjectAudit(): Promise<ProjectAudit> {
  const response = await invoke<unknown>("get_project_audit");
  if (!isProjectAudit(response)) {
    throw new Error("Invalid project audit response");
  }
  return response;
}

function isMinimumCurvatureCalculation(value: unknown): value is MinimumCurvatureCalculation {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  const result = candidate.result;
  const receipt = candidate.receipt;
  if (typeof result !== "object" || result === null || typeof receipt !== "object" || receipt === null) return false;
  const resultValue = result as Record<string, unknown>;
  const receiptValue = receipt as Record<string, unknown>;
  const context = receiptValue.context;
  const inputRevisions = receiptValue.inputRevisions;
  if (!["northM", "eastM", "tvdM", "doglegRad", "doglegSeverityRadPerM"].every((key) => typeof resultValue[key] === "number" && Number.isFinite(resultValue[key]))) return false;
  if (receiptValue.algorithm !== "minimum-curvature" || typeof receiptValue.algorithmVersion !== "string" || receiptValue.algorithmVersion.trim().length === 0 || !isSha256(receiptValue.outputSha256)) return false;
  if (typeof context !== "object" || context === null || !Array.isArray(inputRevisions)) return false;
  const contextValue = context as Record<string, unknown>;
  if (contextValue.unitSystem !== "si" || contextValue.crs !== "EPSG:4979" || contextValue.backend !== "cpu" || contextValue.actorId !== "local-workstation" || !Array.isArray(contextValue.warnings) || !contextValue.warnings.every((warning) => typeof warning === "string" && warning.trim().length > 0)) return false;
  const projectRevisions = inputRevisions.filter((revision) => (
    typeof revision === "object"
    && revision !== null
    && (revision as Record<string, unknown>).kind === "project_revision"
  ));
  const requestRevisions = inputRevisions.filter((revision) => (
    typeof revision === "object"
    && revision !== null
    && (revision as Record<string, unknown>).kind === "minimum_curvature_request"
  ));
  if (projectRevisions.length !== 1 || requestRevisions.length !== 1) return false;

  return inputRevisions.every((revision) => {
    if (typeof revision !== "object" || revision === null) return false;
    const input = revision as Record<string, unknown>;
    if (input.kind === "project_revision") {
      return typeof input.id === "string"
        && input.id.trim().length > 0
        && !isSha256(input.id)
        && isSha256(input.contentSha256);
    }
    return input.kind === "minimum_curvature_request"
      && isSha256(input.id)
      && isSha256(input.contentSha256)
      && input.id === input.contentSha256;
  });
}

async function calculateMinimumCurvature(
  start: SurveyStation,
  end: SurveyStation,
): Promise<MinimumCurvatureCalculation> {
  const response = await invoke<unknown>("calculate_minimum_curvature", {
    request: {
      start: {
        md_m: start.mdM,
        inclination_rad: start.inclinationRad,
        azimuth_true_rad: start.azimuthTrueRad,
      },
      end: {
        md_m: end.mdM,
        inclination_rad: end.inclinationRad,
        azimuth_true_rad: end.azimuthTrueRad,
      },
    },
  });
  if (!isMinimumCurvatureCalculation(response)) {
    throw new Error("Invalid minimum-curvature calculation response");
  }
  return response;
}

export interface SurveyPosition {
  mdM: number;
  northM: number;
  eastM: number;
  tvdM: number;
}

export const wellforgeIpc = {
  ping: () => invoke<{ message: string }>("ping"),
  selectProject,
  saveProject: () => invoke<ProjectDisplaySummary>("save_project"),
  getUnits: () => invoke<UnitPreferences>("get_units"),
  getPlotPreferences: () => invoke<PlotPreferences>("get_plot_preferences"),
  getProjectAudit,
  inspectDocument: inspectActiveDocument,
  calculateMinimumCurvature,
  buildSurveyScene: async (stations: SurveyPosition[]): Promise<SceneDocumentV1> => parseSceneDocumentV1(
    await invoke<unknown>("build_survey_scene", { request: { stations } }),
  ),
};

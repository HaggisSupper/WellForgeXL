import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AuditWorkspace } from "./AuditWorkspace";
import { useProjectStore } from "../stores/project";

const { getProjectAudit } = vi.hoisted(() => ({
  getProjectAudit: vi.fn(),
}));

vi.mock("../lib/ipc", () => ({
  wellforgeIpc: { getProjectAudit },
}));

describe("Audit workspace", () => {
  beforeEach(() => {
    getProjectAudit.mockReset();
    useProjectStore.setState({ activeProject: null });
  });

  it("shows immutable lineage and receipt metadata without exposing private data", async () => {
    useProjectStore.setState({ activeProject: { name: "Aster-01.drillproj" } });
    getProjectAudit.mockResolvedValue({
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
        outputSha256: "c".repeat(64),
        warningCount: 1,
      }],
    });

    render(<AuditWorkspace />);

    await waitFor(() => expect(screen.getByRole("region", { name: "Revision lineage" })).toBeInTheDocument());
    expect(screen.getByText("minimum-curvature")).toBeInTheDocument();
    expect(screen.getByText("2026.1")).toBeInTheDocument();
    expect(screen.getByText("Receipt SHA-256")).toBeInTheDocument();
    expect(screen.getByText("Revision content SHA-256")).toBeInTheDocument();
    expect(screen.getByText("1 warning recorded")).toBeInTheDocument();
    expect(screen.queryByText(/C:\\|sqlite|receipt payload|calculation output/i)).not.toBeInTheDocument();
  });

  it("does not display a raw audit-load error", async () => {
    useProjectStore.setState({ activeProject: { name: "Aster-01.drillproj" } });
    getProjectAudit.mockRejectedValue(new Error("Could not read C:\\private\\local-authority.sqlite3"));

    render(<AuditWorkspace />);

    expect(await screen.findByRole("alert")).toHaveTextContent("AUDIT_LOAD_FAILED");
    expect(screen.getByRole("alert")).toHaveTextContent("Audit metadata could not be loaded.");
    expect(screen.queryByText(/C:\\private|sqlite3/i)).not.toBeInTheDocument();
  });

  it("does not display an untrusted structured audit-load error code", async () => {
    useProjectStore.setState({ activeProject: { name: "Aster-01.drillproj" } });
    getProjectAudit.mockRejectedValue({
      code: "C:\\private\\local-authority.sqlite3",
      message: "Could not load local audit data.",
    });

    render(<AuditWorkspace />);

    expect(await screen.findByRole("alert")).toHaveTextContent("AUDIT_LOAD_FAILED");
    expect(screen.queryByText(/C:\\private|sqlite3/i)).not.toBeInTheDocument();
  });
});

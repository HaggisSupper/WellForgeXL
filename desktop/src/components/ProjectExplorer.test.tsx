import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProjectExplorer } from "./ProjectExplorer";
import { useProjectStore } from "../stores/project";

const { selectProject } = vi.hoisted(() => ({
  selectProject: vi.fn(),
}));

vi.mock("../lib/ipc", () => ({
  wellforgeIpc: { selectProject },
}));

describe("Project explorer", () => {
  beforeEach(() => {
    selectProject.mockReset();
    useProjectStore.setState({
      activeProject: null,
      inspection: null,
      selectionState: "idle",
      error: null,
    });
  });

  it("requests backend-owned selection then displays its read-only inspection", async () => {
    selectProject.mockResolvedValue({
      project: { name: "Aster-01.drillproj" },
      inspection: {
        documentType: "project",
        rootName: "DrillProject",
        caption: "Aster 01",
        surveyCount: 3,
        componentCount: 0,
        diagnostics: [
          { severity: "warning", code: "MISSING_UWI", message: "No UWI was provided." },
          { severity: "error", code: "INVALID_SURVEY", message: "A survey value is invalid." },
        ],
      },
    });

    render(<ProjectExplorer />);
    fireEvent.click(screen.getByRole("button", { name: "Select project" }));

    await waitFor(() => expect(screen.getByText("Aster-01.drillproj")).toBeInTheDocument());
    expect(screen.queryByText("C:/approved/Aster-01.drillproj")).not.toBeInTheDocument();
    expect(screen.getByText("DrillProject")).toBeInTheDocument();
    expect(screen.getByText("Aster 01")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByRole("listitem", { name: "Warning diagnostic: MISSING_UWI. No UWI was provided." })).toBeInTheDocument();
    expect(screen.getByRole("listitem", { name: "Error diagnostic: INVALID_SURVEY. A survey value is invalid." })).toBeInTheDocument();
    expect(screen.getByText("Warning")).toBeInTheDocument();
    expect(screen.getByText("Error")).toBeInTheDocument();
  });

  it("acknowledges a cancelled native picker without showing an error", async () => {
    selectProject.mockResolvedValue(null);

    render(<ProjectExplorer />);
    fireEvent.click(screen.getByRole("button", { name: "Select project" }));

    await waitFor(() => expect(screen.getByText("Project selection was cancelled.")).toBeInTheDocument());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows a structured error when the selected document cannot be inspected", async () => {
    selectProject.mockRejectedValue({ code: "FORMAT_MALFORMED_XML", message: "The selected document is not well-formed XML" });

    render(<ProjectExplorer />);
    fireEvent.click(screen.getByRole("button", { name: "Select project" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("FORMAT_MALFORMED_XML");
    expect(screen.getByRole("alert")).toHaveTextContent("The selected document is not well-formed XML");
  });

  it("shows a structured error when the native selection request fails", async () => {
    selectProject.mockRejectedValue({ code: "PROJECT_SELECTION_FAILED", message: "The native picker did not complete" });

    render(<ProjectExplorer />);
    fireEvent.click(screen.getByRole("button", { name: "Select project" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("PROJECT_SELECTION_FAILED");
    expect(screen.getByRole("alert")).toHaveTextContent("The native picker did not complete");
  });
});

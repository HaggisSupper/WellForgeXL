import { render, screen } from "@testing-library/react";
import { ThreeDViewport } from "./ThreeDViewport";
import type { SceneDocumentV1 } from "../lib/scene";

const fixtureScene: SceneDocumentV1 = {
  schemaVersion: "wellforge.scene/v1",
  sceneId: "survey-scene",
  title: "Survey trajectory",
  coordinateFrame: "north-east-tvd-m",
  bounds: {
    minimum: { x: 0, y: 0, z: 0 },
    maximum: { x: 20, y: 10, z: 100 },
  },
  provenance: {
    algorithm: "survey-position-adapter",
    profileVersion: "v1",
    backend: "cpu",
    inputRevision: null,
    warnings: [],
  },
  layers: [
    {
      id: "survey-path",
      name: "Survey path",
      visibleByDefault: true,
      selectable: false,
      color: "#bfc5ca",
      primitives: [{ kind: "polyline", points: [{ x: 0, y: 0, z: 0 }, { x: 20, y: 10, z: 100 }] }],
    },
  ],
};

describe("3Dmk viewport", () => {
  it("shows scene provenance and exposes an enabled layer toggle", () => {
    render(<ThreeDViewport scene={fixtureScene} />);

    expect(screen.getByText("survey-position-adapter")).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Survey path" })).toBeChecked();
  });
});

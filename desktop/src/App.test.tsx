import { render, screen } from "@testing-library/react";
import { App } from "./App";

describe("WellForge application shell", () => {
  it("renders every requested module in the navigation", () => {
    render(<App />);

    const navigation = screen.getByRole("navigation", { name: "WellForge modules" });
    expect(navigation).toHaveTextContent("Project");
    expect(navigation).toHaveTextContent("Surveys");
    expect(navigation).toHaveTextContent("Plans");
    expect(navigation).toHaveTextContent("AC");
    expect(navigation).toHaveTextContent("BHA");
    expect(navigation).toHaveTextContent("T&D");
    expect(navigation).toHaveTextContent("Hydraulics");
    expect(navigation).toHaveTextContent("Reports");
  });
});

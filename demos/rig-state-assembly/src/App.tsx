import { useCallback, useEffect } from "react";

import { operatorScenarios } from "./preview/operator-scenarios.ts";
import { statusPresentation } from "./ui/operator-format.ts";
import { RigStateProvider, useRigStateStore } from "./store/RigStateContext.tsx";

import { Header } from "./components/Header.tsx";
import { GlyphViewer } from "./components/GlyphViewer.tsx";
import { StateSummary } from "./components/StateSummary.tsx";
import { ScenarioSelector } from "./components/ScenarioSelector.tsx";
import { EvidenceLedger } from "./components/EvidenceLedger.tsx";
import { SemanticNotes } from "./components/SemanticNotes.tsx";
import { CommandPalette } from "./components/CommandPalette.tsx";
import { StateBuilder } from "./components/StateBuilder.tsx";
import { StateComparison } from "./components/StateComparison.tsx";
import { DeterminismGuide } from "./components/DeterminismGuide.tsx";
import { ErrorBoundary } from "./components/ErrorBoundary.tsx";

export function DashboardContent() {
  const store = useRigStateStore();
  const {
    activeKey,
    scenario,
    viewModel,
    glyphSvg,
    hoveredLayerId,
    isPaletteOpen,
    isBuilderOpen,
    isComparisonOpen,
    isGuideOpen,
    selectScenario,
    applyCustomInput,
    setHoveredLayerId,
    setIsPaletteOpen,
    setIsBuilderOpen,
    setIsComparisonOpen,
    setIsGuideOpen,
  } = store;

  const status = statusPresentation(viewModel.status.kind);

  const handleSelectScenario = useCallback(
    (key: string) => {
      selectScenario(key);
    },
    [selectScenario]
  );

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement) return;

      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsPaletteOpen(!isPaletteOpen);
        return;
      }

      const num = parseInt(e.key, 10);
      if (!isNaN(num) && num >= 1 && num <= operatorScenarios.length) {
        handleSelectScenario(operatorScenarios[num - 1].key);
      } else if (e.key === "ArrowDown" || e.key === "ArrowRight") {
        const currentIndex = operatorScenarios.findIndex((s) => s.key === activeKey);
        const nextIndex = (currentIndex + 1) % operatorScenarios.length;
        handleSelectScenario(operatorScenarios[nextIndex].key);
      } else if (e.key === "ArrowUp" || e.key === "ArrowLeft") {
        const currentIndex = operatorScenarios.findIndex((s) => s.key === activeKey);
        const prevIndex = (currentIndex - 1 + operatorScenarios.length) % operatorScenarios.length;
        handleSelectScenario(operatorScenarios[prevIndex].key);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeKey, handleSelectScenario, isPaletteOpen, setIsPaletteOpen]);

  return (
    <div className="app-shell">
      <Header
        onOpenCommandPalette={() => setIsPaletteOpen(true)}
        onOpenStateBuilder={() => setIsBuilderOpen(true)}
        onOpenComparison={() => setIsComparisonOpen(true)}
        onOpenDeterminismGuide={() => setIsGuideOpen(true)}
      />

      <p className="demo-notice">Simulated scenarios only. No live WITSML/ETP, telemetry inference, or alarm authority.</p>
      <main className="workspace" aria-labelledby="workspace-title">
        <section className="operator-stage" aria-labelledby="workspace-title">
          <div className="stage-heading">
            <div>
              <p className="section-label">Current interpretation</p>
              <h1 id="workspace-title">{scenario.label}</h1>
            </div>
            <span className={`status-chip tone-${status.tone}`} aria-live="polite">
              <span className="status-dot" aria-hidden="true" />
              {status.label}
            </span>
          </div>

          <div className="glyph-stage">
            <GlyphViewer
              glyphSvg={glyphSvg}
              severity={viewModel.severity}
              statusKind={viewModel.status.kind}
              hoveredLayerId={hoveredLayerId}
            />

            <StateSummary
              view={viewModel}
              scenario={scenario}
              glyphSvg={glyphSvg}
              hoveredLayerId={hoveredLayerId}
              onHoverLayer={setHoveredLayerId}
            />
          </div>
        </section>

        <aside className="control-panel" aria-label="State preview controls">
          <ScenarioSelector activeKey={activeKey} onSelectScenario={handleSelectScenario} />
          <EvidenceLedger evidence={viewModel.evidence} />
          <SemanticNotes />
        </aside>
      </main>

      <CommandPalette
        isOpen={isPaletteOpen}
        onClose={() => setIsPaletteOpen(false)}
        onSelectScenario={handleSelectScenario}
      />

      {isBuilderOpen && (
        <StateBuilder
          onApplyAssemblyInput={applyCustomInput}
          onClose={() => setIsBuilderOpen(false)}
        />
      )}

      {isComparisonOpen && (
        <StateComparison
          onClose={() => setIsComparisonOpen(false)}
        />
      )}

      {isGuideOpen && (
        <DeterminismGuide
          onClose={() => setIsGuideOpen(false)}
        />
      )}
    </div>
  );
}

export function App() {
  return (
    <ErrorBoundary>
      <RigStateProvider>
        <DashboardContent />
      </RigStateProvider>
    </ErrorBoundary>
  );
}

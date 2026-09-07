import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

import { assembleRigState, type RigStateAssemblyInput } from "../contracts/rig-state.ts";
import { getScenario, type OperatorScenario } from "../preview/operator-scenarios.ts";
import { renderRigStateSvg } from "../glyph/render-svg.ts";
import { createOperatorViewModel, type OperatorViewModel } from "../ui/operator-view-model.ts";
import { MockScenarioTelemetryProvider, type TelemetryConnectionStatus } from "../telemetry/telemetry-provider.ts";

interface RigStateContextType {
  activeKey: string;
  scenario: OperatorScenario;
  viewModel: OperatorViewModel;
  glyphSvg: string;
  hoveredLayerId: string | null;
  connectionStatus: TelemetryConnectionStatus;
  isPaletteOpen: boolean;
  isBuilderOpen: boolean;
  isComparisonOpen: boolean;
  isGuideOpen: boolean;
  isReportOpen: boolean;
  selectScenario: (key: string) => void;
  applyCustomInput: (input: RigStateAssemblyInput) => void;
  setHoveredLayerId: (layerId: string | null) => void;
  setIsPaletteOpen: (open: boolean) => void;
  setIsBuilderOpen: (open: boolean) => void;
  setIsComparisonOpen: (open: boolean) => void;
  setIsGuideOpen: (open: boolean) => void;
  setIsReportOpen: (open: boolean) => void;
}

const RigStateContext = createContext<RigStateContextType | null>(null);

const defaultProvider = new MockScenarioTelemetryProvider();

export function RigStateProvider({ children }: { children: ReactNode }) {
  const [activeKey, setActiveKey] = useState("rotate-on-bottom");
  const [customScenario, setCustomScenario] = useState<OperatorScenario | null>(null);
  const [hoveredLayerId, setHoveredLayerId] = useState<string | null>(null);
  const [connectionStatus] = useState<TelemetryConnectionStatus>(defaultProvider.getStatus());

  const [isPaletteOpen, setIsPaletteOpen] = useState(false);
  const [isBuilderOpen, setIsBuilderOpen] = useState(false);
  const [isComparisonOpen, setIsComparisonOpen] = useState(false);
  const [isGuideOpen, setIsGuideOpen] = useState(false);
  const [isReportOpen, setIsReportOpen] = useState(false);

  const scenario = useMemo(() => {
    if (customScenario && activeKey === "custom-simulation") {
      return customScenario;
    }
    return getScenario(activeKey);
  }, [activeKey, customScenario]);

  const viewModel = useMemo(() => createOperatorViewModel(scenario), [scenario]);

  const glyphSvg = useMemo(
    () => renderRigStateSvg(scenario.input, { instanceId: activeKey }),
    [activeKey, scenario.input]
  );

  const selectScenario = (key: string) => {
    setActiveKey(key);
    setHoveredLayerId(null);
    defaultProvider.selectScenario(key);
  };

  const applyCustomInput = (input: RigStateAssemblyInput) => {
    const customSc: OperatorScenario = {
      key: "custom-simulation",
      label: `Custom Simulation — ${input.stateId}`,
      shortLabel: "Custom",
      expectedStatus: assembleRigState(input).status.kind,
      input,
    };
    setCustomScenario(customSc);
    setActiveKey("custom-simulation");
  };

  const value = {
    activeKey,
    scenario,
    viewModel,
    glyphSvg,
    hoveredLayerId,
    connectionStatus,
    isPaletteOpen,
    isBuilderOpen,
    isComparisonOpen,
    isGuideOpen,
    isReportOpen,
    selectScenario,
    applyCustomInput,
    setHoveredLayerId,
    setIsPaletteOpen,
    setIsBuilderOpen,
    setIsComparisonOpen,
    setIsGuideOpen,
    setIsReportOpen,
  };

  return <RigStateContext.Provider value={value}>{children}</RigStateContext.Provider>;
}

export function useRigStateStore(): RigStateContextType {
  const context = useContext(RigStateContext);
  if (!context) {
    throw new Error("useRigStateStore must be used within a RigStateProvider");
  }
  return context;
}

import { operatorScenarios, type OperatorScenario } from "../preview/operator-scenarios.ts";

interface ScenarioSelectorProps {
  activeKey: string;
  onSelectScenario: (key: string) => void;
}

export function ScenarioSelector({ activeKey, onSelectScenario }: ScenarioSelectorProps) {
  return (
    <section className="scenario-selector" aria-labelledby="scenario-title">
      <div className="panel-heading">
        <div>
          <p className="section-label">Demonstration cases</p>
          <h2 id="scenario-title">Choose a rig condition</h2>
        </div>
        <span className="panel-count">{operatorScenarios.length} cases</span>
      </div>
      <ul className="scenario-list" role="list">
        {operatorScenarios.map((candidate: OperatorScenario, index: number) => (
          <li key={candidate.key} role="listitem">
            <button
              className={candidate.key === activeKey ? "scenario-button is-active" : "scenario-button"}
              type="button"
              aria-pressed={candidate.key === activeKey}
              onClick={() => onSelectScenario(candidate.key)}
              title={`Shortcut: Key ${index + 1}`}
            >
              <span>
                <kbd className="hotkey-badge">{index + 1}</kbd>
                {candidate.shortLabel}
              </span>
              <small>{candidate.expectedStatus}</small>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

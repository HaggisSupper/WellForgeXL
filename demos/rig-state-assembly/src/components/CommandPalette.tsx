import { useEffect, useState } from "react";
import { operatorScenarios } from "../preview/operator-scenarios.ts";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectScenario: (key: string) => void;
}

export function CommandPalette({ isOpen, onClose, onSelectScenario }: CommandPaletteProps) {
  const [query, setQuery] = useState("");

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        if (isOpen) onClose();
        else {
          setQuery("");
          // Open handled by parent or state toggle
        }
      } else if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const filteredScenarios = operatorScenarios.filter(
    (s) =>
      s.label.toLowerCase().includes(query.toLowerCase()) ||
      s.shortLabel.toLowerCase().includes(query.toLowerCase()) ||
      s.key.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div className="palette-backdrop" onClick={onClose}>
      <div className="palette-modal" onClick={(e) => e.stopPropagation()}>
        <div className="palette-input-wrapper">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            className="palette-input"
            placeholder="Search rig state, scenario, or telemetry..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoFocus
          />
          <kbd className="palette-esc-badge">ESC</kbd>
        </div>

        <div className="palette-results">
          {filteredScenarios.length === 0 ? (
            <div className="palette-empty">No rig state conditions found for "{query}"</div>
          ) : (
            filteredScenarios.map((scenario) => (
              <button
                key={scenario.key}
                type="button"
                className="palette-item"
                onClick={() => {
                  onSelectScenario(scenario.key);
                  onClose();
                }}
              >
                <div>
                  <span className="palette-item-title">{scenario.label}</span>
                  <span className="palette-item-key">key: {scenario.key}</span>
                </div>
                <span className="palette-item-status">{scenario.expectedStatus}</span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

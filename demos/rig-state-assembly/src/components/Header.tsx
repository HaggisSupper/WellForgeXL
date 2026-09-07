interface HeaderProps {
  onOpenCommandPalette?: () => void;
  onOpenStateBuilder?: () => void;
  onOpenComparison?: () => void;
  onOpenDeterminismGuide?: () => void;
}

export function Header({
  onOpenCommandPalette,
  onOpenStateBuilder,
  onOpenComparison,
  onOpenDeterminismGuide,
}: HeaderProps) {
  return (
    <header className="topbar">
      <div className="brand-lockup" aria-label="WellForge Rig State Assembly">
        <span className="brand-mark" aria-hidden="true">WF</span>
        <div>
          <p className="product-name">WellForge</p>
          <p className="product-subtitle">Rig state assembly web demo</p>
        </div>
      </div>

      <div className="topbar-context">
        <button
          type="button"
          className="action-button"
          onClick={onOpenCommandPalette}
          title="Open Command Palette (Ctrl+K)"
          style={{ padding: "0.35rem 0.65rem" }}
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <span>Search</span>
          <kbd className="hotkey-badge" style={{ margin: 0, fontSize: "0.62rem" }}>⌘K</kbd>
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onOpenStateBuilder}
          title="Open Custom Telemetry Simulator & State Builder"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
          </svg>
          Simulator
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onOpenComparison}
          title="Compare Rig States Side-by-Side"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="2" y="3" width="8" height="18" rx="2" />
            <rect x="14" y="3" width="8" height="18" rx="2" />
          </svg>
          Compare
        </button>

        <button
          type="button"
          className="action-button"
          onClick={onOpenDeterminismGuide}
          title="Open Sensor Determinism & Boundary Guide"
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
          Guide
        </button>

        <span className="context-label">Contract-first</span>
        <span className="release-chip">v0.1 prototype</span>
      </div>
    </header>
  );
}

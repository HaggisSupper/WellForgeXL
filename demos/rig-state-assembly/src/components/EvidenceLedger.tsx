interface EvidenceLedgerProps {
  evidence: readonly string[];
}

export function EvidenceLedger({ evidence }: EvidenceLedgerProps) {
  return (
    <section className="evidence-panel" aria-labelledby="evidence-title">
      <div className="panel-heading">
        <div>
          <p className="section-label">Why this state</p>
          <h2 id="evidence-title">Evidence ledger</h2>
        </div>
      </div>
      <ul className="evidence-list">
        {evidence.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
      <p className="evidence-note">
        Confidence and liveness remain visible even when the display retains the last confirmed activity.
      </p>
    </section>
  );
}

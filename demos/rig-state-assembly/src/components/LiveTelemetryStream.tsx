import { useEffect, useState } from "react";

interface TelemetryMetric {
  label: string;
  value: string;
  unit: string;
  status: "normal" | "warning" | "critical";
  trend: "up" | "down" | "stable";
}

interface LiveTelemetryStreamProps {
  scenarioKey: string;
}

export function LiveTelemetryStream({ scenarioKey }: LiveTelemetryStreamProps) {
  const [metrics, setMetrics] = useState<TelemetryMetric[]>([]);
  const [pulse, setPulse] = useState(false);

  useEffect(() => {
    // Generate scenario-tailored metrics
    const getMetrics = (): TelemetryMetric[] => {
      switch (scenarioKey) {
        case "rotate-on-bottom":
          return [
            { label: "RPM", value: "120", unit: "rpm", status: "normal", trend: "stable" },
            { label: "WOB", value: "28.4", unit: "klbf", status: "normal", trend: "up" },
            { label: "Torque", value: "14.2", unit: "klbf·ft", status: "normal", trend: "stable" },
            { label: "Flow In", value: "520", unit: "gpm", status: "normal", trend: "stable" },
          ];
        case "flow-check":
          return [
            { label: "Pump Pressure", value: "0", unit: "psi", status: "normal", trend: "down" },
            { label: "Flow Out", value: "1.2", unit: "%", status: "normal", trend: "down" },
            { label: "Pit Delta", value: "+0.1", unit: "bbl", status: "normal", trend: "stable" },
            { label: "Hookload", value: "185", unit: "klbf", status: "normal", trend: "stable" },
          ];
        case "losses":
          return [
            { label: "Pit Delta", value: "-8.4", unit: "bbl", status: "critical", trend: "down" },
            { label: "Flow Out", value: "91.8", unit: "%", status: "warning", trend: "down" },
            { label: "SPP", value: "2,450", unit: "psi", status: "normal", trend: "stable" },
            { label: "Flow In", value: "540", unit: "gpm", status: "normal", trend: "stable" },
          ];
        case "influx":
          return [
            { label: "Flow Out", value: "114.5", unit: "%", status: "critical", trend: "up" },
            { label: "Pit Delta", value: "+6.2", unit: "bbl", status: "critical", trend: "up" },
            { label: "SPP", value: "2,120", unit: "psi", status: "warning", trend: "down" },
            { label: "Active Pit", value: "482", unit: "bbl", status: "critical", trend: "up" },
          ];
        case "well-control":
          return [
            { label: "BOP Status", value: "CLOSED", unit: "annular", status: "critical", trend: "stable" },
            { label: "Choke Pos", value: "23", unit: "%", status: "warning", trend: "stable" },
            { label: "SIDPP", value: "420", unit: "psi", status: "critical", trend: "up" },
            { label: "SICP", value: "680", unit: "psi", status: "critical", trend: "up" },
          ];
        default:
          return [
            { label: "Scenario Source", value: "SIMULATED", unit: "offline", status: "normal", trend: "stable" },
          ];
      }
    };

    setMetrics(getMetrics());
  }, [scenarioKey]);

  useEffect(() => {
    const interval = setInterval(() => {
      setPulse((prev) => !prev);
    }, 1200);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="telemetry-stream-card">
      <div className="stream-header">
        <div className="stream-title-lockup">
          <span className={`heartbeat-dot ${pulse ? "pulse" : ""}`} />
          <span className="stream-label">Simulated Scenario Telemetry</span>
        </div>
        <span className="badge-live">SAMPLE DATA</span>
      </div>

      <div className="metrics-grid">
        {metrics.map((m) => (
          <div key={m.label} className={`metric-cell status-${m.status}`}>
            <span className="metric-label">{m.label}</span>
            <div className="metric-value-row">
              <span className="metric-value tabular-num">{m.value}</span>
              <span className="metric-unit">{m.unit}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

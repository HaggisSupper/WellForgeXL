import type { RigStateAssemblyInput } from "../contracts/rig-state.ts";
import { getScenario } from "../preview/operator-scenarios.ts";

export type TelemetryConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

export interface TelemetryProvider {
  name: string;
  getStatus(): TelemetryConnectionStatus;
  subscribe(onSample: (input: RigStateAssemblyInput) => void): () => void;
  selectScenario?(key: string): RigStateAssemblyInput;
}

export class MockScenarioTelemetryProvider implements TelemetryProvider {
  public name = "Mock Scenario Telemetry";
  private listeners = new Set<(input: RigStateAssemblyInput) => void>();
  private activeKey = "rotate-on-bottom";

  public getStatus(): TelemetryConnectionStatus {
    return "connected";
  }

  public selectScenario(key: string): RigStateAssemblyInput {
    this.activeKey = key;
    const scenario = getScenario(key);
    this.notify(scenario.input);
    return scenario.input;
  }

  public subscribe(onSample: (input: RigStateAssemblyInput) => void): () => void {
    this.listeners.add(onSample);
    const initialScenario = getScenario(this.activeKey);
    onSample(initialScenario.input);

    return () => {
      this.listeners.delete(onSample);
    };
  }

  private notify(input: RigStateAssemblyInput) {
    for (const listener of this.listeners) {
      listener(input);
    }
  }
}

export class WebSocketTelemetryProvider implements TelemetryProvider {
  public name = "WITSML / ETP Stream";
  private status: TelemetryConnectionStatus = "disconnected";
  private listeners = new Set<(input: RigStateAssemblyInput) => void>();
  private ws: WebSocket | null = null;
  private url: string;

  constructor(url: string = "ws://localhost:8080/witsml/stream") {
    this.url = url;
  }

  public getStatus(): TelemetryConnectionStatus {
    return this.status;
  }

  public subscribe(onSample: (input: RigStateAssemblyInput) => void): () => void {
    this.listeners.add(onSample);

    if (this.listeners.size === 1) {
      this.connect();
    }

    return () => {
      this.listeners.delete(onSample);
      if (this.listeners.size === 0 && this.ws) {
        this.ws.close();
        this.ws = null;
        this.status = "disconnected";
      }
    };
  }

  private connect() {
    this.status = "connecting";
    try {
      if (typeof WebSocket === "undefined") {
        this.status = "disconnected";
        return;
      }

      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.status = "connected";
      };

      this.ws.onmessage = (event) => {
        try {
          const sample = JSON.parse(event.data) as RigStateAssemblyInput;
          for (const listener of this.listeners) {
            listener(sample);
          }
        } catch (err) {
          console.error("Failed to parse WITSML telemetry frame:", err);
        }
      };

      this.ws.onerror = () => {
        this.status = "error";
      };

      this.ws.onclose = () => {
        this.status = "disconnected";
      };
    } catch {
      this.status = "error";
    }
  }
}

import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error("Uncaught error in WellForge app shell:", error, errorInfo);
  }

  public render(): ReactNode {
    if (this.state.hasError) {
      return (
        <div className="error-fallback-stage" style={{ padding: "2rem", color: "var(--critical)", background: "var(--canvas-deep)" }}>
          <h2>Application Error</h2>
          <p style={{ color: "var(--muted)", margin: "1rem 0" }}>
            An unexpected error occurred during rig state assembly.
          </p>
          <pre style={{ background: "var(--surface)", padding: "1rem", borderRadius: "8px", overflowX: "auto" }}>
            {this.state.error?.message}
          </pre>
          <button
            type="button"
            className="scenario-button"
            style={{ marginTop: "1rem" }}
            onClick={() => this.setState({ hasError: false, error: null })}
          >
            Reset Application State
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

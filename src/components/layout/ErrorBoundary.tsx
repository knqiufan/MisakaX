import { Component, type ErrorInfo, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[ErrorBoundary] Uncaught error:", error, info.componentStack);
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return <DefaultFallback error={this.state.error} onReset={this.handleReset} />;
    }

    return this.props.children;
  }
}

function DefaultFallback({
  error,
  onReset,
}: {
  error: Error | null;
  onReset: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex h-full w-full items-center justify-center bg-background p-8">
      <div className="max-w-md space-y-4 text-center">
        <div className="text-4xl">⚠️</div>
        <h2 className="text-xl font-semibold text-foreground">
          {t("errorBoundary.title")}
        </h2>
        <p className="text-sm text-muted-foreground">
          {t("errorBoundary.description")}
        </p>
        {error && (
          <details className="rounded-md border bg-muted/50 p-3 text-left text-xs">
            <summary className="cursor-pointer font-medium text-muted-foreground">
              {t("errorBoundary.details")}
            </summary>
            <pre className="mt-2 overflow-auto whitespace-pre-wrap text-destructive">
              {error.message}
            </pre>
          </details>
        )}
        <div className="flex justify-center gap-3">
          <button
            onClick={onReset}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            {t("errorBoundary.retry")}
          </button>
          <button
            onClick={() => window.location.reload()}
            className="rounded-md border px-4 py-2 text-sm font-medium text-foreground hover:bg-muted transition-colors"
          >
            {t("errorBoundary.reload")}
          </button>
        </div>
      </div>
    </div>
  );
}

import { Toaster } from "@/components/ui/sonner";
import { AppShell } from "@/components/layout/AppShell";
import { ErrorBoundary } from "@/components/layout/ErrorBoundary";
import { TrayBridge } from "@/components/layout/TrayBridge";

export function App() {
  return (
    <ErrorBoundary>
      <TrayBridge />
      <AppShell />
      <Toaster position="bottom-right" richColors />
    </ErrorBoundary>
  );
}

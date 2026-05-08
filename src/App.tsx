import { Toaster } from "@/components/ui/sonner";
import { AppShell } from "@/components/layout/AppShell";
import { ErrorBoundary } from "@/components/ErrorBoundary";

export function App() {
  return (
    <ErrorBoundary>
      <AppShell />
      <Toaster position="bottom-right" richColors />
    </ErrorBoundary>
  );
}

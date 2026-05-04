import { Toaster } from "@/components/ui/sonner";
import { AppShell } from "@/components/layout/AppShell";

export function App() {
  return (
    <>
      <AppShell />
      <Toaster position="bottom-right" richColors />
    </>
  );
}

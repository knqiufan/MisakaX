import { useEffect } from "react";
import { useAppStore } from "@/stores/app-store";

/** One-release compatibility route for historic `{ page: "skills" }` links. */
export function SkillsPage() {
  const navigate = useAppStore((state) => state.navigate);

  useEffect(() => {
    navigate({ page: "settings", tab: "skills" });
  }, [navigate]);

  return null;
}

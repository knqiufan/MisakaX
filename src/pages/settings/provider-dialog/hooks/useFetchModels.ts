import { useState } from "react";
import type {
  FetchModelsResult,
  FetchProviderModelsRequest,
} from "@/lib/ipc";
import { useSettingsStore } from "@/stores/settings-store";

export function useFetchModels() {
  const fetchModels = useSettingsStore((state) => state.fetchModels);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<FetchModelsResult | null>(null);

  async function run(request: FetchProviderModelsRequest) {
    setLoading(true);
    try {
      const next = await fetchModels(request);
      setResult(next);
      return next;
    } finally {
      setLoading(false);
    }
  }

  return { fetchModels: run, loading, result };
}

import { useState } from "react";
import type { ModelTestResult, TestModelRequest } from "@/lib/ipc";
import { useSettingsStore } from "@/stores/settings-store";

export function useModelTest() {
  const testModel = useSettingsStore((state) => state.testModel);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ModelTestResult | null>(null);

  async function run(request: TestModelRequest) {
    setLoading(true);
    try {
      const next = await testModel(request);
      setResult(next);
      return next;
    } finally {
      setLoading(false);
    }
  }

  return { testModel: run, loading, result };
}

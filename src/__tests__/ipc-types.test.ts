import { describe, expectTypeOf, it } from "vitest";
import type { FetchModelsResult } from "@/lib/ipc";
import type { ModelInfo } from "@/lib/ipc/models";

describe("IPC types", () => {
  it("models fetched from providers use transient model metadata", () => {
    expectTypeOf<FetchModelsResult["models"][number]>().toEqualTypeOf<ModelInfo>();
  });
});

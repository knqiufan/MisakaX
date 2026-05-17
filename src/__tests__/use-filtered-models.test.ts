import { describe, expect, it } from "vitest";
import {
  filterModels,
  groupModels,
} from "@/components/chat/model-selector/useFilteredModels";
import {
  isSelectedModelValid,
  selectedModelLabel,
} from "@/components/chat/model-selector/model-data";
import type { FlatModel } from "@/components/chat/ModelSelector";

describe("model selector filtering", () => {
  it("filters by model id, label, router name, and vendor label", () => {
    expect(filterModels(models, "glm").map((model) => model.id)).toEqual([
      "router-2:glm-4.5",
    ]);
    expect(filterModels(models, "work").map((model) => model.id)).toEqual([
      "router-1:gpt-4o",
    ]);
    expect(filterModels(models, "kimi").map((model) => model.id)).toEqual([
      "router-3:moonshot-v1-8k",
    ]);
  });

  it("groups models by vendor label and sorts items", () => {
    const groups = groupModels(models);

    expect(groups.map((group) => group.vendorLabel)).toEqual([
      "GLM",
      "Kimi",
      "OpenAI",
    ]);
    expect(groups[2].items.map((model) => model.label)).toEqual(["GPT-4o"]);
  });

  it("strictly falls back when selected model is no longer available", () => {
    expect(selectedModelLabel("router-9:missing-model", models, "Select model")).toBe(
      "Select model",
    );
    expect(isSelectedModelValid("router-9:missing-model", models)).toBe(false);
  });
});

const models: FlatModel[] = [
  {
    id: "router-1:gpt-4o",
    label: "GPT-4o",
    modelId: "gpt-4o",
    provider: "openai",
    routerName: "Work OpenAI",
    vendorId: "openai",
    vendorLabel: "OpenAI",
  },
  {
    id: "router-2:glm-4.5",
    label: "GLM-4.5",
    modelId: "glm-4.5",
    provider: "openai",
    routerName: "China Models",
    vendorId: "zhipu",
    vendorLabel: "GLM",
  },
  {
    id: "router-3:moonshot-v1-8k",
    label: "Moonshot 8K",
    modelId: "moonshot-v1-8k",
    provider: "openai",
    routerName: "Kimi Router",
    vendorId: "moonshot",
    vendorLabel: "Kimi",
  },
];

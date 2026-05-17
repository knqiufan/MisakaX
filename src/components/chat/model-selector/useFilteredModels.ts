import { useMemo } from "react";
import type { FlatModel, ModelGroup } from "./types";

export function useFilteredModels(models: FlatModel[], query: string): ModelGroup[] {
  return useMemo(() => groupModels(filterModels(models, query)), [models, query]);
}

export function filterModels(models: FlatModel[], query: string): FlatModel[] {
  const normalized = query.trim().toLowerCase();
  if (!normalized) return models;

  return models.filter((model) =>
    [model.label, model.modelId, model.routerName, model.vendorLabel]
      .filter(Boolean)
      .some((value) => value.toLowerCase().includes(normalized)),
  );
}

export function groupModels(models: FlatModel[]): ModelGroup[] {
  const groups = new Map<string, ModelGroup>();

  for (const model of models) {
    const key = model.vendorId;
    const group = groups.get(key) ?? {
      vendorId: model.vendorId,
      vendorLabel: model.vendorLabel,
      items: [],
    };
    group.items.push(model);
    groups.set(key, group);
  }

  return Array.from(groups.values())
    .sort((left, right) => left.vendorLabel.localeCompare(right.vendorLabel))
    .map((group) => ({
      ...group,
      items: [...group.items].sort(compareModels),
    }));
}

function compareModels(left: FlatModel, right: FlatModel): number {
  return (
    left.vendorLabel.localeCompare(right.vendorLabel) ||
    left.routerName.localeCompare(right.routerName) ||
    left.label.localeCompare(right.label)
  );
}

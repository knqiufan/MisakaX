import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ModelSelector, type FlatModel } from "@/components/chat/ModelSelector";
import { useAppStore } from "@/stores/app-store";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        modelSearch: "Search models",
        clearModelSearch: "Clear search",
        modelSearchEmpty: "No matching models",
        noModels: "No models available",
        goToSettings: "Go to settings",
      })[key] ?? key,
  }),
}));

describe("ModelSelector", () => {
  it("renders empty state when no models are configured", () => {
    renderSelector({ models: [] });

    expect(screen.getByText("No models available")).toBeTruthy();
    expect(screen.getByText("Go to settings")).toBeTruthy();
  });

  it("navigates to model settings from empty state", () => {
    useAppStore.setState({ route: { page: "chat" } });
    renderSelector({ models: [] });

    fireEvent.click(screen.getByText("Go to settings"));

    expect(useAppStore.getState().route).toEqual({
      page: "settings",
      tab: "models",
    });
  });

  it("filters models by search query and selects a model", () => {
    const onSelect = vi.fn();
    renderSelector({ onSelect });

    fireEvent.change(screen.getByPlaceholderText("Search models"), {
      target: { value: "glm" },
    });

    expect(screen.getByText("GLM-4.5")).toBeTruthy();
    expect(screen.queryByText("GPT-4o")).toBeNull();

    fireEvent.click(screen.getByText("GLM-4.5"));

    expect(onSelect).toHaveBeenCalledWith("router-2:glm-4.5");
  });

  it("clears the search query", () => {
    renderSelector({});

    fireEvent.change(screen.getByPlaceholderText("Search models"), {
      target: { value: "glm" },
    });
    fireEvent.click(screen.getByLabelText("Clear search"));

    expect(screen.getByText("GPT-4o")).toBeTruthy();
    expect(screen.getByText("GLM-4.5")).toBeTruthy();
  });

  it("shows selected state for the current model", () => {
    renderSelector({ selectedModel: "router-1:gpt-4o" });

    expect(screen.getByText("GPT-4o")).toBeTruthy();
  });
});

function renderSelector({
  models = modelFixtures,
  onSelect = vi.fn(),
  selectedModel = null,
}: {
  models?: FlatModel[];
  onSelect?: (modelId: string | null) => void;
  selectedModel?: string | null;
}) {
  return render(
    <ModelSelector
      models={models}
      selectedModel={selectedModel}
      onSelect={onSelect}
      open
      onOpenChange={vi.fn()}
      trigger={<button type="button">Model</button>}
    />,
  );
}

const modelFixtures: FlatModel[] = [
  {
    id: "router-1:gpt-4o",
    label: "GPT-4o",
    modelId: "gpt-4o",
    provider: "openai",
    routerName: "OpenAI Router",
    vendorId: "openai",
    vendorLabel: "OpenAI",
  },
  {
    id: "router-2:glm-4.5",
    label: "GLM-4.5",
    modelId: "glm-4.5",
    provider: "openai",
    routerName: "GLM Router",
    vendorId: "zhipu",
    vendorLabel: "GLM",
  },
];

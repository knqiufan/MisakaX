import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  init: vi.fn(),
  setOption: vi.fn(),
  resize: vi.fn(),
  dispose: vi.fn(),
  resizeCallback: undefined as (() => void) | undefined,
  mutationCallback: undefined as (() => void) | undefined,
  resizeDisconnect: vi.fn(),
  mutationDisconnect: vi.fn(),
}));

vi.mock("echarts", () => ({ init: mocks.init }));

import { EChartCanvas, resolveChartCssVariables } from "@/components/charts/EChartCanvas";

describe("EChartCanvas", () => {
  beforeEach(() => {
    mocks.setOption.mockReset();
    mocks.resize.mockReset();
    mocks.dispose.mockReset();
    mocks.resizeDisconnect.mockReset();
    mocks.mutationDisconnect.mockReset();
    mocks.init.mockReset().mockReturnValue({
      setOption: mocks.setOption,
      resize: mocks.resize,
      dispose: mocks.dispose,
    });
    vi.stubGlobal(
      "ResizeObserver",
      class {
        constructor(callback: () => void) {
          mocks.resizeCallback = callback;
        }
        observe() {}
        disconnect() {
          mocks.resizeDisconnect();
        }
      }
    );
    vi.stubGlobal(
      "MutationObserver",
      class {
        constructor(callback: () => void) {
          mocks.mutationCallback = callback;
        }
        observe() {}
        disconnect() {
          mocks.mutationDisconnect();
        }
      }
    );
    document.documentElement.style.setProperty("--chart-1", "rgb(1, 2, 3)");
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    document.documentElement.removeAttribute("style");
  });

  it("lazy-initializes, resolves theme variables, resizes, reapplies theme, and disposes", async () => {
    const { container, unmount } = render(
      <EChartCanvas
        option={{ color: ["var(--chart-1)"] }}
        ariaLabel="Usage chart"
        className="h-64"
        fallback={<div>fallback</div>}
      />
    );

    await waitFor(() => expect(mocks.init).toHaveBeenCalledOnce());
    expect(mocks.setOption.mock.calls[0][0]).toEqual({ color: ["rgb(1, 2, 3)"] });
    expect(container.querySelector('[data-chart-state="ready"]')).toBeTruthy();
    fireEvent(window, new Event("resize"));
    mocks.resizeCallback?.();
    expect(mocks.resize).toHaveBeenCalledOnce();

    document.documentElement.style.setProperty("--chart-1", "rgb(4, 5, 6)");
    mocks.mutationCallback?.();
    expect(
      mocks.setOption.mock.calls[mocks.setOption.mock.calls.length - 1]?.[0]
    ).toEqual({ color: ["rgb(4, 5, 6)"] });

    const mutationDisconnectCount = mocks.mutationDisconnect.mock.calls.length;
    unmount();
    expect(mocks.dispose).toHaveBeenCalledOnce();
    expect(mocks.resizeDisconnect).toHaveBeenCalledOnce();
    expect(mocks.mutationDisconnect).toHaveBeenCalledTimes(
      mutationDisconnectCount + 1
    );
  });

  it("renders caller-owned fallback when ECharts initialization fails", async () => {
    mocks.init.mockImplementationOnce(() => {
      throw new Error("canvas unavailable");
    });
    render(
      <EChartCanvas
        option={{}}
        ariaLabel="Usage chart"
        fallback={<div role="alert">Readable data fallback</div>}
      />
    );
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Readable data fallback"
    );
  });

  it("preserves formatter functions while resolving nested CSS variables", () => {
    const formatter = () => "value";
    const styles = {
      getPropertyValue: (name: string) => (name === "--chart-1" ? "#123456" : ""),
    } as CSSStyleDeclaration;
    expect(
      resolveChartCssVariables(
        { color: "var(--chart-1)", nested: [{ formatter }] },
        styles
      )
    ).toEqual({ color: "#123456", nested: [{ formatter }] });
  });
});

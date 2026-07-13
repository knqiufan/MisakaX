import { createElement } from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import sourcePng from "@/assets/brand/misakax-logo.png?inline";
import sourceSvg from "@/assets/brand/misakax-logo.svg?raw";
import { MisakaLogo } from "@/components/brand/MisakaLogo";
import buildScript from "../../src-tauri/build.rs?raw";

function readPngColorType(dataUrl: string) {
  const encodedPng = dataUrl.split(",", 2)[1];
  expect(encodedPng).toBeDefined();

  return atob(encodedPng ?? "").charCodeAt(25);
}

describe("brand asset pipeline", () => {
  it("keeps the canonical PNG transparent outside the mark", () => {
    expect(readPngColorType(sourcePng)).toBe(6);
  });

  it("uses the PNG canvas geometry in its SVG counterpart", () => {
    expect(sourceSvg).toContain('viewBox="0 0 128 128"');
    expect(sourceSvg).toContain('<circle cx="64" cy="64" r="55.00"');
    expect(sourceSvg.match(/<path\b/g)).toHaveLength(2);
  });

  it("renders the shared SVG asset instead of duplicating its geometry", () => {
    render(createElement(MisakaLogo, { title: "MisakaX" }));

    expect(screen.getByRole("img", { name: "MisakaX" }).tagName).toBe("IMG");
  });

  it("rebuilds the Windows resource after the ICO changes", () => {
    expect(buildScript).toContain('cargo:rerun-if-changed=icons/icon.ico');
  });
});

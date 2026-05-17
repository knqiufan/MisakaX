import { describe, expect, it } from "vitest";
import {
  PROVIDER_APIS,
  VENDOR_CATALOG,
  getEndpoint,
  supportedVendorIds,
  type ProviderApi,
} from "@/lib/providers/catalog";
import catalogSnapshot from "@/lib/providers/catalog.snapshot.json";

describe("provider catalog", () => {
  it("contains the supported provider APIs", () => {
    expect(PROVIDER_APIS).toEqual(["openai", "anthropic", "google"]);
  });

  it("contains every required vendor", () => {
    expect(supportedVendorIds()).toEqual([
      "openai",
      "anthropic",
      "google",
      "deepseek",
      "zhipu",
      "minimax",
      "stepfun",
      "moonshot",
      "siliconflow",
      "custom",
    ]);
  });

  it("ensures every vendor has at least one endpoint", () => {
    for (const vendor of Object.values(VENDOR_CATALOG)) {
      const supportedApis = Object.keys(vendor.endpoints) as ProviderApi[];
      expect(supportedApis.length, `${vendor.id} endpoint count`).toBeGreaterThan(0);
    }
  });

  it("returns undefined for unsupported combinations", () => {
    expect(getEndpoint("google", "deepseek")).toBeUndefined();
  });

  it("keeps key provider metadata available for UI auto fill", () => {
    const zhipu = VENDOR_CATALOG.zhipu;

    expect(zhipu.homepage).toBe("https://bigmodel.cn");
    expect(zhipu.apiKeyUrl).toContain("apikeys");
    expect(getEndpoint("openai", "zhipu")?.baseUrl).toBe(
      "https://open.bigmodel.cn/api/paas/v4",
    );
  });

  it("matches the shared Rust mirror snapshot", () => {
    const endpointSnapshot = Object.fromEntries(
      supportedVendorIds().map((vendor) => [
        vendor,
        VENDOR_CATALOG[vendor].endpoints,
      ]),
    );

    expect(endpointSnapshot).toEqual(catalogSnapshot);
  });
});

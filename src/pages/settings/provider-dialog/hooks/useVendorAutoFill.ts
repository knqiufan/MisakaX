import { useEffect } from "react";
import type { ProviderApi, VendorId } from "@/lib/providers/catalog";
import { getEndpoint } from "@/lib/providers/catalog";

interface UseVendorAutoFillOptions {
  provider: ProviderApi;
  vendor: VendorId;
  baseUrl: string;
  baseUrlDirty: boolean;
  onBaseUrlChange: (value: string) => void;
}

export function defaultBaseUrl(provider: ProviderApi, vendor: VendorId): string {
  return getEndpoint(provider, vendor)?.baseUrl ?? "";
}

export function shouldAutoFillBaseUrl(
  nextDefault: string,
  currentBaseUrl: string,
  baseUrlDirty: boolean,
): boolean {
  if (!baseUrlDirty) return true;
  if (currentBaseUrl.trim() === "") return true;
  return currentBaseUrl === nextDefault;
}

export function useVendorAutoFill({
  provider,
  vendor,
  baseUrl,
  baseUrlDirty,
  onBaseUrlChange,
}: UseVendorAutoFillOptions) {
  useEffect(() => {
    const nextDefault = defaultBaseUrl(provider, vendor);
    if (
      baseUrl !== nextDefault &&
      shouldAutoFillBaseUrl(nextDefault, baseUrl, baseUrlDirty)
    ) {
      onBaseUrlChange(nextDefault);
    }
  }, [baseUrl, baseUrlDirty, onBaseUrlChange, provider, vendor]);
}

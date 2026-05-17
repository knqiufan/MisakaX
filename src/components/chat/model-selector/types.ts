import type { VendorId } from "@/lib/providers/catalog";

export interface FlatModel {
  id: string;
  label: string;
  modelId: string;
  provider: string;
  routerName: string;
  vendorId: VendorId;
  vendorLabel: string;
}

export interface ModelGroup {
  vendorId: VendorId;
  vendorLabel: string;
  items: FlatModel[];
}

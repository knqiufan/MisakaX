import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { getEndpoint, type ProviderApi, type VendorId } from "@/lib/providers/catalog";
import { FormField } from "./FormField";

interface ProviderEndpointFieldsProps {
  baseUrl: string;
  provider: ProviderApi;
  vendor: VendorId;
  onBaseUrlChange: (value: string) => void;
}

export function ProviderEndpointFields({
  baseUrl,
  provider,
  vendor,
  onBaseUrlChange,
}: ProviderEndpointFieldsProps) {
  const { t } = useTranslation("settings");
  const endpoint = getEndpoint(provider, vendor);

  return (
    <FormField
      label={t("models.baseUrl")}
      hint={
        endpoint?.baseUrl
          ? t("providers.baseUrl.autoFilled", "Auto-filled from vendor")
          : t("providers.baseUrl.customRequired", "Custom URL required")
      }
    >
      <Input
        value={baseUrl}
        onChange={(event) => onBaseUrlChange(event.target.value)}
        placeholder={endpoint?.baseUrl || t("models.baseUrlPlaceholder")}
      />
    </FormField>
  );
}

import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  getEndpoint,
  PROVIDER_APIS,
  supportedVendorIds,
  VENDOR_CATALOG,
  type ProviderApi,
  type VendorId,
} from "@/lib/providers/catalog";
import { cn } from "@/lib/utils";
import { PROVIDER_API_LABELS, VENDOR_FALLBACK_LABELS } from "./catalog-ui";
import { FormField } from "./FormField";

interface ProviderBasicFieldsProps {
  name: string;
  provider: ProviderApi;
  vendor: VendorId;
  onNameChange: (value: string) => void;
  onProviderChange: (value: ProviderApi) => void;
  onVendorChange: (value: VendorId) => void;
}

export function ProviderBasicFields(props: ProviderBasicFieldsProps) {
  const { t } = useTranslation("settings");

  return (
    <section className="space-y-4">
      <FormField label={t("models.providerName")}>
        <Input
          value={props.name}
          onChange={(event) => props.onNameChange(event.target.value)}
          placeholder={t("models.providerNamePlaceholder")}
        />
      </FormField>

      <FormField label={t("models.providerType")}>
        <RadioGroup
          className="grid grid-cols-3 gap-2"
          value={props.provider}
          onValueChange={(value) => props.onProviderChange(value as ProviderApi)}
        >
          {PROVIDER_APIS.map((api) => (
            <ChoiceCard
              key={api}
              checked={props.provider === api}
              label={PROVIDER_API_LABELS[api]}
              value={api}
            />
          ))}
        </RadioGroup>
      </FormField>

      <FormField label={t("providers.vendor.label", "Provider vendor")}>
        <TooltipProvider>
          <RadioGroup
            className="grid grid-cols-2 gap-2 md:grid-cols-5"
            value={props.vendor}
            onValueChange={(value) => props.onVendorChange(value as VendorId)}
          >
            {supportedVendorIds().map((vendor) => {
              const disabled = !getEndpoint(props.provider, vendor);
              return (
                <ChoiceCard
                  key={vendor}
                  checked={props.vendor === vendor}
                  disabled={disabled}
                  label={vendorLabel(t, vendor)}
                  tooltip={
                    disabled ? t("providers.vendor.unsupported") : undefined
                  }
                  value={vendor}
                />
              );
            })}
          </RadioGroup>
        </TooltipProvider>
      </FormField>
    </section>
  );
}

function ChoiceCard({
  checked,
  disabled,
  label,
  tooltip,
  value,
}: {
  checked: boolean;
  disabled?: boolean;
  label: string;
  tooltip?: string;
  value: string;
}) {
  const card = (
    <label
      className={cn(
        "flex min-h-9 cursor-pointer items-center gap-2 rounded-[var(--radius-ui-md)] border px-3 py-2 text-sm transition-colors",
        checked
          ? "border-primary/55 bg-primary/10 text-foreground"
          : "border-[color:var(--border-strong)] bg-[color:var(--surface-card)] text-muted-foreground hover:bg-[color:var(--surface-hover)]",
        disabled && "cursor-not-allowed opacity-45 hover:bg-[color:var(--surface-card)]",
      )}
    >
      <RadioGroupItem value={value} disabled={disabled} />
      <span className="truncate">{label}</span>
    </label>
  );

  if (!tooltip) return card;

  return (
    <Tooltip>
      <TooltipTrigger asChild>{card}</TooltipTrigger>
      <TooltipContent>{tooltip}</TooltipContent>
    </Tooltip>
  );
}

function vendorLabel(t: ReturnType<typeof useTranslation>["t"], vendor: VendorId) {
  const meta = VENDOR_CATALOG[vendor];
  return t(meta.labelKey, VENDOR_FALLBACK_LABELS[vendor]);
}

import { Eye, EyeOff, ExternalLink, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { VENDOR_CATALOG, type VendorId } from "@/lib/providers/catalog";
import { FormField } from "./FormField";

interface ProviderCredentialFieldsProps {
  apiKey: string;
  isEditing: boolean;
  revealing: boolean;
  showKey: boolean;
  vendor: VendorId;
  onApiKeyChange: (value: string) => void;
  onReveal: () => void;
  onShowKeyChange: (value: boolean) => void;
}

export function ProviderCredentialFields(props: ProviderCredentialFieldsProps) {
  const { t } = useTranslation("settings");
  const meta = VENDOR_CATALOG[props.vendor];

  return (
    <section className="grid gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)]">
      <FormField label={t("providers.vendor.homepage", "Vendor website")}>
        <div className="flex min-h-9 items-center rounded-[var(--radius-ui-md)] border border-[color:var(--border-strong)] bg-[color:var(--surface-card)] px-3 text-sm">
          {meta.homepage ? (
            <a
              className="inline-flex min-w-0 items-center gap-1.5 text-primary hover:underline"
              href={meta.homepage}
              rel="noreferrer"
              target="_blank"
            >
              <span className="truncate">{meta.homepage}</span>
              <ExternalLink className="size-3.5 shrink-0" />
            </a>
          ) : (
            <span className="text-muted-foreground">
              {t("providers.vendor.customEndpoint")}
            </span>
          )}
        </div>
      </FormField>

      <FormField
        label={t("models.apiKey")}
        hint={
          meta.apiKeyUrl && (
            <a
              className="inline-flex items-center gap-1 text-primary hover:underline"
              href={meta.apiKeyUrl}
              rel="noreferrer"
              target="_blank"
            >
              {t("providers.apiKey.get", "Get API Key")}
              <ExternalLink className="size-3" />
            </a>
          )
        }
      >
        <div className="relative">
          <Input
            className="pr-20"
            type={props.showKey ? "text" : "password"}
            value={props.apiKey}
            onChange={(event) => props.onApiKeyChange(event.target.value)}
            placeholder={
              props.isEditing
                ? t("providers.apiKey.keepExisting", "Leave empty to keep current key")
                : t("models.apiKeyPlaceholder")
            }
          />
          <Button
            aria-label={
              props.isEditing && !props.apiKey
                ? t("providers.apiKey.reveal", "Reveal API key")
                : t("providers.apiKey.toggleVisible", "Toggle API key visibility")
            }
            disabled={props.revealing}
            onClick={() => {
              if (props.isEditing && !props.apiKey) {
                props.onReveal();
              } else {
                props.onShowKeyChange(!props.showKey);
              }
            }}
            size="icon"
            variant="ghost"
            className="absolute right-1.5 top-1/2 size-7 -translate-y-1/2"
          >
            {props.revealing ? (
              <Loader2 className="size-3.5 animate-spin" />
            ) : props.showKey ? (
              <EyeOff className="size-3.5" />
            ) : (
              <Eye className="size-3.5" />
            )}
          </Button>
        </div>
      </FormField>
    </section>
  );
}

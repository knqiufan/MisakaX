import { useTranslation } from "react-i18next";
import { FileCode2, Pencil, Trash2, Wifi } from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusPill } from "@/components/settings";
import { cn } from "@/lib/utils";
import type { RouterConfigView } from "@/lib/ipc";
import { VENDOR_CATALOG, type VendorId } from "@/lib/providers/catalog";
import {
  PROVIDER_API_LABELS,
  VENDOR_FALLBACK_LABELS,
} from "./provider-dialog/catalog-ui";

interface ProviderCardProps {
  config: RouterConfigView;
  onEdit: () => void;
  onDelete: () => void;
  onTest: () => void;
  testing?: boolean;
}

export function ProviderCard({
  config,
  onEdit,
  onDelete,
  onTest,
  testing,
}: ProviderCardProps) {
  const { t } = useTranslation("settings");

  return (
    <div
      className={cn(
        "flex h-full flex-col gap-3 rounded-lg border border-border/50 bg-card p-5",
        "transition-colors duration-[var(--ds-dur-fast)] ease-out hover:bg-muted/40"
      )}
    >
      <div className="flex items-start gap-3">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10">
          <ProviderIcon provider={config.provider} />
        </div>
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-center gap-2">
            <span className="truncate text-sm font-medium text-foreground">
              {config.name}
            </span>
            {config.is_active ? (
              <StatusPill
                tone="available"
                label={t("models.defaultProvider")}
              />
            ) : null}
          </div>
          <div className="flex flex-wrap items-center gap-1.5 pl-0 text-[11px] text-muted-foreground">
            <span>{PROVIDER_API_LABELS[config.provider]}</span>
            <span aria-hidden>·</span>
            <span>{vendorLabel(t, config.vendor)}</span>
            {config.model ? (
              <>
                <span aria-hidden>·</span>
                <span className="truncate">{config.model}</span>
              </>
            ) : null}
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-0.5">
          <Button
            aria-label={t("models.testConnection")}
            variant="ghost"
            size="icon"
            onClick={onTest}
            disabled={testing}
            title={t("models.testConnection")}
            className="size-7 text-muted-foreground/80 hover:bg-muted/50 hover:text-foreground"
          >
            <Wifi
              className={cn("size-3.5", testing && "animate-pulse")}
            />
          </Button>
          <Button
            aria-label={t("models.editProvider")}
            variant="ghost"
            size="icon"
            onClick={onEdit}
            title={t("models.editProvider")}
            className="size-7 text-muted-foreground/80 hover:bg-muted/50 hover:text-foreground"
          >
            <Pencil className="size-3.5" />
          </Button>
          <Button
            aria-label={t("models.deleteProvider")}
            variant="ghost"
            size="icon"
            onClick={onDelete}
            title={t("models.deleteProvider")}
            className="size-7 text-muted-foreground/80 hover:bg-destructive/10 hover:text-destructive"
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-2 text-[10px] text-muted-foreground">
        <FileCode2 className="size-3 shrink-0 opacity-60" aria-hidden />
        <span className="truncate font-mono">{config.api_key_masked}</span>
      </div>
    </div>
  );
}

function ProviderIcon({ provider }: { provider: string }) {
  const initial = provider.charAt(0).toUpperCase();
  return (
    <span className="text-sm font-bold text-primary">{initial}</span>
  );
}

function vendorLabel(
  t: ReturnType<typeof useTranslation>["t"],
  vendor: VendorId | null
): string {
  const id = vendor ?? "custom";
  return t(VENDOR_CATALOG[id].labelKey, VENDOR_FALLBACK_LABELS[id]);
}

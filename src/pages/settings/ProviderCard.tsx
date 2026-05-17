import { useTranslation } from "react-i18next";
import { Pencil, Trash2, Wifi, WifiOff } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
    <Card>
      <CardContent className="flex items-center gap-4 p-4">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
          <ProviderIcon provider={config.provider} />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-foreground truncate">
              {config.name}
            </span>
            <Badge variant="outline" className="text-xs">
              {PROVIDER_API_LABELS[config.provider]}
            </Badge>
            <Badge variant="secondary" className="text-xs">
              {vendorLabel(t, config.vendor)}
            </Badge>
            {config.is_active && (
              <Badge variant="default" className="text-xs">
                {t("models.defaultProvider")}
              </Badge>
            )}
          </div>
          <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
            <span className="font-mono">{config.api_key_masked}</span>
            {config.model && <span>· {config.model}</span>}
          </div>
        </div>

        <div className="flex items-center gap-1">
          <Button
            aria-label={t("models.testConnection")}
            variant="ghost"
            size="icon"
            onClick={onTest}
            disabled={testing}
            title={t("models.testConnection")}
          >
            {testing ? (
              <Wifi className="h-4 w-4 animate-pulse" />
            ) : (
              <WifiOff className="h-4 w-4" />
            )}
          </Button>
          <Button
            aria-label={t("models.editProvider")}
            variant="ghost"
            size="icon"
            onClick={onEdit}
            title={t("models.editProvider")}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            aria-label={t("models.deleteProvider")}
            variant="ghost"
            size="icon"
            onClick={onDelete}
            title={t("models.deleteProvider")}
            className="text-destructive hover:text-destructive"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function ProviderIcon({ provider }: { provider: string }) {
  const initial = provider.charAt(0).toUpperCase();
  return (
    <span className="text-sm font-bold text-primary">{initial}</span>
  );
}

function vendorLabel(t: ReturnType<typeof useTranslation>["t"], vendor: VendorId | null): string {
  const id = vendor ?? "custom";
  return t(VENDOR_CATALOG[id].labelKey, VENDOR_FALLBACK_LABELS[id]);
}

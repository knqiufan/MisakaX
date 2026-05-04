import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type {
  RouterConfigView,
  CreateRouterConfig,
  UpdateRouterConfig,
} from "@/lib/ipc";

interface ProviderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editingProvider?: RouterConfigView | null;
  onSubmit: (data: CreateRouterConfig | UpdateRouterConfig) => Promise<void>;
}

const PROVIDER_TYPES = [
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic" },
  { value: "google", label: "Google (Gemini)" },
  { value: "deepseek", label: "DeepSeek" },
  { value: "custom", label: "Custom (OpenAI Compatible)" },
];

const SHOWS_BASE_URL = new Set(["openai", "deepseek", "custom"]);

export function ProviderDialog({
  open,
  onOpenChange,
  editingProvider,
  onSubmit,
}: ProviderDialogProps) {
  const { t } = useTranslation("settings");
  const [loading, setLoading] = useState(false);
  const [showKey, setShowKey] = useState(false);

  const [name, setName] = useState("");
  const [provider, setProvider] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [isActive, setIsActive] = useState(false);

  const isEditing = !!editingProvider;

  useEffect(() => {
    if (open) {
      if (editingProvider) {
        setName(editingProvider.name);
        setProvider(editingProvider.provider);
        setApiKey("");
        setBaseUrl(editingProvider.base_url ?? "");
        setModel(editingProvider.model ?? "");
        setIsActive(editingProvider.is_active);
      } else {
        resetForm();
      }
      setShowKey(false);
    }
  }, [open, editingProvider]);

  function resetForm() {
    setName("");
    setProvider("openai");
    setApiKey("");
    setBaseUrl("");
    setModel("");
    setIsActive(false);
  }

  async function handleSubmit() {
    setLoading(true);
    try {
      if (isEditing) {
        const updates: UpdateRouterConfig = {
          name,
          provider,
          model: model || undefined,
          base_url: baseUrl || undefined,
          is_active: isActive,
        };
        if (apiKey) {
          updates.api_key = apiKey;
        }
        await onSubmit(updates);
      } else {
        const data: CreateRouterConfig = {
          name,
          provider,
          api_key: apiKey,
          model: model || undefined,
          base_url: baseUrl || undefined,
          is_active: isActive,
        };
        await onSubmit(data);
      }
      onOpenChange(false);
    } finally {
      setLoading(false);
    }
  }

  const canSubmit = name.trim() && (isEditing || apiKey.trim());

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEditing ? t("models.editProvider") : t("models.addProvider")}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <FormField label={t("models.providerName")}>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My OpenAI"
            />
          </FormField>

          <FormField label={t("models.providerType")}>
            <Select value={provider} onValueChange={setProvider}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PROVIDER_TYPES.map((pt) => (
                  <SelectItem key={pt.value} value={pt.value}>
                    {pt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </FormField>

          <FormField label={t("models.apiKey")}>
            <div className="relative">
              <Input
                type={showKey ? "text" : "password"}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={
                  isEditing
                    ? "Leave empty to keep current key"
                    : t("models.apiKeyPlaceholder")
                }
                className="pr-10"
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showKey ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </button>
            </div>
          </FormField>

          {SHOWS_BASE_URL.has(provider) && (
            <FormField label={t("models.baseUrl")}>
              <Input
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder={t("models.baseUrlPlaceholder")}
              />
            </FormField>
          )}

          <FormField label={t("models.model")}>
            <Input
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={t("models.modelPlaceholder")}
            />
          </FormField>

          <div className="flex items-center justify-between">
            <Label>{t("models.setDefault")}</Label>
            <Switch checked={isActive} onCheckedChange={setIsActive} />
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={loading}
          >
            {t("common:cancel")}
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!canSubmit || loading}
          >
            {loading ? t("common:loading") : t("common:save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function FormField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label className="text-sm">{label}</Label>
      {children}
    </div>
  );
}

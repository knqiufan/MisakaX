import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { usageIpc } from "@/lib/ipc/usage";

interface UsageDataControlsProps {
  onCleared: () => Promise<void> | void;
}

export function UsageDataControls({ onCleared }: UsageDataControlsProps) {
  const { t } = useTranslation("profile");
  const [open, setOpen] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [error, setError] = useState(false);
  const [clearedCount, setClearedCount] = useState<number | null>(null);

  async function clearHistory() {
    setClearing(true);
    setError(false);
    try {
      const deleted = await usageIpc.clearHistory();
      setClearedCount(deleted);
      setOpen(false);
      await onCleared();
    } catch {
      setError(true);
    } finally {
      setClearing(false);
    }
  }

  return (
    <section className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3" aria-labelledby="usage-data-controls-title">
      <div className="min-w-0">
        <h2 id="usage-data-controls-title" className="text-sm font-medium">
          {t("dataControls.title")}
        </h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          {clearedCount === null
            ? t("dataControls.description")
            : t("dataControls.cleared", { count: clearedCount })}
        </p>
      </div>
      <Dialog open={open} onOpenChange={(nextOpen) => {
        if (!clearing) setOpen(nextOpen);
        if (nextOpen) setError(false);
      }}>
        <DialogTrigger asChild>
          <Button variant="outline" size="sm">
            <Trash2 className="size-3.5" />
            {t("dataControls.action")}
          </Button>
        </DialogTrigger>
        <DialogContent showCloseButton={!clearing}>
          <DialogHeader>
            <DialogTitle>{t("dataControls.dialogTitle")}</DialogTitle>
            <DialogDescription>{t("dataControls.dialogDescription")}</DialogDescription>
          </DialogHeader>
          {error ? (
            <p className="text-xs text-destructive" role="alert">
              {t("dataControls.error")}
            </p>
          ) : null}
          <DialogFooter>
            <DialogClose asChild>
              <Button type="button" variant="outline" disabled={clearing}>
                {t("edit.cancel")}
              </Button>
            </DialogClose>
            <Button
              type="button"
              variant="destructive"
              disabled={clearing}
              onClick={() => void clearHistory()}
            >
              {clearing ? t("dataControls.clearing") : t("dataControls.confirm")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  );
}

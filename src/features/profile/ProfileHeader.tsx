import { FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Pencil } from "lucide-react";

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
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { ProfileAvatar } from "./ProfileAvatar";
import { useProfileStore } from "./profile-store";

export function validateDisplayName(value: string): "empty" | "tooLong" | null {
  const length = Array.from(value.trim()).length;
  if (length === 0) return "empty";
  if (length > 40) return "tooLong";
  return null;
}

export function ProfileHeader() {
  const { t } = useTranslation("profile");
  const profile = useProfileStore((state) => state.profile);
  const status = useProfileStore((state) => state.status);
  const updating = useProfileStore((state) => state.updating);
  const updateError = useProfileStore((state) => state.updateError);
  const load = useProfileStore((state) => state.load);
  const update = useProfileStore((state) => state.update);
  const clearUpdateError = useProfileStore((state) => state.clearUpdateError);
  const [open, setOpen] = useState(false);
  const [displayName, setDisplayName] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  useEffect(() => {
    if (open && profile) setDisplayName(profile.display_name);
  }, [open, profile]);

  function handleOpenChange(nextOpen: boolean) {
    setOpen(nextOpen);
    setValidationError(null);
    clearUpdateError();
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const validation = validateDisplayName(displayName);
    if (validation) {
      setValidationError(t(`edit.errors.${validation}`));
      return;
    }
    setValidationError(null);
    try {
      await update({ displayName: displayName.trim() });
      setOpen(false);
    } catch {
      // The store exposes a stable inline error while keeping the dialog open.
    }
  }

  if (status === "loading" && !profile) {
    return (
      <div className="flex min-h-40 flex-col items-center justify-center gap-3" aria-label={t("loadingProfile")}>
        <Skeleton className="size-20 rounded-full" />
        <Skeleton className="h-7 w-48" />
        <Skeleton className="h-8 w-24 rounded-full" />
      </div>
    );
  }

  if (status === "error" && !profile) {
    return (
      <div className="flex min-h-40 flex-col items-center justify-center gap-3 text-center" role="alert">
        <p className="text-sm font-medium text-foreground">{t("profileLoadError")}</p>
        <p className="max-w-md text-xs text-muted-foreground">{t("profileLoadErrorDescription")}</p>
        <Button variant="outline" size="sm" onClick={() => void load(true).catch(() => undefined)}>
          {t("retry")}
        </Button>
      </div>
    );
  }

  if (!profile) return null;

  return (
    <header className="flex min-h-40 flex-col items-center justify-center gap-3 text-center">
      <ProfileAvatar
        displayName={profile.display_name}
        avatarUrl={profile.avatar_storage_key}
        className="size-20 ring-1 ring-border"
        fallbackClassName="text-2xl"
      />
      <h2 className="max-w-full truncate px-4 text-xl font-semibold tracking-tight text-foreground">
        {profile.display_name}
      </h2>
      <Dialog open={open} onOpenChange={handleOpenChange}>
        <DialogTrigger asChild>
          <Button variant="outline" size="sm">
            <Pencil className="size-3.5" />
            {t("edit.action")}
          </Button>
        </DialogTrigger>
        <DialogContent>
          <form onSubmit={handleSubmit} className="grid gap-4">
            <DialogHeader>
              <DialogTitle>{t("edit.title")}</DialogTitle>
              <DialogDescription>{t("edit.description")}</DialogDescription>
            </DialogHeader>
            <div className="grid gap-2">
              <label htmlFor="profile-display-name" className="text-sm font-medium">
                {t("edit.nameLabel")}
              </label>
              <Input
                id="profile-display-name"
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                autoComplete="name"
                maxLength={80}
                aria-invalid={Boolean(validationError || updateError)}
                aria-describedby="profile-name-help profile-name-error"
                autoFocus
              />
              <p id="profile-name-help" className="text-xs text-muted-foreground">
                {t("edit.nameHelp")}
              </p>
              {validationError || updateError ? (
                <p id="profile-name-error" className="text-xs text-destructive" role="alert">
                  {validationError ?? t("edit.errors.saveFailed")}
                </p>
              ) : null}
            </div>
            <DialogFooter>
              <DialogClose asChild>
                <Button type="button" variant="outline" disabled={updating}>
                  {t("edit.cancel")}
                </Button>
              </DialogClose>
              <Button type="submit" disabled={updating}>
                {updating ? t("edit.saving") : t("edit.save")}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </header>
  );
}

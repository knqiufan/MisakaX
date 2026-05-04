import { useTranslation } from "react-i18next";
import { Bell } from "lucide-react";

export function NotificationsPage() {
  const { t } = useTranslation();

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <Bell className="h-12 w-12 text-muted-foreground/50" />
      <h2 className="text-lg font-medium text-foreground">
        {t("nav:notifications")}
      </h2>
      <p className="text-sm text-muted-foreground">
        {t("comingSoon")}
      </p>
    </div>
  );
}

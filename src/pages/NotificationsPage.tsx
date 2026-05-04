import { useTranslation } from "react-i18next";
import { Bell } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";

export function NotificationsPage() {
  const { t } = useTranslation();

  return (
    <EmptyState
      icon={Bell}
      title={t("nav:notifications")}
      description={t("comingSoon")}
    />
  );
}

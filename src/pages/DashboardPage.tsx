import { useTranslation } from "react-i18next";
import { LayoutDashboard } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";

export function DashboardPage() {
  const { t } = useTranslation();

  return (
    <EmptyState
      icon={LayoutDashboard}
      title={t("nav:dashboard")}
      description={t("comingSoon")}
    />
  );
}

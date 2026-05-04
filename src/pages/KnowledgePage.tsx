import { useTranslation } from "react-i18next";
import { BookOpen } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";

export function KnowledgePage() {
  const { t } = useTranslation();

  return (
    <EmptyState
      icon={BookOpen}
      title={t("nav:knowledge")}
      description={t("comingSoon")}
    />
  );
}

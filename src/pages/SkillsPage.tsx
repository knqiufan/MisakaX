import { useTranslation } from "react-i18next";
import { Sparkles } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";

export function SkillsPage() {
  const { t } = useTranslation();

  return (
    <EmptyState
      icon={Sparkles}
      title={t("nav:skills")}
      description={t("comingSoon")}
    />
  );
}

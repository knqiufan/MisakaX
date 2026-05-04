import { useTranslation } from "react-i18next";
import { Sparkles } from "lucide-react";

export function SkillsPage() {
  const { t } = useTranslation();

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <Sparkles className="h-12 w-12 text-muted-foreground/50" />
      <h2 className="text-lg font-medium text-foreground">
        {t("nav:skills")}
      </h2>
      <p className="text-sm text-muted-foreground">
        {t("comingSoon")}
      </p>
    </div>
  );
}

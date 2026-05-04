import { useTranslation } from "react-i18next";
import { BookOpen } from "lucide-react";

export function KnowledgePage() {
  const { t } = useTranslation();

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <BookOpen className="h-12 w-12 text-muted-foreground/50" />
      <h2 className="text-lg font-medium text-foreground">
        {t("nav:knowledge")}
      </h2>
      <p className="text-sm text-muted-foreground">
        {t("comingSoon")}
      </p>
    </div>
  );
}

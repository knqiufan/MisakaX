import { useTranslation } from "react-i18next";
import { MonolithIcon } from "@/components/brand/MonolithIcon";

export function ChatEmptyState() {
  const { t } = useTranslation("chat");

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8 py-12 text-center">
      <MonolithIcon className="h-9 w-9 text-muted-foreground/50" />
      <div className="max-w-md space-y-2">
        <h2 className="text-base font-medium tracking-tight text-foreground">
          {t("emptyTitle")}
        </h2>
        <p className="text-[13px] leading-relaxed text-muted-foreground">
          {t("emptyDescription")}
        </p>
      </div>
    </div>
  );
}

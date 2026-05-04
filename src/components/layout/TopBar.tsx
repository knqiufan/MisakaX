import { useTranslation } from "react-i18next";
import { useAppStore } from "@/stores";

const PAGE_TITLES: Record<string, string> = {
  chat: "nav:chat",
  skills: "nav:skills",
  knowledge: "nav:knowledge",
  dashboard: "nav:dashboard",
  notifications: "nav:notifications",
  settings: "settings:title",
};

export function TopBar() {
  const route = useAppStore((s) => s.route);
  const { t } = useTranslation();

  const titleKey = PAGE_TITLES[route.page] || "nav:chat";

  return (
    <header className="sticky top-0 z-10 flex h-12 items-center border-b border-border bg-background/80 px-6 backdrop-blur-md">
      <h1 className="text-sm font-semibold text-foreground">
        {t(titleKey)}
      </h1>
      <div className="flex-1" />
    </header>
  );
}

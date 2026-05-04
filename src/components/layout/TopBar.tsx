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
    <header className="sticky top-0 z-[3] flex h-12 shrink-0 items-center border-b border-border px-6 bg-[color:var(--surface-topbar)] backdrop-blur-xl backdrop-saturate-150">
      <h1 className="text-settings-section-title text-foreground">{t(titleKey)}</h1>
      <div className="flex-1" />
    </header>
  );
}

import { useAppStore } from "@/stores/app-store";
import {
  ChatPage,
  KnowledgePage,
  DashboardPage,
  NotificationsPage,
  SettingsPage,
} from "@/pages";

export function ContentArea() {
  const route = useAppStore((s) => s.route);

  switch (route.page) {
    case "chat":
      return <ChatPage />;
    case "knowledge":
      return <KnowledgePage />;
    case "dashboard":
      return <DashboardPage />;
    case "notifications":
      return <NotificationsPage />;
    case "settings":
      return <SettingsPage />;
  }
}

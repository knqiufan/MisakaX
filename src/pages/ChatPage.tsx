import { useTranslation } from "react-i18next";
import { MessageSquare } from "lucide-react";
import { EmptyState } from "@/components/layout/EmptyState";

export function ChatPage() {
  const { t } = useTranslation();

  return (
    <EmptyState
      icon={MessageSquare}
      title={t("nav:chat")}
      description={t("comingSoon")}
    />
  );
}

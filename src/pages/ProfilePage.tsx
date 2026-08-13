import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { ProfileHeader, useProfileStore } from "@/features/profile";
import { UsageOverviewCards, useUsageDashboard } from "@/features/usage-analytics";

export function ProfilePage() {
  const { t } = useTranslation("profile");
  const loadProfile = useProfileStore((state) => state.load);
  const { dashboard, status, error, refresh } = useUsageDashboard();

  useEffect(() => {
    void loadProfile().catch(() => undefined);
  }, [loadProfile]);

  return (
    <div className="min-h-0 flex-1 overflow-y-auto" data-profile-page>
      <div className="mx-auto grid max-w-6xl gap-8 p-6 lg:p-10">
        <ProfileHeader />
        <section className="grid gap-4" aria-labelledby="usage-overview-heading">
          <div>
            <h2 id="usage-overview-heading" className="text-lg font-semibold tracking-tight">
              {t("overview.title")}
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">{t("overview.description")}</p>
          </div>
          <UsageOverviewCards
            dashboard={dashboard}
            status={status}
            error={error}
            onRetry={() => void refresh()}
          />
        </section>
      </div>
    </div>
  );
}

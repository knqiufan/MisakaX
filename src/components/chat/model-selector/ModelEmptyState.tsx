import { Settings } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ModelEmptyStateProps {
  hasModels: boolean;
  noModelsText: string;
  noResultsText: string;
  goToSettingsText: string;
  onGoToSettings: () => void;
}

export function ModelEmptyState({
  hasModels,
  noModelsText,
  noResultsText,
  goToSettingsText,
  onGoToSettings,
}: ModelEmptyStateProps) {
  if (hasModels) {
    return (
      <div className="px-3 py-6 text-center text-xs text-muted-foreground">
        {noResultsText}
      </div>
    );
  }

  return (
    <div className="space-y-3 px-3 py-6 text-center">
      <p className="text-xs text-muted-foreground">{noModelsText}</p>
      <Button size="sm" variant="outline" onClick={onGoToSettings}>
        <Settings className="size-3.5" />
        {goToSettingsText}
      </Button>
    </div>
  );
}

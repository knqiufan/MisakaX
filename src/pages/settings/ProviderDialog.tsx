import { ProviderDialogView } from "./provider-dialog/ProviderDialogView";
import { useProviderDialogController } from "./provider-dialog/useProviderDialogController";
import type { ProviderDialogProps } from "./provider-dialog/types";

export function ProviderDialog(props: ProviderDialogProps) {
  const controller = useProviderDialogController(props);
  return <ProviderDialogView controller={controller} />;
}

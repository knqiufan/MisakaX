import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  sessionsIpc,
  trayIpc,
  type CloseRequestAction,
} from "@/lib/ipc";
import { useAppStore } from "@/stores/app-store";
import { useChatStore } from "@/stores/chat-store";

const CLOSE_REQUESTED_EVENT = "tray:close-requested";
const NEW_TASK_EVENT = "tray:new-task";
const OPEN_SETTINGS_EVENT = "tray:open-settings";
const OPERATION_ERROR_EVENT = "tray:operation-error";

interface TrayNewTaskPayload {
  workingDirectory?: string | null;
}

interface TrayOperationErrorPayload {
  operation: "open_project" | string;
  error: string;
}

interface CloseRequestDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CloseRequestDialog({
  open,
  onOpenChange,
}: CloseRequestDialogProps) {
  const { t } = useTranslation("tray");
  const [remember, setRemember] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setRemember(false);
      setIsSubmitting(false);
    }
  }, [open]);

  const handleOpenChange = useCallback(
    (nextOpen: boolean) => {
      if (!isSubmitting) {
        onOpenChange(nextOpen);
      }
    },
    [isSubmitting, onOpenChange]
  );

  const resolve = useCallback(
    async (action: CloseRequestAction) => {
      setIsSubmitting(true);
      try {
        await trayIpc.resolveCloseRequest(action, remember);
        onOpenChange(false);
      } catch (error) {
        console.error("Failed to resolve close request:", error);
        toast.error(t("closeTitle"), { description: String(error) });
      } finally {
        setIsSubmitting(false);
      }
    },
    [onOpenChange, remember, t]
  );

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent showCloseButton={false} className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("closeTitle")}</DialogTitle>
          <DialogDescription>{t("closeDescription")}</DialogDescription>
        </DialogHeader>

        <div className="flex items-center justify-between gap-4 rounded-md border border-border bg-muted/35 px-3 py-2.5">
          <Label htmlFor="tray-close-remember" className="cursor-pointer leading-snug">
            {t("rememberChoice")}
          </Label>
          <Switch
            id="tray-close-remember"
            size="sm"
            checked={remember}
            disabled={isSubmitting}
            onCheckedChange={setRemember}
          />
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            disabled={isSubmitting}
            onClick={() => handleOpenChange(false)}
          >
            {t("cancel")}
          </Button>
          <Button
            disabled={isSubmitting}
            onClick={() => void resolve("minimize_to_tray")}
          >
            {t("minimize")}
          </Button>
          <Button
            variant="destructive"
            disabled={isSubmitting}
            onClick={() => void resolve("quit")}
          >
            {t("quit")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/** Bridges native tray actions into the single React application shell. */
export function TrayBridge() {
  const { t } = useTranslation("tray");
  const activeSession = useChatStore((state) => state.activeSession);
  const [closeDialogOpen, setCloseDialogOpen] = useState(false);

  const workingDirectory = activeSession?.working_directory ?? null;
  const projectName = activeSession?.project_name ?? null;
  const workspaceKind = activeSession?.workspace_kind ?? null;

  useEffect(() => {
    void trayIpc
      .updateContext({ workingDirectory, projectName, workspaceKind })
      .catch((error) => {
        console.warn("Failed to synchronize tray context:", error);
      });
  }, [projectName, workingDirectory, workspaceKind]);

  const createTask = useCallback(
    async (payload: TrayNewTaskPayload) => {
      useAppStore.getState().navigate({ page: "chat" });

      if (!payload.workingDirectory) {
        useChatStore.getState().openWorkspaceSelector("new-session");
        return;
      }

      try {
        const session = await sessionsIpc.create({
          workingDirectory: payload.workingDirectory,
        });
        useChatStore.getState().upsertSession(session);
      } catch (error) {
        console.error("Failed to create task from tray menu:", error);
        toast.error(t("newTaskFailed"), { description: String(error) });
      }
    },
    [t]
  );

  useEffect(() => {
    let disposed = false;
    let unlisteners: UnlistenFn[] = [];

    const registerListeners = async () => {
      const listeners = await Promise.all([
        listen(CLOSE_REQUESTED_EVENT, () => setCloseDialogOpen(true)),
        listen<TrayNewTaskPayload>(NEW_TASK_EVENT, (event) => {
          void createTask(event.payload);
        }),
        listen(OPEN_SETTINGS_EVENT, () => {
          useAppStore.getState().navigate({ page: "settings", tab: "general" });
        }),
        listen<TrayOperationErrorPayload>(OPERATION_ERROR_EVENT, (event) => {
          if (event.payload.operation === "open_project") {
            toast.error(t("openProjectFailed"), {
              description: event.payload.error,
            });
          }
        }),
      ]);

      if (disposed) {
        listeners.forEach((unlisten) => unlisten());
      } else {
        unlisteners = listeners;
      }
    };

    void registerListeners().catch((error) => {
      console.warn("Failed to register tray listeners:", error);
    });

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [createTask, t]);

  return (
    <CloseRequestDialog
      open={closeDialogOpen}
      onOpenChange={setCloseDialogOpen}
    />
  );
}

import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { FolderOpen } from "lucide-react";
import { MonolithIcon } from "@/components/brand/MonolithIcon";
import { MessageInput } from "@/components/chat/composer/MessageInput";
import { Button } from "@/components/ui/button";
import { useChatStore } from "@/stores/chat-store";
import { sessionsIpc } from "@/lib/ipc";
import type { MessageAttachment } from "@/lib/ipc";

interface NewChatWelcomeProps {
  onSelectWorkspace: () => void;
}

function getTimeOfDayKey(): "morning" | "afternoon" | "evening" | "night" {
  const hour = new Date().getHours();
  if (hour < 6) return "night";
  if (hour < 12) return "morning";
  if (hour < 18) return "afternoon";
  return "evening";
}

const PROMPT_KEYS = ["welcome.prompt1", "welcome.prompt2", "welcome.prompt3"] as const;

export function NewChatWelcome({ onSelectWorkspace }: NewChatWelcomeProps) {
  const { t } = useTranslation("chat");
  const { t: tWorkspace } = useTranslation("workspace");
  const setActiveSessionData = useChatStore((s) => s.setActiveSessionData);
  const setPendingOutbound = useChatStore((s) => s.setPendingOutbound);
  const [creating, setCreating] = useState(false);

  const promptKey = useMemo(() => {
    const idx = Math.floor(Math.random() * PROMPT_KEYS.length);
    return PROMPT_KEYS[idx] ?? PROMPT_KEYS[0];
  }, []);

  const greeting = t(`welcome.${getTimeOfDayKey()}`, {
    defaultValue: t("welcome.hello", { defaultValue: "你好" }),
  });
  const prompt = t(promptKey, {
    defaultValue: t("emptyDescription"),
  });

  const handleSend = useCallback(
    async (
      content: string,
      modelOverride?: string,
      attachments?: MessageAttachment[]
    ) => {
      if (creating) return;
      setCreating(true);
      try {
        setPendingOutbound({ content, modelOverride, attachments });
        const session = await sessionsIpc.create({});
        setActiveSessionData(session);
      } catch (err) {
        setPendingOutbound(null);
        console.error("Failed to create session:", err);
        toast.error(t("errorOccurred"), { description: String(err) });
      } finally {
        setCreating(false);
      }
    },
    [creating, setActiveSessionData, setPendingOutbound, t]
  );

  return (
    <div className="flex flex-1 flex-col items-center justify-center overflow-y-auto px-4 py-8">
      <div className="flex w-full max-w-3xl flex-col items-stretch">
        <div className="mb-8 flex items-center justify-center gap-3">
          <MonolithIcon className="h-9 w-9 text-foreground" />
          <h1 className="text-3xl font-medium leading-none tracking-tight text-foreground">
            {greeting}
            <span className="mx-2 text-muted-foreground/40">·</span>
            <span className="text-muted-foreground">{prompt}</span>
          </h1>
        </div>

        <div className="w-full">
          <MessageInput
            onSend={handleSend}
            onStop={() => undefined}
            disabled={creating}
          />
        </div>

        <div className="mt-6 flex flex-col items-center gap-3 text-center">
          <p className="max-w-md text-xs text-muted-foreground">
            {t("welcome.workspaceHint", {
              defaultValue:
                "可选工作目录后开始 Agent 任务，或直接输入消息创建会话。",
            })}
          </p>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-1.5"
            onClick={onSelectWorkspace}
          >
            <FolderOpen className="size-3.5" />
            {tWorkspace("selectWorkingDir")}
          </Button>
        </div>
      </div>
    </div>
  );
}

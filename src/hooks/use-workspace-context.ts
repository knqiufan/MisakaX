import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { IpcError } from "@/lib/ipc";
import type { DomainEvent, WorkspaceContext } from "@/lib/ipc/contracts";
import {
  WORKSPACE_CONTEXT_CHANGED_EVENT,
  workspaceIpc,
} from "@/lib/ipc/workspace";

export type WorkspaceContextStatus =
  | "idle"
  | "loading"
  | "ready"
  | "stale"
  | "error";

export interface WorkspaceContextState {
  context: WorkspaceContext | null;
  status: WorkspaceContextStatus;
  errorCode: string | null;
  identity: string | null;
}

const IDLE_STATE: WorkspaceContextState = {
  context: null,
  status: "idle",
  errorCode: null,
  identity: null,
};

export function useWorkspaceContext(
  chatSessionId: string | null | undefined,
  workingDirectory: string | null | undefined,
): WorkspaceContextState {
  const [state, setState] = useState<WorkspaceContextState>(IDLE_STATE);
  const requestSequence = useRef(0);
  const activeIdentity = useRef<string | null>(null);

  const load = useCallback(
    async (refresh: boolean) => {
      if (!chatSessionId || !workingDirectory) {
        requestSequence.current += 1;
        activeIdentity.current = null;
        setState(IDLE_STATE);
        return;
      }
      const identity = `${chatSessionId}\u0000${workingDirectory}`;
      const identityChanged = activeIdentity.current !== identity;
      activeIdentity.current = identity;
      const sequence = ++requestSequence.current;
      setState((current) => ({
        context: identityChanged ? null : current.context,
        status: !identityChanged && current.context ? "stale" : "loading",
        errorCode: null,
        identity,
      }));
      try {
        const context = await workspaceIpc.getContext(chatSessionId, refresh);
        if (sequence !== requestSequence.current) return;
        setState((current) => {
          if (
            current.context &&
            current.context.generation > context.generation
          ) {
            return current;
          }
          return { context, status: "ready", errorCode: null, identity };
        });
      } catch (error) {
        if (sequence !== requestSequence.current) return;
        setState((current) => ({
          context: current.context,
          status: "error",
          errorCode: error instanceof IpcError ? error.code : "INTERNAL_ERROR",
          identity,
        }));
      }
    },
    [chatSessionId, workingDirectory],
  );

  useEffect(() => {
    void load(false);
    return () => {
      requestSequence.current += 1;
    };
  }, [load]);

  useEffect(() => {
    if (!chatSessionId || !workingDirectory) return;
    const unlisten = listen<DomainEvent<WorkspaceContext>>(
      WORKSPACE_CONTEXT_CHANGED_EVENT,
      ({ payload: event }) => {
        if (event.aggregateId !== chatSessionId) return;
        if (event.generation !== event.payload.generation) return;
        setState((current) => {
          if (
            current.context &&
            current.context.generation > event.generation
          ) {
            return current;
          }
          return {
            context: event.payload,
            status: "ready",
            errorCode: null,
            identity: `${chatSessionId}\u0000${workingDirectory}`,
          };
        });
      },
    );
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [chatSessionId, workingDirectory]);

  useEffect(() => {
    if (!chatSessionId || !workingDirectory) return;
    const onFocus = () => void load(true);
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [chatSessionId, load, workingDirectory]);

  return state;
}

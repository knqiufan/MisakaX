import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Terminal as XtermTerminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import {
  ClipboardPaste,
  Copy,
  Eraser,
  Loader2,
  RotateCcw,
  Terminal as TerminalIcon,
  X,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import "@xterm/xterm/css/xterm.css";
import "@/styles/terminal.css";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  TERMINAL_EXITED_EVENT,
  TERMINAL_OUTPUT_EVENT,
  type TerminalExitedEvent,
  type TerminalExitReason,
  type TerminalOutputEvent,
} from "@/lib/ipc/terminal";
import {
  attachTerminalRuntime,
  detachTerminalRuntime,
  terminalMatchesTarget,
  useTerminalStore,
  type TerminalTarget,
} from "@/stores/terminal-store";

const DEFAULT_ROWS = 24;
const DEFAULT_COLS = 80;
const RESIZE_DEBOUNCE_MS = 100;
const INPUT_CHUNK_BYTES = 16 * 1024;

interface TerminalPanelProps {
  active: boolean;
  chatSessionId: string;
  workspaceGeneration: number;
  workingDir: string;
  onClose: () => void;
}

export function TerminalPanel({
  active,
  chatSessionId,
  workspaceGeneration,
  workingDir,
  onClose,
}: TerminalPanelProps) {
  const { t } = useTranslation("workspace");
  const hostRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<XtermTerminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const inputQueueRef = useRef(Promise.resolve());
  const resizeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [xtermReady, setXtermReady] = useState(false);
  const [listenersReady, setListenersReady] = useState(false);
  const [hasSelection, setHasSelection] = useState(false);
  const [retainedTargetKey, setRetainedTargetKey] = useState<string | null>(null);

  const status = useTerminalStore((state) => state.status);
  const session = useTerminalStore((state) => state.session);
  const workspaceLabel = useTerminalStore((state) => state.workspaceLabel);
  const exit = useTerminalStore((state) => state.exit);
  const error = useTerminalStore((state) => state.error);
  const ensureStarted = useTerminalStore((state) => state.ensureStarted);
  const restart = useTerminalStore((state) => state.restart);
  const resize = useTerminalStore((state) => state.resize);
  const clearError = useTerminalStore((state) => state.clearError);

  const target = useMemo<TerminalTarget>(
    () => ({
      chatSessionId,
      workspaceGeneration,
      workspaceLabel: summarizeWorkspace(workingDir),
    }),
    [chatSessionId, workspaceGeneration, workingDir],
  );
  const targetKey = `${chatSessionId}\u0000${workspaceGeneration}`;
  const mismatchedWorkspace =
    session !== null && !terminalMatchesTarget(session, target);
  const showWorkspaceChoice =
    active && mismatchedWorkspace && retainedTargetKey !== targetKey;

  const fitAndMeasure = useCallback(() => {
    const terminal = terminalRef.current;
    const fitAddon = fitAddonRef.current;
    const host = hostRef.current;
    if (!terminal || !fitAddon || !host || !active || host.clientWidth === 0) {
      return { rows: DEFAULT_ROWS, cols: DEFAULT_COLS };
    }
    try {
      fitAddon.fit();
      return { rows: terminal.rows, cols: terminal.cols };
    } catch {
      return { rows: DEFAULT_ROWS, cols: DEFAULT_COLS };
    }
  }, [active]);

  useEffect(() => {
    attachTerminalRuntime();
    return detachTerminalRuntime;
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const terminal = new XtermTerminal({
      allowProposedApi: false,
      allowTransparency: false,
      altClickMovesCursor: false,
      convertEol: false,
      cursorBlink: true,
      cursorInactiveStyle: "outline",
      cursorStyle: "block",
      fontFamily: '"Geist Mono Variable", "SFMono-Regular", Consolas, monospace',
      fontSize: 12,
      lineHeight: 1.35,
      linkHandler: {
        activate: (event) => event.preventDefault(),
      },
      logLevel: "off",
      rightClickSelectsWord: true,
      screenReaderMode: false,
      scrollback: 5_000,
      theme: readTerminalTheme(host),
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(host);
    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    const queueInput = (bytes: Uint8Array) => {
      for (let offset = 0; offset < bytes.length; offset += INPUT_CHUNK_BYTES) {
        const chunk = bytes.slice(offset, offset + INPUT_CHUNK_BYTES);
        inputQueueRef.current = inputQueueRef.current.then(() =>
          useTerminalStore.getState().writeBytes(chunk),
        );
      }
    };
    const dataDisposable = terminal.onData((data) => {
      queueInput(encodeTerminalText(data));
    });
    const binaryDisposable = terminal.onBinary((data) => {
      queueInput(binaryStringToBytes(data));
    });
    const selectionDisposable = terminal.onSelectionChange(() => {
      setHasSelection(terminal.hasSelection());
    });
    terminal.attachCustomKeyEventHandler((event) => {
      const primary = event.ctrlKey || event.metaKey;
      const copy = primary && event.key.toLowerCase() === "c";
      const paste = primary && event.key.toLowerCase() === "v";
      if (copy && terminal.hasSelection()) {
        void writeClipboard(terminal.getSelection());
        return false;
      }
      if (paste) {
        void readClipboard().then((text) => {
          if (text) terminal.paste(text);
        });
        return false;
      }
      return true;
    });

    const root = document.documentElement;
    const themeObserver = new MutationObserver(() => {
      terminal.options.theme = readTerminalTheme(host);
    });
    themeObserver.observe(root, {
      attributes: true,
      attributeFilter: ["class", "data-theme", "style"],
    });

    setXtermReady(true);
    const frame = requestAnimationFrame(() => {
      try {
        fitAddon.fit();
      } catch {
        // A hidden panel has no measurable viewport yet.
      }
    });
    return () => {
      cancelAnimationFrame(frame);
      themeObserver.disconnect();
      dataDisposable.dispose();
      binaryDisposable.dispose();
      selectionDisposable.dispose();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      setXtermReady(false);
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlistenOutput: (() => void) | null = null;
    let unlistenExit: (() => void) | null = null;

    void Promise.all([
      listen<TerminalOutputEvent>(TERMINAL_OUTPUT_EVENT, (event) => {
        const accepted = useTerminalStore.getState().acceptOutput(event.payload);
        if (!accepted) return;
        try {
          terminalRef.current?.write(
            decodeTerminalBase64(event.payload.payload.data_base64),
          );
        } catch {
          // Invalid protocol data is discarded and never interpreted as DOM.
        }
      }),
      listen<TerminalExitedEvent>(TERMINAL_EXITED_EVENT, (event) => {
        useTerminalStore.getState().acceptExit(event.payload);
      }),
    ])
      .then(([outputUnlisten, exitUnlisten]) => {
        if (disposed) {
          outputUnlisten();
          exitUnlisten();
          return;
        }
        unlistenOutput = outputUnlisten;
        unlistenExit = exitUnlisten;
        setListenersReady(true);
      })
      .catch(() => {
        if (!disposed) {
          useTerminalStore.setState({
            status: "error",
            error: { code: null, correlationId: null, retryable: true },
          });
        }
      });

    return () => {
      disposed = true;
      setListenersReady(false);
      unlistenOutput?.();
      unlistenExit?.();
    };
  }, []);

  useEffect(() => {
    if (!active || !xtermReady || !listenersReady || workspaceGeneration <= 0) {
      return;
    }
    if (session || status !== "idle") return;
    const dimensions = fitAndMeasure();
    void ensureStarted(target, dimensions.rows, dimensions.cols);
  }, [
    active,
    ensureStarted,
    fitAndMeasure,
    listenersReady,
    session,
    status,
    target,
    workspaceGeneration,
    xtermReady,
  ]);

  useEffect(() => {
    if (!xtermReady) return;
    const host = hostRef.current;
    if (!host) return;
    const observer = new ResizeObserver(() => {
      if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
      resizeTimerRef.current = setTimeout(() => {
        const dimensions = fitAndMeasure();
        void resize(dimensions.rows, dimensions.cols);
      }, RESIZE_DEBOUNCE_MS);
    });
    observer.observe(host);
    return () => {
      observer.disconnect();
      if (resizeTimerRef.current) clearTimeout(resizeTimerRef.current);
      resizeTimerRef.current = null;
    };
  }, [fitAndMeasure, resize, xtermReady]);

  useEffect(() => {
    if (!active || !xtermReady) return;
    const frame = requestAnimationFrame(() => {
      fitAndMeasure();
      terminalRef.current?.focus();
    });
    return () => cancelAnimationFrame(frame);
  }, [active, fitAndMeasure, xtermReady]);

  useEffect(() => {
    setRetainedTargetKey(null);
  }, [targetKey]);

  const handleCopy = useCallback(async () => {
    const terminal = terminalRef.current;
    if (!terminal?.hasSelection()) return;
    await writeClipboard(terminal.getSelection());
    terminal.focus();
  }, []);

  const handlePaste = useCallback(async () => {
    const text = await readClipboard();
    if (!text) return;
    terminalRef.current?.paste(text);
    terminalRef.current?.focus();
  }, []);

  const handleRestart = useCallback(async () => {
    clearError();
    terminalRef.current?.clear();
    const dimensions = fitAndMeasure();
    await restart(target, dimensions.rows, dimensions.cols);
    setRetainedTargetKey(null);
    terminalRef.current?.focus();
  }, [clearError, fitAndMeasure, restart, target]);

  const handleRetry = useCallback(async () => {
    clearError();
    const dimensions = fitAndMeasure();
    await ensureStarted(target, dimensions.rows, dimensions.cols);
  }, [clearError, ensureStarted, fitAndMeasure, target]);

  const liveMessage = error
    ? t("terminal.errorAnnouncement")
    : exit
      ? formatExitMessage(t, exit.exit_code, exit.reason)
      : "";
  const activeWorkspaceLabel = workspaceLabel ?? target.workspaceLabel;

  return (
    <aside
      className="flex h-full min-w-0 flex-col border-l border-border/40 bg-background"
      data-testid="workspace-terminal"
      aria-label={t("terminal.regionLabel")}
    >
      <div className="flex h-10 shrink-0 items-center justify-between gap-2 px-3">
        <div className="flex min-w-0 items-center gap-2">
          <h2 className="flex shrink-0 items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
            <TerminalIcon className="size-3.5" aria-hidden />
            {t("terminal.title")}
          </h2>
          <span className="truncate font-mono text-[10px] text-muted-foreground/70">
            {session
              ? t("terminal.shellSummary", {
                  shell: session.shell_name,
                  workspace: activeWorkspaceLabel,
                })
              : activeWorkspaceLabel}
          </span>
          {mismatchedWorkspace && retainedTargetKey === targetKey ? (
            <span className="shrink-0 rounded-full border border-border/60 bg-muted/50 px-1.5 py-0.5 text-[10px] text-muted-foreground">
              {t("terminal.retained")}
            </span>
          ) : null}
        </div>
        <div className="flex shrink-0 items-center gap-0.5">
          <TerminalToolbarButton
            label={t("terminal.clear")}
            disabled={!session}
            onClick={() => {
              terminalRef.current?.clear();
              terminalRef.current?.focus();
            }}
          >
            <Eraser className="size-3.5" />
          </TerminalToolbarButton>
          <TerminalToolbarButton
            label={t("terminal.copySelection")}
            disabled={!hasSelection}
            onClick={() => void handleCopy()}
          >
            <Copy className="size-3.5" />
          </TerminalToolbarButton>
          <TerminalToolbarButton
            label={t("terminal.paste")}
            disabled={!clipboardAvailable() || status !== "running"}
            onClick={() => void handlePaste()}
          >
            <ClipboardPaste className="size-3.5" />
          </TerminalToolbarButton>
          <TerminalToolbarButton label={t("terminal.collapse")} onClick={onClose}>
            <X className="size-3.5" />
          </TerminalToolbarButton>
        </div>
      </div>

      <div className="relative min-h-0 flex-1 border-t border-border/30">
        <div ref={hostRef} className="misaka-terminal-viewport h-full min-h-0" />
        {status === "starting" || (active && workspaceGeneration <= 0) ? (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center bg-background/88">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" aria-hidden />
              {workspaceGeneration <= 0
                ? t("terminal.waitingContext")
                : t("terminal.starting")}
            </div>
          </div>
        ) : null}
      </div>

      {status === "error" ? (
        <TerminalStatusBar
          message={t("terminal.startFailed")}
          detail={error?.correlationId ? t("terminal.errorId", { id: shortId(error.correlationId) }) : null}
          actionLabel={t("terminal.retry")}
          onAction={() => void handleRetry()}
        />
      ) : null}
      {status === "exited" && exit ? (
        <TerminalStatusBar
          message={formatExitMessage(t, exit.exit_code, exit.reason)}
          detail={null}
          actionLabel={t("terminal.restart")}
          onAction={() => void handleRestart()}
        />
      ) : null}
      {status === "running" && error ? (
        <TerminalStatusBar
          message={t("terminal.commandFailed")}
          detail={error.correlationId ? t("terminal.errorId", { id: shortId(error.correlationId) }) : null}
          actionLabel={t("terminal.dismissError")}
          onAction={clearError}
        />
      ) : null}

      <p className="sr-only" role="status" aria-live="polite" aria-atomic="true">
        {liveMessage}
      </p>

      <Dialog open={showWorkspaceChoice}>
        <DialogContent
          showCloseButton={false}
          onEscapeKeyDown={(event) => event.preventDefault()}
          onPointerDownOutside={(event) => event.preventDefault()}
        >
          <DialogHeader>
            <DialogTitle>{t("terminal.workspaceChangedTitle")}</DialogTitle>
            <DialogDescription>
              {t("terminal.workspaceChangedDescription", {
                workspace: target.workspaceLabel,
              })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setRetainedTargetKey(targetKey)}
            >
              {t("terminal.keepOld")}
            </Button>
            <Button type="button" onClick={() => void handleRestart()}>
              <RotateCcw className="size-4" />
              {t("terminal.restartInNew")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </aside>
  );
}

function TerminalToolbarButton({
  label,
  children,
  ...props
}: {
  label: string;
  children: React.ReactNode;
} & Omit<React.ComponentProps<typeof Button>, "children" | "aria-label">) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-7"
          aria-label={label}
          {...props}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

function TerminalStatusBar({
  message,
  detail,
  actionLabel,
  onAction,
}: {
  message: string;
  detail: string | null;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <div className="flex min-h-10 shrink-0 items-center justify-between gap-3 border-t border-border/40 bg-muted/35 px-3 py-1.5">
      <div className="min-w-0 text-xs text-muted-foreground">
        <span>{message}</span>
        {detail ? <span className="ml-1 font-mono text-[10px]">{detail}</span> : null}
      </div>
      <Button type="button" variant="outline" size="xs" onClick={onAction}>
        {actionLabel}
      </Button>
    </div>
  );
}

export function decodeTerminalBase64(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

export function encodeTerminalText(value: string): Uint8Array {
  return new TextEncoder().encode(value);
}

export function binaryStringToBytes(value: string): Uint8Array {
  return Uint8Array.from(value, (character) => character.charCodeAt(0) & 0xff);
}

function summarizeWorkspace(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  const segments = normalized.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? "workspace";
}

function readTerminalTheme(host: HTMLElement): ITheme {
  const styles = getComputedStyle(host);
  const value = (name: string, fallback: string) =>
    styles.getPropertyValue(name).trim() || fallback;
  const foreground = value("--foreground", "#e8e5df");
  const background = value("--background", "#171615");
  return {
    background,
    foreground,
    cursor: foreground,
    cursorAccent: background,
    selectionBackground: "rgba(127, 120, 108, 0.34)",
    selectionForeground: foreground,
    black: "#242220",
    red: "#d96c68",
    green: "#79b88a",
    yellow: "#c9a86a",
    blue: "#7d9fc2",
    magenta: "#a88bb8",
    cyan: "#6faeae",
    white: "#d9d5ce",
    brightBlack: "#77716a",
    brightRed: "#ef8a83",
    brightGreen: "#91cf9e",
    brightYellow: "#e1c17f",
    brightBlue: "#98b7d8",
    brightMagenta: "#c0a1ce",
    brightCyan: "#87c6c6",
    brightWhite: "#f5f1e9",
  };
}

function clipboardAvailable(): boolean {
  return typeof navigator !== "undefined" && Boolean(navigator.clipboard);
}

async function writeClipboard(value: string): Promise<void> {
  if (!navigator.clipboard) return;
  await navigator.clipboard.writeText(value).catch(() => undefined);
}

async function readClipboard(): Promise<string> {
  if (!navigator.clipboard) return "";
  return navigator.clipboard.readText().catch(() => "");
}

function shortId(value: string): string {
  return value.slice(0, 8);
}

function formatExitMessage(
  t: ReturnType<typeof useTranslation>["t"],
  exitCode: number | null,
  reason: TerminalExitReason,
): string {
  if (exitCode !== null) {
    return t("terminal.exitedWithCode", { code: exitCode });
  }
  return t(`terminal.exitReason.${reason}`);
}

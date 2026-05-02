import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export class IpcError extends Error {
  public readonly command: string;
  public readonly originalError: string;

  constructor(command: string, originalError: string) {
    super(`IPC Error [${command}]: ${originalError}`);
    this.name = "IpcError";
    this.command = command;
    this.originalError = originalError;
  }
}

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (import.meta.env.DEV) {
    console.log(`[IPC] → ${command}`, args ?? {});
  }

  try {
    const result = await tauriInvoke<T>(command, args);

    if (import.meta.env.DEV) {
      console.log(`[IPC] ← ${command}`, result);
    }

    return result;
  } catch (error: unknown) {
    const message = typeof error === "string" ? error : String(error);
    const ipcError = new IpcError(command, message);

    if (import.meta.env.DEV) {
      console.error(`[IPC] ✗ ${command}`, message);
    }

    throw ipcError;
  }
}

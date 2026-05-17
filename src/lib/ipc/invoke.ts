import { invoke as tauriInvoke } from "@tauri-apps/api/core";

const REDACTED = "<redacted>";
const SENSITIVE_KEYS = new Set([
  "apikey",
  "authorization",
  "password",
  "secret",
  "token",
]);

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
    console.log(`[IPC] → ${command}`, sanitizeForLog(args ?? {}));
  }

  try {
    const result = await tauriInvoke<T>(command, args);

    if (import.meta.env.DEV) {
      console.log(`[IPC] ← ${command}`, sanitizeIpcResult(command, result));
    }

    return result;
  } catch (error: unknown) {
    const message = typeof error === "string" ? error : String(error);
    const ipcError = new IpcError(command, message);

    if (import.meta.env.DEV) {
      console.error(`[IPC] ✗ ${command}`, sanitizeForLog(message));
    }

    throw ipcError;
  }
}

export function sanitizeForLog(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => sanitizeForLog(item));
  }

  if (!isRecord(value)) {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [
      key,
      isSensitiveKey(key) ? REDACTED : sanitizeForLog(entry),
    ]),
  );
}

function sanitizeIpcResult(command: string, value: unknown): unknown {
  if (command === "reveal_router_api_key") {
    return REDACTED;
  }

  return sanitizeForLog(value);
}

function isSensitiveKey(key: string): boolean {
  const normalized = key.toLowerCase().replace(/[^a-z0-9]/g, "");
  return SENSITIVE_KEYS.has(normalized) || normalized.endsWith("apikey");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

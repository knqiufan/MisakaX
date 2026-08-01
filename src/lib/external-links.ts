import { open } from "@tauri-apps/plugin-shell";

const MAX_EXTERNAL_URL_LENGTH = 2_048;
const CONTROL_CHARACTER = /[\u0000-\u001f\u007f]/;

/** Return a canonical URL only for explicit, credential-free HTTP(S) links. */
export function normalizeExternalUrl(value: string): string | null {
  if (
    value.length === 0 ||
    value.length > MAX_EXTERNAL_URL_LENGTH ||
    CONTROL_CHARACTER.test(value)
  ) {
    return null;
  }

  try {
    const url = new URL(value);
    if (
      (url.protocol !== "https:" && url.protocol !== "http:") ||
      url.username ||
      url.password
    ) {
      return null;
    }
    return url.href;
  } catch {
    return null;
  }
}

export async function openExternalUrl(value: string): Promise<void> {
  const url = normalizeExternalUrl(value);
  if (!url) throw new Error("External URL is not allowed");
  await open(url);
}

export function handleExternalLinkClick(
  event: { preventDefault: () => void },
  value: string,
): void {
  event.preventDefault();
  void openExternalUrl(value).catch(() => {
    // Keep the current WebView in place. Callers may add local UI feedback later.
  });
}

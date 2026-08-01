export interface FeatureFlags {
  workspaceTerminal: boolean;
  narrowWebviewCapabilities: boolean;
}

type EnvSource = Record<string, string | boolean | undefined>;

function enabled(value: string | boolean | undefined): boolean {
  if (value === true) return true;
  return typeof value === "string" && ["1", "true"].includes(value.trim().toLowerCase());
}

function enabledByDefault(value: string | boolean | undefined): boolean {
  if (value === undefined) return true;
  if (value === false) return false;
  return !(
    typeof value === "string" &&
    ["0", "false"].includes(value.trim().toLowerCase())
  );
}

export function readFeatureFlags(env: EnvSource = import.meta.env): FeatureFlags {
  return {
    workspaceTerminal: enabledByDefault(env.VITE_MISAKAX_WORKSPACE_TERMINAL),
    narrowWebviewCapabilities: enabled(env.VITE_MISAKAX_NARROW_WEBVIEW_CAPABILITIES),
  };
}

export const FEATURE_FLAGS = readFeatureFlags();

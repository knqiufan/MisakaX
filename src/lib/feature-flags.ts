export interface FeatureFlags {
  skillsSettingsTabV2: boolean;
  skillsLazyFilePreview: boolean;
  skillsSecurityGate: boolean;
  skillsActivationView: boolean;
  skillsDeepScanner: boolean;
  workspaceContextBadge: boolean;
  workspaceTerminal: boolean;
  narrowWebviewCapabilities: boolean;
}

type EnvSource = Record<string, string | boolean | undefined>;

function enabled(value: string | boolean | undefined): boolean {
  if (value === true) return true;
  return typeof value === "string" && ["1", "true"].includes(value.trim().toLowerCase());
}

export function readFeatureFlags(env: EnvSource = import.meta.env): FeatureFlags {
  return {
    skillsSettingsTabV2: enabled(env.VITE_MISAKAX_SKILLS_SETTINGS_TAB_V2),
    skillsLazyFilePreview: enabled(env.VITE_MISAKAX_SKILLS_LAZY_FILE_PREVIEW),
    skillsSecurityGate: enabled(env.VITE_MISAKAX_SKILLS_SECURITY_GATE),
    skillsActivationView: enabled(env.VITE_MISAKAX_SKILLS_ACTIVATION_VIEW),
    skillsDeepScanner: enabled(env.VITE_MISAKAX_SKILLS_DEEP_SCANNER),
    workspaceContextBadge: enabled(env.VITE_MISAKAX_WORKSPACE_CONTEXT_BADGE),
    workspaceTerminal: enabled(env.VITE_MISAKAX_WORKSPACE_TERMINAL),
    narrowWebviewCapabilities: enabled(env.VITE_MISAKAX_NARROW_WEBVIEW_CAPABILITIES),
  };
}

export const FEATURE_FLAGS = readFeatureFlags();

import { invoke } from "./invoke";

export type SidecarStatus =
  | "stopped"
  | "starting"
  | "ready"
  | "error"
  | "restarting";

export interface SidecarStatusEvent {
  status: SidecarStatus;
  message: string | null;
  port: number;
}

export async function getSidecarStatus(): Promise<SidecarStatus> {
  return invoke<SidecarStatus>("get_sidecar_status");
}

export async function restartSidecar(): Promise<void> {
  return invoke<void>("restart_sidecar");
}

export const sidecarIpc = {
  getSidecarStatus,
  restartSidecar,
} as const;

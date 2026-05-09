import { invoke } from "./invoke";
import type { Session } from "./types";

export interface CreateSessionParams {
  title?: string;
  model?: string;
  workingDirectory?: string;
}

export interface UpdateSessionWorkingDirParams {
  sessionId: string;
  workingDirectory?: string;
}

export const sessionsIpc = {
  create: (params?: CreateSessionParams) =>
    invoke<Session>("create_session", {
      title: params?.title,
      model: params?.model,
      workingDirectory: params?.workingDirectory,
    }),

  updateWorkingDir: (params: UpdateSessionWorkingDirParams) =>
    invoke<void>("update_session_working_dir", {
      sessionId: params.sessionId,
      workingDirectory: params.workingDirectory,
    }),

  get: (sessionId: string) =>
    invoke<Session>("get_session", { sessionId }),
};

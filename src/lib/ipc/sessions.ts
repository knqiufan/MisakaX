import { invoke } from "./invoke";
import type { Session, UpdateSessionParams } from "./types";

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

  list: (status?: string) =>
    invoke<Session[]>("list_sessions", { status }),

  get: (sessionId: string) =>
    invoke<Session>("get_session", { sessionId }),

  update: (params: UpdateSessionParams) =>
    invoke<void>("update_session", {
      id: params.id,
      title: params.title,
      model: params.model,
      pinned: params.pinned,
      status: params.status,
    }),

  delete: (id: string) => invoke<void>("delete_session", { id }),

  search: (query: string) =>
    invoke<Session[]>("search_sessions", { query }),

  updateWorkingDir: (params: UpdateSessionWorkingDirParams) =>
    invoke<void>("update_session_working_dir", {
      sessionId: params.sessionId,
      workingDirectory: params.workingDirectory,
    }),
};

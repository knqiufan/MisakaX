import { invoke } from "./invoke";
import type {
  Session,
  UpdateSessionParams,
  MessageSearchResult,
  ImportResult,
} from "./types";

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

  pin: (id: string, pinned: boolean) =>
    invoke<void>("pin_session", { id, pinned }),

  archive: (id: string, archived: boolean) =>
    invoke<void>("archive_session", { id, archived }),

  setGroup: (id: string, group: string | null) =>
    invoke<void>("set_session_group", { id, group }),

  listGroups: () =>
    invoke<string[]>("list_session_groups"),

  searchMessages: (
    query: string,
    sessionId?: string,
    limit?: number,
  ) =>
    invoke<MessageSearchResult[]>("search_messages", {
      query,
      sessionId,
      limit,
    }),

  exportSessions: (sessionIds: string[], filePath: string) =>
    invoke<void>("export_sessions", { sessionIds, filePath }),

  importSessions: (filePath: string) =>
    invoke<ImportResult>("import_sessions", { filePath }),
};

import { invoke } from "./invoke";

export interface FsEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
}

export const fsIpc = {
  listDir: (workingDir: string, path: string) =>
    invoke<FsEntry[]>("fs_list_dir", { workingDir, path }),

  readTextFile: (workingDir: string, path: string) =>
    invoke<string>("fs_read_text_file", { workingDir, path }),

  writeTextFile: (workingDir: string, path: string, content: string) =>
    invoke<void>("fs_write_text_file", { workingDir, path, content }),

  revealInExplorer: (workingDir: string, path: string) =>
    invoke<void>("fs_reveal_in_explorer", { workingDir, path }),
};

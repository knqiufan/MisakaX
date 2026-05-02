import { invoke } from "./invoke";
import type {
  RouterConfigView,
  CreateRouterConfig,
  UpdateRouterConfig,
  ConnectionTestResult,
} from "./types";

export const routerConfigsIpc = {
  list: () => invoke<RouterConfigView[]>("list_router_configs"),

  create: (config: CreateRouterConfig) =>
    invoke<string>("create_router_config", { config }),

  update: (id: string, config: UpdateRouterConfig) =>
    invoke<void>("update_router_config", { id, config }),

  delete: (id: string) => invoke<void>("delete_router_config", { id }),

  testConnection: (id: string) =>
    invoke<ConnectionTestResult>("test_router_connection", { id }),
};

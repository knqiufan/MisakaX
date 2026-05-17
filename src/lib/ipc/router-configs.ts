import { invoke } from "./invoke";
import type {
  RouterConfigView,
  CreateRouterConfig,
  CreateRouterConfigWithModels,
  UpdateRouterConfig,
  ConnectionTestResult,
} from "./types";

export const routerConfigsIpc = {
  list: () => invoke<RouterConfigView[]>("list_router_configs"),

  create: (config: CreateRouterConfig) =>
    invoke<string>("create_router_config", { config }),

  createWithModels: (request: CreateRouterConfigWithModels) =>
    invoke<string>("create_router_config_with_models", { request }),

  update: (id: string, config: UpdateRouterConfig) =>
    invoke<void>("update_router_config", { id, config }),

  delete: (id: string) => invoke<void>("delete_router_config", { id }),

  revealApiKey: (id: string) => invoke<string>("reveal_router_api_key", { id }),

  testConnection: (id: string) =>
    invoke<ConnectionTestResult>("test_router_connection", { id }),
};

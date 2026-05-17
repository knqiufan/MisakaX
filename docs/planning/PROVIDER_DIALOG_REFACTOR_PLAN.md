# Provider 配置精细化重构 — 详细实施方案

| 元数据 | 内容 |
| --- | --- |
| 文档目的 | 重构「设置 → 模型」中的 Provider 配置弹窗（双维度类型/供应商、自动填 URL、拉取模型列表、单模型测试、精简版高级配置、Key 回填修复），并**同步修复对话页 `MessageInput` 模型选择器**（按品牌分组、搜索、滚动、空状态、失效回退、保存后自动刷新） |
| 受众 | 前端 + Rust 后端 + Python sidecar 同步实施 |
| 涉及层 | React UI / Tauri Command / SQLite schema / `services/llm` / `crypto` / i18n |
| 创建日期 | 2026-05-17 |
| 最后审阅 | 2026-05-17 |

---

## 0. 需求摘要与价值

### 0.1 用户原始需求（10 条）

1. **Provider 类型**只保留三种：OpenAI / Anthropic / Google。
2. 新增**Provider 供应商**（radio 单选）：OpenAI / Anthropic / Google / DeepSeek / 智谱（GLM）/ MiniMax / 阶跃星辰（StepFun）/ Kimi / 硅基流动（SiliconFlow）/ Custom。
3. 「Provider 类型 × 供应商」二维矩阵自动决定默认 base URL，**抽离为可配置字典**。
4. API Key 前新增「供应商官网」字段，根据供应商自动填入。
5. API Key 表单 label 旁加「获取 API Key」超链接；**修复编辑模式下密钥不回填的 BUG**。
6. 模型字段重命名为「可用模型」，右上角新增「获取模型」按钮：调用接口拉取模型列表 → 多选弹窗 → 回填；支持手填；**只有被显式加入清单的模型才可在对话中使用**。
7. 已选模型行尾增加单模型连通性测试按钮。
8. 折叠的「高级配置」区，内含合理默认值。
9. 弹窗尺寸：宽 ×2，高 ×1.5（即从 `max-w-md` 扩展到 `max-w-3xl`，并控制弹窗高度）。
10. **同步修复 `MessageInput` 模型选择器**：当前下拉显示的不是模型设置里实际配置/选择的模型；按"供应商"分类展示；下拉有最大高度与滚动；下拉内置搜索（详见第 6 章）。

### 0.2 设计目标

| 目标 | 措施 |
| --- | --- |
| **数据驱动** | Provider 元数据集中维护，UI/后端共用一份真理源 |
| **接口风格 ≠ 供应商品牌** | `provider`（接口风格）与 `vendor`（品牌）正交分离，避免互相污染 |
| **安全可回填** | 编辑态可显示明文 API Key（需点击眼睛），但默认掩码 |
| **可测试性** | 单模型连通性测试 / 列表拉取与连接测试解耦 |
| **遵守项目设计规范** | 弹窗、按钮、label 样式严格使用 `docs/design/*` 既有 token 与组件 |
| **代码可维护** | 单方法 ≤ 80 行，组件按职责拆分；常量集中、enum/字典统一管理 |

---

## 1. 基线分析（现状梳理）

### 1.1 前端

| 文件 | 现状 | 待改 |
| --- | --- | --- |
| `src/pages/settings/ProviderDialog.tsx` | 单文件 240 行；provider 用 Select；编辑时 `setApiKey("")` 导致密钥无法回填；`SHOWS_BASE_URL` 硬编码 | 整体重构 + 拆分子组件 |
| `src/pages/settings/ProviderCard.tsx` | `PROVIDER_LABELS` 硬编码 | 复用统一 Provider 字典 |
| `src/pages/settings/ModelSettings.tsx` | 列表 + 弹窗调度 | 接入新数据流（可用模型管理） |
| `src/stores/settings-store.ts` | provider CRUD | 新增 `customModels` 加载/写入 actions |
| `src/lib/ipc/router-configs.ts` | 暴露 5 个 IPC | 新增 `list_models`、`test_model`、`add/list/delete custom_models` 等 |
| `src/lib/ipc/types.ts` | `RouterConfigView` 缺 `vendor`、`api_compat`、`config` 等字段 | 类型扩展 |

### 1.2 后端

| 模块 | 现状 |
| --- | --- |
| `db/migrations.rs` | v2 已加 `router_configs.api_compat` 和 `custom_models` 表；v5 是最新版 |
| `db/repository/router_config_repo.rs` | `RouterConfigView` 缺 `vendor`、`api_compat`、`config_json` 字段，未返回明文 Key |
| `commands/router_configs.rs` | `test_router_connection` 已支持三类，但模型 URL 硬编码且不返回模型列表 |
| `commands/models.rs` | 已有 `list_available_models` / `add_custom_model` / `delete_custom_model` |
| `services/llm/factory.rs` | 已通过 `api_compat` 字段分发 OpenAI 兼容 / Anthropic 兼容 |
| `services/llm/registry.rs` | `builtin_models` 内置写死，但用户改用「拉取 + 手添」之后内置不再是单一真理源 |

**结论**：后端骨架基本就绪，本次重构以**前端为主**；后端需追加：
- `vendor` 字段持久化
- 「获取明文 Key」专用 IPC（仅在编辑弹窗短期内使用）
- 「拉取模型列表」与「单模型测试」两个 IPC
- `is_active`（默认 Provider）与 `custom_models` 的语义对齐：**对话只能选择 `custom_models` 内的模型**

### 1.3 业界调研结论（base URL）

| 供应商 | OpenAI 风格 base URL | Anthropic 风格 base URL | 官网 | API Key 页面 |
| --- | --- | --- | --- | --- |
| OpenAI | `https://api.openai.com/v1` | — | https://openai.com | https://platform.openai.com/api-keys |
| Anthropic | — | `https://api.anthropic.com` | https://anthropic.com | https://console.anthropic.com/settings/keys |
| Google | — | — (走 Gemini 原生) | https://ai.google.dev | https://aistudio.google.com/apikey |
| DeepSeek | `https://api.deepseek.com/v1` | `https://api.deepseek.com/anthropic` | https://deepseek.com | https://platform.deepseek.com/api_keys |
| 智谱 GLM | `https://open.bigmodel.cn/api/paas/v4` | `https://open.bigmodel.cn/api/anthropic` | https://bigmodel.cn | https://bigmodel.cn/usercenter/proj-mgmt/apikeys |
| MiniMax | `https://api.minimaxi.com/v1` | — | https://minimaxi.com | https://platform.minimaxi.com/user-center/basic-information/interface-key |
| 阶跃星辰 StepFun | `https://api.stepfun.com/v1` | — | https://stepfun.com | https://platform.stepfun.com/interface-key |
| Kimi (Moonshot) | `https://api.moonshot.cn/v1` | `https://api.moonshot.cn/anthropic` | https://moonshot.cn | https://platform.moonshot.cn/console/api-keys |
| 硅基流动 SiliconFlow | `https://api.siliconflow.cn/v1` | — | https://siliconflow.cn | https://cloud.siliconflow.cn/account/ak |
| Custom | 由用户填写 | 由用户填写 | — | — |

> "—" 表示该供应商不提供对应接口风格；UI 上需要对未支持的「类型 × 供应商」组合给出禁用 + 提示。

---

## 2. 数据模型与字典设计

### 2.1 决策：常量 / 枚举 / 字典 / DB 哪个最好？

| 方案 | 优 | 劣 | 评分 |
| --- | --- | --- | --- |
| 硬编码在组件 | 简单 | 重复散乱 | ✗ |
| TS `enum` | 类型安全 | 难表达「不支持组合」「URL/Key 页面等多属性」 | △ |
| TS 常量字典 (`Record<...>`) | 灵活、可静态校验、可被 Rust 镜像 | 多语言需走 i18n | ✓ |
| 写入数据库 | 可用户扩展 | Custom 之外的预设无需运行时改动；徒增 schema 复杂度 | ✗ |
| YAML/JSON 配置文件 | 可外部覆盖 | 增加加载与版本管理成本 | △ |

**最终选择**：**TS 常量字典（`src/lib/providers/catalog.ts`）+ Rust 端镜像常量（`src-tauri/src/services/llm/catalog.rs`）**，两侧通过测试用例保证一致性。Custom 走数据库 + 用户输入。

### 2.2 类型设计（前端）

```ts
// src/lib/providers/catalog.ts

/** 接口风格：决定请求/响应协议与请求头 */
export type ProviderApi = "openai" | "anthropic" | "google";

/** 供应商品牌：决定 baseURL/官网/Key 页面/可拉模型列表的端点 */
export type VendorId =
  | "openai" | "anthropic" | "google"
  | "deepseek" | "zhipu" | "minimax"
  | "stepfun" | "moonshot" | "siliconflow"
  | "custom";

export interface VendorEndpoint {
  /** 默认 base URL（用于直接发请求；不含 /models 等子路径） */
  baseUrl: string;
  /** 模型列表 endpoint 子路径，相对 baseUrl；为空表示不支持自动拉取 */
  modelsPath?: string;
}

export interface VendorMeta {
  id: VendorId;
  /** i18n key，最终展示走 t() */
  labelKey: string;
  /** 官网首页 */
  homepage?: string;
  /** API Key 申请页 */
  apiKeyUrl?: string;
  /** 该品牌在不同接口风格下的端点；缺失表示不支持 */
  endpoints: Partial<Record<ProviderApi, VendorEndpoint>>;
}

export const VENDOR_CATALOG: Readonly<Record<VendorId, VendorMeta>> = {
  openai: { /* ... */ },
  zhipu: {
    id: "zhipu",
    labelKey: "providers.vendor.zhipu",
    homepage: "https://bigmodel.cn",
    apiKeyUrl: "https://bigmodel.cn/usercenter/proj-mgmt/apikeys",
    endpoints: {
      openai:    { baseUrl: "https://open.bigmodel.cn/api/paas/v4", modelsPath: "/models" },
      anthropic: { baseUrl: "https://open.bigmodel.cn/api/anthropic" /* 无 list */ },
    },
  },
  custom: {
    id: "custom",
    labelKey: "providers.vendor.custom",
    endpoints: { openai: { baseUrl: "" }, anthropic: { baseUrl: "" }, google: { baseUrl: "" } },
  },
  // ...
};

export function getEndpoint(api: ProviderApi, vendor: VendorId): VendorEndpoint | undefined {
  return VENDOR_CATALOG[vendor].endpoints[api];
}
```

### 2.3 类型设计（Rust 镜像）

```rust
// src-tauri/src/services/llm/catalog.rs

#[derive(Debug, Clone)]
pub struct VendorEndpoint {
    pub base_url: &'static str,
    pub models_path: Option<&'static str>,
}

pub fn endpoint(api: &str, vendor: &str) -> Option<VendorEndpoint> { /* ... */ }
```

> 仅供 Rust 端在「拉取模型 / 测试连通性」时按 vendor 取得默认值，前端用户编辑后的 `base_url` 优先。

### 2.4 数据库 schema 变更（migration v6）

```sql
-- 1) router_configs 表新增 vendor 字段
ALTER TABLE router_configs ADD COLUMN vendor TEXT;

-- 2) router_configs 表新增高级配置 JSON 字段（独立列，便于检索；保留 config_json 作为兼容遗留）
ALTER TABLE router_configs ADD COLUMN advanced_json TEXT;

-- 3) custom_models 表补字段：是否启用 / 排序
ALTER TABLE custom_models ADD COLUMN enabled INTEGER DEFAULT 1;
ALTER TABLE custom_models ADD COLUMN sort_order INTEGER DEFAULT 0;

INSERT INTO _schema_version (version) VALUES (6);
```

**向后兼容**：
- 历史数据 `vendor IS NULL` → 视为 `custom`，避免误归类品牌。
- `advanced_json` 缺失 → 全部回落到默认值。

### 2.5 RouterConfigView 字段扩展

```rust
pub struct RouterConfigView {
    pub id: String,
    pub name: String,
    pub provider: String,           // 接口风格 (openai/anthropic/google)
    pub vendor: Option<String>,     // 供应商品牌
    pub api_key_masked: String,
    pub base_url: Option<String>,
    pub api_compat: Option<String>, // 当 vendor=custom 时由用户选择
    pub advanced: AdvancedConfig,   // 反序列化自 advanced_json
    pub is_active: bool,
    pub created_at: String,
}
```

### 2.6 「可用模型」语义（解决需求 6 的核心约束）

**新约束**：对话发送时，模型选择器只展示**用户在「可用模型」表单中显式加入的项**。
- 「内置模型表」`registry.rs::builtin_models` 退化为**「获取模型」失败时的兜底候选**（只用于弹窗候选，不直接进入会话）。
- `commands/models.rs::list_available_models` 改为：`只返回该 router_config_id 下的 custom_models`。
- `custom_models` 新增 `enabled` 字段：单模型测试失败/用户手动停用时可标记为不可用（UI 灰显但不删除）。

---

## 3. 高级配置项

> **范围决策（最新）**：本期"高级配置"**只实现两项**：`temperature`、`max_tokens`。其余字段（top_p / penalty / proxy / extra_headers / streaming / thinking_budget 等）一律不做，留作未来扩展（参见第 10 节）。

| 字段 | 默认 | 取值 | 适用范围 | 说明 |
| --- | --- | --- | --- | --- |
| `temperature` | `0.7` | `0.0 – 2.0`（slider + 数字框，步长 0.1） | 全部 provider | 创造性 / 多样性 |
| `max_tokens` | 留空（走模型默认） | 正整数；留空表示不传该参数 | 全部 provider | 单次回复 token 上限 |

UI 规范：
- 用 `<Accordion type="single" collapsible>` 折叠，默认 **collapsed**。
- 折叠标题：`t("providers.fields.advanced")`（「高级配置」）。
- 两个字段并排（grid-cols-2），label 简短，右上角带 `i` Tooltip 解释含义。
- `temperature` 用 shadcn `Slider` + 数字 `Input` 双绑；`max_tokens` 用纯 `Input type="number"`，空字符串 → `null`。
- 持久化：序列化为 `AdvancedConfig { temperature: f32, max_tokens: Option<i32> }` 写入 `router_configs.advanced_json`。
- 兼容旧数据：`advanced_json IS NULL` 时使用默认值。

---

## 4. 前端组件拆分

为遵守"单方法 ≤80 行 / SOLID"，新建以下文件：

```
src/pages/settings/provider-dialog/
├── ProviderDialog.tsx          // 容器（仅状态机 + 提交，<80 行）
├── ProviderBasicFields.tsx     // 名称 / 类型 / 供应商 / 官网（受控）
├── ProviderCredentialFields.tsx// API Key + 获取链接 + 显隐切换
├── ProviderEndpointFields.tsx  // base URL（含 vendor 自动填充逻辑）
├── ProviderModelSection.tsx    // 可用模型列表 + 获取按钮 + 行测试
├── FetchModelsDialog.tsx       // 「获取模型」结果多选弹窗
├── ProviderAdvancedFields.tsx  // 折叠的高级配置
├── hooks/
│   ├── useProviderForm.ts      // 表单状态 + 校验 + 提交
│   ├── useVendorAutoFill.ts    // 监听 type/vendor 变化 → 自动填默认 baseURL/官网
│   ├── useFetchModels.ts       // 包装 IPC「拉取模型」
│   └── useModelTest.ts         // 包装 IPC「单模型测试」
└── index.ts                    // 仅 re-export
```

### 4.1 关键交互流程

#### 4.1.1 自动填充策略（useVendorAutoFill）

```
当 type/vendor 变化：
  1. endpoint = getEndpoint(type, vendor)
  2. 若 endpoint 不存在 → 在 UI 上 disable 该供应商在该 type 下（radio + tooltip 说明）
  3. 若用户**未手动改过** baseUrl（dirty 标记），则覆盖为 endpoint.baseUrl
  4. 同步刷新「官网」「获取 API Key」链接显示
  5. vendor === "custom" 时显式让 baseUrl/官网/Key URL 全部可编辑
```

#### 4.1.2 编辑模式 Key 回填修复

新增 IPC `reveal_router_api_key(id)`：
- 仅当**弹窗打开 + 编辑态 + 用户点击"显示密钥"按钮**时才调用
- 返回明文 Key 直接填入 `apiKey` 状态
- 默认仍以 mask 展示，避免无意泄露
- IPC 后端鉴权：依赖 Tauri 进程内调用（OS 级隔离），无额外密码

> 这种"按需揭露"既修复 BUG（编辑后留空仍会保留旧 Key），又不破坏安全姿势。
> 同时**修复原 BUG**：当用户编辑后输入框留空时，提交时不应该清除原 Key（保留 `if (apiKey.trim()) updates.api_key = apiKey` 语义即可）。

#### 4.1.3 「获取模型」流程

```
点击「获取模型」按钮：
  1. 校验：必须已填 API Key + Base URL
  2. 调用 IPC `fetch_provider_models({ api: provider, vendor, base_url, api_key })`
     - 后端按 provider/vendor 选择 endpoint：
       openai:    GET {base}/models          头: Authorization: Bearer {key}
       anthropic: GET {base}/v1/models       头: x-api-key, anthropic-version
       google:    GET {base}/v1beta/models?key={key}
     - 注意 Anthropic 端点：智谱/DeepSeek/Kimi 的 Anthropic 兼容路径未必都暴露 /models；
       未暴露时返回 fallback = registry.builtin_models(provider) 与一个 warning 字段
  3. 成功 → 打开 FetchModelsDialog（带搜索框、批量选）
  4. 失败 → toast.error，详情包含 HTTP status
  5. 用户在弹窗中勾选 → 关闭弹窗后批量写入到「可用模型」本地状态
```

#### 4.1.4 「单模型测试」流程

```
点击模型行尾「闪电图标」：
  1. 调用 IPC `test_model({ api, vendor, base_url, api_key, model_id })`
     - 后端发一条最小 prompt（如 "ping"）+ max_tokens=4 + temperature=0
     - 测量延迟，返回 { success, latency_ms, message }
  2. 成功 → 图标变绿勾 + tooltip 显示延迟
  3. 失败 → 图标变红叉 + tooltip 显示错误
  4. 状态仅前端持有（不入库），切换弹窗后重置
```

#### 4.1.5 保存提交

- **新建态**：提交 `CreateRouterConfig` + 同步把可用模型清单写入 `custom_models`
  - 推荐用一个事务 IPC：`create_router_config_with_models`
- **编辑态**：
  - 更新 `RouterConfig`（含 `vendor` / `advanced_json`）
  - Diff `custom_models`：新增 / 删除 / 启停更新；IPC：`replace_custom_models(router_config_id, models[])`

### 4.2 弹窗尺寸与布局

- `DialogContent` 由 `max-w-md` (28rem) 改为 `max-w-3xl` (48rem)，最小宽 768px；高度通过 `max-h-[85vh]` + 内部 `overflow-y-auto` 实现"约 1.5×"。
- 采用**两栏自适应**：
  - 左栏：基础信息（名称、类型、供应商 radio）
  - 右栏：连接信息（官网、API Key、Base URL）
- 下方占满整行：**可用模型卡片** + **高级配置 Accordion**。
- 严格使用 `docs/design/button-menu-design-spec.md` 的 `--ds-modal-card`、`.ds-modal-button` token；不引入新动画。

### 4.3 i18n 增量

`src/locales/{zh-CN,en-US}/settings.json` 新增节：

```json
"providers": {
  "api": { "openai": "OpenAI", "anthropic": "Anthropic", "google": "Google" },
  "vendor": {
    "openai": "OpenAI", "anthropic": "Anthropic", "google": "Google",
    "deepseek": "DeepSeek", "zhipu": "智谱（GLM）", "minimax": "MiniMax",
    "stepfun": "阶跃星辰（StepFun）", "moonshot": "Kimi", "siliconflow": "硅基流动（SiliconFlow）",
    "custom": "Custom（自定义）"
  },
  "fields": {
    "vendor": "Provider 供应商", "homepage": "供应商官网",
    "getApiKey": "获取 API Key", "availableModels": "可用模型",
    "fetchModels": "获取模型", "advanced": "高级配置",
    "addModelManually": "手动添加模型",
    "modelTestTooltip": "测试此模型可用性"
  },
  "fetchModels": {
    "title": "选择要启用的模型", "search": "搜索模型...",
    "fallbackHint": "供应商未暴露模型列表，已使用内置兜底清单",
    "empty": "未找到任何模型", "selectAll": "全选", "confirm": "确认添加"
  },
  "advanced": {
    "temperature": "温度（Temperature）",
    "temperatureHint": "控制输出随机性，0 严谨、2 发散，默认 0.7",
    "maxTokens": "最大 Token",
    "maxTokensHint": "单次回复 token 上限；留空使用模型默认"
  },
  "errors": {
    "unsupportedCombination": "{{vendor}} 不支持 {{api}} 接口风格",
    "missingApiKey": "请先填写 API Key", "missingBaseUrl": "请先填写接口地址",
    "modelsFetchFailed": "获取模型失败"
  }
}
```

---

## 5. 后端改动

### 5.1 IPC 新增/调整

| Command | 输入 | 输出 | 用途 |
| --- | --- | --- | --- |
| `reveal_router_api_key` | `id: String` | `String` (明文) | 编辑弹窗内"显示原始密钥" |
| `fetch_provider_models` | `{ api, vendor, base_url, api_key }` | `{ models: Vec<ModelInfo>, source: "remote"\|"fallback", warning?: String }` | 拉取模型列表 |
| `test_model` | `{ api, base_url, api_key, model_id, advanced? }` | `{ success, latency_ms, message }` | 单模型连通性 |
| `replace_custom_models` | `{ router_config_id, models: Vec<CreateCustomModel> }` | `()` | 编辑保存时全量替换 |
| `create_router_config_with_models` | `{ config, models }` | `id: String` | 新增时事务写入 |
| `list_custom_models` | `router_config_id: String` | `Vec<CustomModel>` | 编辑态回填 |

`CreateRouterConfig` / `UpdateRouterConfig` 增加 `vendor: Option<String>` 与 `advanced: Option<AdvancedConfig>`（序列化为 `advanced_json`）。

### 5.2 `commands/router_configs.rs` 调整要点

- 抽出 `build_models_url` 到 `services::llm::catalog`，按 `(api, vendor, base_url_override)` 三元组返回最终 URL，避免 `match provider` 硬编码。
- `test_router_connection` 仅做"鉴权可用 + 是否能 ping 通"判定，不再返回模型列表（与新接口职责分离）。
- 拉取模型/单模型测试**统一支持代理**（`AdvancedConfig.proxy_url`）。

### 5.3 `services/llm/registry.rs` 调整

- 公开 `builtin_models(provider)` 仅作为**前端 fetchModelsDialog 的兜底数据源**（通过 `fetch_provider_models` 的 `source: "fallback"` 路径返回）。
- `available_models` 不再合并内置 + 自定义；仅返回 `custom_models WHERE enabled = 1`。
- 受影响的会话流（`commands/chat.rs`）：模型选择器仅展示自定义模型；如果该 router 下 `custom_models` 为空，UI 给出明确空状态。

### 5.4 `services/llm/factory.rs` 调整

- 当 `provider in {openai, anthropic, google}` 但 `vendor != provider` 时（如 `provider=openai, vendor=zhipu`），仍使用对应原生 Provider，**base URL 用 `RouterConfig.base_url`**（vendor 字段不影响请求层，只是 UI 提示用）。
- `vendor=custom` 时按既有逻辑走 `OpenAiCompatProvider` 或 `AnthropicProvider`（由 `api_compat` 决定）。

### 5.5 `crypto` 模块

- 复用现成 `encrypt` / `decrypt`；`reveal_router_api_key` 直接走 `RouterConfigRepo::find_by_id` + `crypto::decrypt`。
- mask 函数保持不变。

### 5.6 Python sidecar 同步

当前 sidecar 暂未直接消费 `vendor`/`advanced`，但 Phase 4 计划中 sidecar 会做对话编排。预留：
- 在 `agent/app/schemas/router_config.py`（若存在）补 `vendor` 与 `advanced` 字段，反序列化时容错（`Optional[...]`）。
- Sidecar 仅用 `provider` + `base_url` + `api_key` + `model` 真正发请求，`vendor` 仅用于日志/遥测。
- 若文件不存在，本期跳过；待 Phase 4 触达再补。

---

## 6. 同步修复：MessageInput 模型选择器

### 6.1 现状缺陷

`src/components/chat/MessageInput.tsx::InputFooter` + `src/components/chat/ModelSelector.tsx` 的现状：

| # | 问题 | 根因 |
| --- | --- | --- |
| F-1 | 选中模型后，下次进入会话或切换 Provider 后，下拉里**显示的不是用户实际配置的模型** | `listAvailable` 在 Phase 2 返回的是"router 自身 `model` 字段 + builtin 兜底"；新方案下数据源已改为 `custom_models`，但前端分组键不对 |
| F-2 | 分组键用的是 `pm.provider.name`（用户起的 router 别名，例如"我的 OpenAI"），导致**同一品牌多 router 时被拆成多组**，不是按"供应商"分类 | 缺 `vendor` 字段 |
| F-3 | 下拉框**没有搜索框** | 未实现 |
| F-4 | 模型多时虽然有 `max-h-[240px]` 滚动，但**高度不够**且滚动条样式与其他面板不一致 | 默认高度偏低 |
| F-5 | 选中的 `selectedModel` 格式为 `${router_config_id}:${model_id}`，如果删除 router 后未清理，`InputFooter` 的 fallback 显示 `model_id` 字面量而不是 placeholder | 缺有效性校验与回退 |

### 6.2 数据流契约（与 Sprint 1/2 对齐）

后端：

```rust
// src-tauri/src/db/repository/router_config_repo.rs
pub struct RouterConfigView {
    pub id: String,
    pub name: String,           // 用户起的别名
    pub provider: String,       // 接口风格
    pub vendor: Option<String>, // **新增：供应商品牌**
    /* ... */
}
```

```rust
// src-tauri/src/commands/models.rs
// list_available_models 改为：仅返回 enabled=1 的 custom_models
// ProviderModels { provider: RouterConfigView, models: Vec<ModelInfo> }
```

前端：

```ts
// src/lib/ipc/models.ts
export interface ProviderModels {
  provider: RouterConfigView; // 含 vendor 字段
  models: ModelInfo[];
}
```

### 6.3 ModelSelector 升级设计

将 `ModelSelector` 从"按 router 别名分组"改为"按 vendor 分组、router 别名作 item 副标题"，并加入搜索框、放大高度、空状态。

**FlatModel 类型扩展**：

```ts
// src/components/chat/ModelSelector.tsx
export interface FlatModel {
  id: string;                 // `${router_config_id}:${model_id}`
  label: string;              // 模型展示名
  vendor: string;             // **品牌 i18n 标签**（如「智谱（GLM）」），用于分组与显示
  vendorId: string;           // vendor 枚举值（zhipu/openai/...），可用于图标
  routerName: string;         // router 别名（"我的 GLM Key"），item 副标题
  modelId: string;            // 用于搜索匹配
}
```

**分组规则**：

```
groupKey = vendor   （展示用 i18n 标签）
  ├─ {label}  · {routerName}     ← 同一 vendor 下多个 router 同模型时显示别名以区分
  └─ ...
```

**搜索匹配字段**：`label` / `modelId` / `routerName` / `vendor`（不区分大小写，子串）。

**布局/尺寸**：

- 容器 `min-w-[280px] max-w-[360px]`（原 `min-w-[200px] max-w-[280px]`）
- 头部固定搜索框，下方列表 `max-h-[360px] overflow-y-auto`（原 240px）
- 滚动条样式：复用 `docs/design/button-menu-design-spec.md` 中已有的滚动条 token（不引入新样式）
- 分组 sticky 标题：`position: sticky; top: 0; bg: var(--surface-popover);`，滚动时不丢失上下文

### 6.4 组件拆分（保证单方法 ≤80 行）

```
src/components/chat/model-selector/
├── ModelSelector.tsx          // 容器（trigger + popover），<60 行
├── ModelSearchInput.tsx       // 搜索框（含清除按钮）
├── ModelGroupList.tsx         // 分组列表渲染（虚拟滚动暂不需要，模型量级 < 200）
├── ModelEmptyState.tsx        // 无配置 / 搜索无结果两态
├── hooks/
│   └── useFilteredModels.ts   // 接收 FlatModel[] + query，返回 [{vendor, items}]
└── index.ts
```

旧 `src/components/chat/ModelSelector.tsx` 改为 re-export 以避免大范围导入路径变更。

### 6.5 InputFooter 修复

```ts
// src/components/chat/MessageInput.tsx::InputFooter
// 1) 把 listAvailable 调用提升到 chat-store（订阅 router 更新事件，避免每次挂载都拉）
// 2) flatModels 用 vendor i18n 标签做分组键
// 3) selectedLabel 严格根据 flatModels 校验：找不到 → 回 placeholder（不再 split 取后缀）
// 4) 当 selectedModel 失效（router 被删 / model 被移除）→ 自动 setSelectedModel(null) 一次
```

### 6.6 与「可用模型」语义的耦合验证

- 当用户在 ProviderDialog 内**新增/启停/移除模型并保存**后，应触发一次 `useChatStore.refreshModels()`：
  - 方案：保存成功 → `settings-store` 派发事件 → `chat-store` 监听 → 重拉 `listAvailable`
  - 实现：`settings-store` 内新增 `bumpModelsVersion()`；`InputFooter` 用 `useChatStore(s => s.modelsVersion)` 作为 `useEffect` 依赖
- 当 `custom_models` 为空时，`InputFooter` 触发器显示 `t("selectModel")`，下拉显示空状态 + "去设置"按钮（跳到模型设置页）

### 6.7 i18n 增量（合入到 `chat.json`）

```json
{
  "selectModel": "选择模型",
  "noModels": "尚未配置任何模型",
  "noModelsHint": "请到「设置 → 模型」中添加 Provider 并启用模型",
  "modelSearch": "搜索模型...",
  "modelSearchEmpty": "未找到匹配的模型",
  "goToSettings": "去设置"
}
```

> `selectModel` / `noModels` / `noModelsHint` 多半已存在；以现有 key 为准，本节只列差量。

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解 |
| --- | --- | --- |
| 各厂商 `/models` 返回结构不一 | 拉取列表失败/字段丢失 | 后端按 `provider`（接口风格）解析，统一映射到 `ModelInfo`；解析失败回落到 builtin |
| Anthropic 兼容路径不暴露 `/v1/models` | 「获取模型」按钮在某些组合下用不了 | 返回 fallback 数据 + 明显 `warning` 提示，引导用户手填 |
| 历史数据 `vendor IS NULL` | 显示错乱 | 视为 `custom`，并在 ProviderCard 上以浅色"未知"角标提示用户去补填 |
| 「可用模型」语义变化 | 既有用户原本能用所有内置模型，升级后空列表 | 迁移脚本：对已有 router_config，自动把对应 `registry::builtin_models(provider)` 全量写入 `custom_models`，保证升级无感 |
| 弹窗扩大后小屏不友好 | 宽度溢出 | 用 `max-w-3xl` + `w-[min(48rem,calc(100vw-2rem))]` 双重保护，并启用 `max-h-[85vh]` 滚动 |
| reveal_api_key 增加泄露面 | 安全 | 仅在弹窗内/手动点击显示后调用；前端不缓存明文超过弹窗生命周期 |

---

## 8. 测试计划

### 8.1 单元测试

- `src-tauri/tests/router_configs_tests.rs`：
  - 创建 + reveal + 编辑（仅改名）→ Key 不变
  - vendor=zhipu/provider=anthropic → base_url 自动取值（在测试上下文调用 `catalog::endpoint`）
  - `replace_custom_models` 的 diff 行为：增/删/启停
- `src-tauri/tests/models_command_tests.rs`：
  - `fetch_provider_models` 走 mock HTTP（用 `wiremock`）
- `src-tauri/tests/llm_factory_tests.rs`：保留 + 新增 vendor=zhipu/provider=openai 用例
- `src/__tests__/provider-catalog.test.ts` (新)：
  - 所有 vendor 都至少配置一个 endpoint
  - `getEndpoint` 缺失组合返回 undefined
- `src/__tests__/use-provider-form.test.ts` (新)：
  - 自动填充：未脏的 baseUrl 被覆盖；脏的不动
  - 编辑态留空 Key → 提交不带 `api_key`

### 8.2 集成 / E2E（可选，Phase 4 联动）

- 真实账号下手动跑：智谱（OpenAI 风格 + Anthropic 风格）、Kimi、DeepSeek、Custom。

---

## 9. 验收清单

- [x] 新建 Provider：能完整走完类型 → 供应商 → 自动填 URL → 拉模型 → 选模型 → 单测 → 保存
- [x] 编辑 Provider：API Key 可揭露/留空保留；可用模型按数据库正确回填
- [x] 所有 9 家供应商 × 支持的接口风格组合，base URL/官网/Key URL 自动填入正确
- [x] 「获取模型」对不支持的端点返回兜底清单 + warning
- [x] 单模型测试图标实时反馈成功/失败/延迟
- [x] 高级配置默认折叠，编辑后保存生效（重新打开数值正确）
- [x] 弹窗宽度约 768px，高度约 85vh 且内部可滚
- [x] 对话页模型选择器只列出 `custom_models` 已启用项
- [x] 模型选择器按 **vendor 品牌** 分组显示，分组标题 sticky 不丢失
- [x] 模型选择器顶部有搜索框，输入可按"名称/router 别名/品牌"过滤；清空恢复全量
- [x] 模型选择器列表 `max-h-[360px]` + 内部滚动，模型量级 >50 时无布局抖动
- [x] `selectedModel` 失效（router 或 model 被删）→ 自动回退到 placeholder
- [x] 模型设置保存后，对话页模型下拉**无需刷新**就能看到最新模型（事件驱动刷新）
- [x] 数据库迁移 v6 在升级路径下不丢数据；既有用户旧 router 自动注入 builtin 模型
- [x] 所有新增方法 ≤ 80 行，文件按 SOLID 拆分
- [x] i18n（zh-CN / en-US）覆盖全部新增文案
- [x] `docs/design/*` 设计规范已更新（弹窗尺寸 + radio group 用法）

---

## 10. 详细 ToDo 列表

按依赖顺序拆为 5 个 Sprint，方便后续按阶段执行/回滚。

### Sprint 1 — 基础设施与数据模型（先于一切 UI）

- [x] **S1-1**：编写 `src-tauri/src/db/migrations.rs::migrate_v6`，新增 `vendor` / `advanced_json` 列、`custom_models.enabled/sort_order` 列
- [x] **S1-2**：补 migration 测试 `src-tauri/tests/db_migrations_tests.rs::v6_*`
- [x] **S1-3**：扩展 `db/models.rs::RouterConfig` 与 `CustomModel` 字段
- [x] **S1-4**：实现 `services/llm/catalog.rs`（Rust 侧 vendor 字典），并写单元测试
- [x] **S1-5**：创建 `src/lib/providers/catalog.ts`（TS 侧字典 + 类型）
- [x] **S1-6**：写 `src/__tests__/provider-catalog.test.ts`（结构性约束 + 与 Rust 镜像一致性 snapshot）
- [x] **S1-7**：扩展 `src/lib/ipc/types.ts`：`RouterConfigView` / `CreateRouterConfig` / `UpdateRouterConfig` / `AdvancedConfig` / `CustomModel` / `FetchModelsResult`

### Sprint 2 — 后端 IPC 与服务层

- [x] **S2-1**：实现 `commands::router_configs::reveal_router_api_key`（按 id 解密返回明文）
- [x] **S2-2**：实现 `commands::models::fetch_provider_models`（按 api+vendor+base_url+key 拉取）
- [x] **S2-3**：实现 `commands::models::test_model`（单模型最小 prompt 测试，含 advanced.proxy）
- [x] **S2-4**：实现 `commands::models::list_custom_models` / `replace_custom_models`（差量更新 + 事务）
- [x] **S2-5**：实现 `commands::router_configs::create_router_config_with_models`（事务版）
- [x] **S2-6**：更新 `services/llm/registry.rs::available_models`：只返回 `custom_models WHERE enabled=1`
- [x] **S2-7**：调整 `services/llm/factory.rs`：处理 `vendor != provider`（如 openai/zhipu）场景
- [x] **S2-8**：升级数据 migration：v6 迁移时对每个已有 router_config 注入 `builtin_models(provider)` 到 `custom_models`（避免老用户出现空列表）
- [x] **S2-9**：在 `src-tauri/src/lib.rs::invoke_handler` 注册所有新 command
- [x] **S2-10**：补 `src-tauri/tests/router_configs_tests.rs` 与 `models_command_tests.rs` 新用例

### Sprint 3 — 前端基础组件与表单状态

- [x] **S3-1**：新建目录 `src/pages/settings/provider-dialog/`
- [x] **S3-2**：在 `src/lib/ipc/router-configs.ts` / `src/lib/ipc/index.ts` 暴露所有新 IPC
- [x] **S3-3**：在 `src/stores/settings-store.ts` 增 `revealApiKey` / `fetchModels` / `testModel` / `replaceModels` actions
- [x] **S3-4**：实现 `hooks/useProviderForm.ts`（受控字段、提交、校验）
- [x] **S3-5**：实现 `hooks/useVendorAutoFill.ts`（dirty 标记 + 联动）
- [x] **S3-6**：实现 `hooks/useFetchModels.ts` 与 `hooks/useModelTest.ts`
- [x] **S3-7**：单测：`__tests__/use-provider-form.test.ts`、`__tests__/use-vendor-auto-fill.test.ts`

### Sprint 4 — 前端 UI 实现

- [x] **S4-1**：实现 `ProviderBasicFields.tsx`（名称 / 类型 radio / 供应商 radio；不支持组合 disable + tooltip）
- [x] **S4-2**：实现 `ProviderCredentialFields.tsx`（官网 + API Key + 获取链接 + 显隐切换；含 reveal 按需调用）
- [x] **S4-3**：实现 `ProviderEndpointFields.tsx`（Base URL，受 `useVendorAutoFill` 控制）
- [x] **S4-4**：实现 `ProviderModelSection.tsx`（列表 + 添加 + 删除 + 行测试图标 + 「获取模型」按钮）
- [x] **S4-5**：实现 `FetchModelsDialog.tsx`（多选弹窗 + 搜索 + 全选 + warning 横条）
- [x] **S4-6**：实现 `ProviderAdvancedFields.tsx`（Accordion 折叠；**仅** `temperature` (Slider+Input) 与 `max_tokens` (number Input)；两字段 grid-cols-2 排布；每字段右上角 Tooltip 解释）
- [x] **S4-7**：重写 `ProviderDialog.tsx` 容器（≤80 行，组合上述子组件 + 提交流）
- [x] **S4-8**：调整 `DialogContent` 尺寸为 `max-w-3xl` + `max-h-[85vh]` + 内部滚动
- [x] **S4-9**：更新 `ProviderCard.tsx`，使用统一 catalog 标签，补 vendor 角标
- [x] **S4-10**：调整 `ModelSettings.tsx`：handleSubmit 走"基本字段 + 模型清单"双写

### Sprint 5 — MessageInput 模型选择器同步修复

- [x] **S5-1**：扩展 `src/lib/ipc/models.ts::ProviderModels.provider`，使其类型对齐新 `RouterConfigView`（含 `vendor`）
- [x] **S5-2**：新建 `src/components/chat/model-selector/` 目录与拆分文件
- [x] **S5-3**：实现 `hooks/useFilteredModels.ts`（query → 分组结果；按 vendor i18n 标签分组）
- [x] **S5-4**：实现 `ModelSearchInput.tsx`（含清除按钮 + 自动聚焦）
- [x] **S5-5**：实现 `ModelGroupList.tsx`（sticky 分组头 + item label 主标 + routerName 副标 + 选中状态）
- [x] **S5-6**：实现 `ModelEmptyState.tsx`（区分"未配置任何模型" / "搜索无结果"两态；前者带"去设置"按钮）
- [x] **S5-7**：重写 `ModelSelector.tsx` 容器（trigger + popover + 组合上述子组件；高度 `max-h-[360px]`、宽度 `min-w-[280px] max-w-[360px]`）
- [x] **S5-8**：旧 `src/components/chat/ModelSelector.tsx` 改为 re-export 兼容路径
- [x] **S5-9**：调整 `MessageInput.tsx::InputFooter`：
  - flatModels 增补 `vendor` / `vendorId` / `routerName` 字段
  - `selectedLabel` 找不到匹配时**严格回退**到 `t("selectModel")`，不再 split 取后缀
  - 监听 `chat-store.modelsVersion` 触发刷新
- [x] **S5-10**：在 `chat-store` 中新增 `modelsVersion: number` + `bumpModels()`；在 `settings-store` 的 provider/model 写入操作成功后调用
- [x] **S5-11**：补 i18n（`chat.json`）新增 key：`modelSearch` / `modelSearchEmpty` / `goToSettings`
- [x] **S5-12**：补单测：
  - `__tests__/use-filtered-models.test.ts`（搜索/分组/排序）
  - `__tests__/model-selector.test.tsx`（空状态 / 选中 / 搜索 / 选中失效自动回退）

### Sprint 6 — i18n / 文档 / 验证

- [x] **S6-1**：补 `src/locales/zh-CN/settings.json` + `chat.json` 全部新增文案
- [x] **S6-2**：补 `src/locales/en-US/settings.json` + `chat.json` 对应英文文案（项目当前语言目录为 `en/`）
- [x] **S6-3**：更新 `docs/design/button-menu-design-spec.md` 第 13 章（新增"扩展尺寸弹窗"小节，约束 max-w-3xl + 85vh 用法）
- [x] **S6-4**：更新 `docs/design/shell-and-workspace-ui-spec.md` 设置页弹窗章节（radio group 间距、Accordion 折叠规范）+ 模型选择器小节（分组/搜索/sticky）
- [x] **S6-5**：在两份 design doc 顶部 bump "最后审阅 / Last reviewed" 日期
- [x] **S6-6**：本地全量回归：依次手动覆盖验收清单第 9 章
- [x] **S6-7**：`cd src-tauri && cargo clean && cargo test`、根目录 `npm run test`、`npm run build` 全绿
- [x] **S6-8**：补 Python sidecar schema 兼容（如适用），否则记录 TODO 到 Phase 4 计划
- [x] **S6-9**：在 `docs/planning/PROVIDER_DIALOG_REFACTOR_PLAN.md` 末尾追加"实施回顾"小节并 sign-off

---

## 11. 不在本期范围

- 多 Provider 一键导入（JSON 配置批量导入）
- 模型用量统计 / 计费面板
- 国际化新增除 zh-CN / en-US 之外的语言
- 在线热加载 vendor catalog（升级即可）
- 高级配置中除 `temperature` / `max_tokens` 以外的字段（top_p、frequency_penalty、presence_penalty、timeout、proxy_url、extra_headers、streaming、thinking_budget 等），`AdvancedConfig` 结构预留 `#[serde(flatten)] extras: serde_json::Value` 字段以便未来扩展时不破坏 schema

---

## 12. 附录：决策记录

| 决策 | 选项 | 选择 | 理由 |
| --- | --- | --- | --- |
| Provider 元数据存储 | 硬编码 / TS 字典 / DB / JSON 文件 | **TS 字典 + Rust 镜像** | 类型安全、可静态校验、零外部依赖 |
| `vendor` 是否影响请求层 | 是 / 否 | **否（仅 UI 与默认值）** | 解耦，避免运行时分支爆炸；请求层仍由 `provider` + `base_url` 决定 |
| Key 回填方式 | 始终下发明文 / 按需 reveal | **按需 reveal** | 安全姿势更佳；不破坏 mask 默认体验 |
| 可用模型语义 | 合并 builtin + custom / 仅 custom | **仅 custom** | 用户显式管理，避免误用；builtin 仅作 fetch fallback |
| 升级路径 | 用户重填 / 自动注入 builtin | **自动注入** | 升级零感知 |

---

## 13. 实施回顾与 Sign-off

**完成日期：** 2026-05-17

### 13.1 交付摘要

- Provider 配置弹窗已完成双维度接口类型/供应商品牌选择、Base URL 自动填充、按需 reveal API Key、获取模型、多选模型、单模型测试、可用模型保存和高级配置折叠编辑。
- 后端已扩展 v6 迁移、provider/vendor catalog、RouterConfig/CustomModel schema、事务式 Provider + models 创建、custom models 替换、fetch/test/reveal IPC，以及只返回已启用 custom models 的 registry 语义。
- 对话页模型选择器已改为按 vendor 品牌分组、顶部搜索、sticky 分组标题、内部滚动、空状态跳设置页、失效 selectedModel 自动回退，并通过 settings-store 到 chat-store 的版本号完成无刷新同步。
- 新增与补强测试覆盖迁移、catalog 镜像一致性、IPC 日志脱敏、表单状态、自动填充、settings/chat store 刷新链路、模型选择器过滤/分组/空状态/清空搜索/去设置入口，以及 Rust registry 和 backend 测试夹具同步。

### 13.2 验证记录

- `npm run build`：通过。仅保留 Vite 大 chunk 警告，非本次功能错误。
- `npm run test`：通过，16 个测试文件、153 个测试。
- `cargo clean`：已按仓库规则在 `src-tauri/` 执行。
- `cargo test`：通过。期间修复了两个测试夹具滞后问题：`llm_backend_tests` 补 `tool_calls` 字段，`llm_registry_tests` 的内存 schema 补 v6 `enabled/sort_order` 字段。
- IDE 诊断：对本轮修改文件检查，无 linter errors。

### 13.3 Python Sidecar 兼容结论

当前 Python sidecar 仍处于 Phase 4 前的 501 占位端点，`agent/app/models.py` 使用 Pydantic v2 且 `extra="ignore"`，本次 Provider Dialog 新增的 provider/vendor/custom model 元数据不会破坏现有 sidecar 请求 schema。后续 DeepAgents 接入时需要继续传递最终解析后的 `model_id`，并按需把 `temperature`、`max_tokens` 映射到 `ChatConfig`；该约束已记录到 `docs/planning/PHASE_4_DETAILED_PLAN.md`。

### 13.4 Sign-off

所有 Sprint TODO 与第 9 章验收清单均已完成并勾选；Sprint 1-5 均完成阶段代码审查与问题修复，Sprint 6 完成文档、i18n、sidecar 兼容记录和全量验证。Provider Dialog Refactor 可进入后续集成或 PR 阶段。

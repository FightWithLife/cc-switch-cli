# OMO / OpenCode 代码架构设计

> 产品 PRD：`../product-prd/20260429_omo_opencode_product_prd.md`

## 1. 现状审计结论

### 1.1 分支状态

```
dev 分支（当前工作分支）
├── OMO：完全没有代码，零 OMO 结构体/路由/DAO
├── OpenCode Provider：有完整的基础表单，但只支持单 model 编辑
└── 通用：TUI 框架、DB、picker 系统已就绪

main 分支
├── OMO：已有 DB-backed template + overlay bindings picker + template editor
├── OpenCode Provider：同 dev
└── 问题：OMO 实现基于 overlay 模式，不符合 PRD 的全页 editor 目标
```

### 1.2 OpenCode Provider 现状

```
当前数据流（dev + main 相同）：

ProviderAddFormState
├── opencode_npm_package    : TextInput
├── opencode_api_key        : TextInput
├── opencode_base_url       : TextInput
├── opencode_model_id       : TextInput     ← 单 model，文本输入
├── opencode_model_name     : TextInput
├── opencode_model_context_limit : TextInput
└── opencode_model_output_limit  : TextInput
         |
         | to_provider_json_value()
         v
Provider.settings_config: Value    ← JSON blob 存入 DB items 表
         |
         | ProviderService::write_live_snapshot()
         v
opencode_config::set_typed_provider()
         |
         v
~/.config/opencode/opencode.json
```

**核心问题**：

- 只支持单 model 编辑，底层 JSON 的 `models` HashMap 被 UI 压成单条
- `opencode_model_id` 是自由文本输入，没有 fetch/picker 集成
- 没有 shared draft 机制，form 通过 `initial_snapshot` 做脏检测
- `ProviderForm -> ModelConfigList -> ModelConfigDetail` 三层页面不存在

### 1.3 OMO Agent Config 现状

```
dev 分支：完全不存在

main 分支现状数据流：

OmoTemplateEditorState（TUI 内存态）
├── template_name / template_options / agent_rows
├── focus: TemplateName | AgentsList | DetailsPane
└── original_* 用于脏检测
         |
         | submit_omo_template_editor()
         v
set_omo_agent_bindings_for_template()
├── DB: set_omo_template_bindings_by_name()
├── settings.json: omo_agent_bindings 镜像
└── oh-my-openagent.json: sync_omo_template_to_oh_my_openagent()
```

**核心问题**：

- main 的 OMO 是 overlay 模式（`Overlay::OmoAgentBindings`），不是全页 editor
- agent 列表硬编码 `DEFAULT_OMO_AGENT_KEYS`（coder, reviewer），没有 catalog 概念
- 没有 stale/orphan 检测机制
- 三边真值（DB / settings.json / oh-my-openagent.json）边界模糊

### 1.4 可复用的基础设施

```
已就绪，可直接复用：
├── TUI 框架：Route 枚举 + NavItem + App 状态机
├── Picker 系统：TuiPicker + overlay_handlers/pickers.rs
├── Model Fetch：runtime_systems/workers.rs 后台线程 + fetch_provider_models_for_tui()
├── DB 框架：SQLite items 表 + DAO 模式 + migration 机制
├── Form 系统：ProviderAddFormState + ProviderAddField 枚举
├── Config 同步：sync_policy.rs + ConfigService::sync_current_providers_to_live()
└── 事务机制：ProviderService::run_transaction() copy-restore 模式
```

---

## 2. 总体架构盒图

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        TUI Layer (cli/tui/)                         │
│                                                                     │
│  ┌──────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │ route.rs │  │   ui/*.rs        │  │  runtime_actions/*.rs    │  │
│  │          │  │                  │  │                          │  │
│  │ Route    │  │ render_content() │  │ handle_key() dispatch    │  │
│  │ enum     │→ │ match route      │→ │ per-route key handlers   │  │
│  └──────────┘  └──────────────────┘  └──────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────┐  ┌─────────────────────────────────┐ │
│  │  app/types.rs            │  │  form/*.rs                      │ │
│  │                          │  │                                 │ │
│  │  OmoTemplateEditorState  │  │  ProviderAddFormState           │ │
│  │  OmoAgentBindingsState   │  │  + OpenCode model list/draft   │ │
│  │  Overlay enum            │  │  + OMO template draft           │ │
│  └──────────────────────────┘  └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Service Layer (services/)                        │
│                                                                     │
│  ┌──────────────────────┐  ┌────────────────────────────────────┐  │
│  │ services/provider/   │  │ services/config.rs                 │  │
│  │                      │  │                                    │  │
│  │ ProviderService      │  │ ConfigService                      │  │
│  │ - add/update/delete  │  │ - sync_current_providers_to_live() │  │
│  │ - import_default     │  │ - sync_omo_template_to_live()     │  │
│  │ - write_live_snapshot│  │                                    │  │
│  └──────────────────────┘  └────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Persistence Layer                                  │
│                                                                     │
│  ┌──────────────────┐  ┌────────────────┐  ┌────────────────────┐  │
│  │ database/dao/    │  │ settings.rs    │  │ *_config.rs        │  │
│  │                  │  │                │  │                    │  │
│  │ providers.rs     │  │ AppSettings    │  │ opencode_config.rs │  │
│  │ omo.rs (新增)    │  │ 读写 JSON 配置 │  │ 读写 opencode.json │  │
│  │                  │  │                │  │                    │  │
│  │ SQLite items 表  │  │ settings.json  │  │ oh-my-openagent.js │  │
│  │ + omo_templates  │  │                │  │ (OMO live target)  │  │
│  └──────────────────┘  └────────────────┘  └────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. OpenCode Provider 升级设计

### 3.1 接入点总览

```text
需要改动的文件                          改动性质
─────────────────────────────────────────────────
provider.rs                    修改    新增 OpenCodeModelEntry 结构体
cli/tui/form.rs                修改    ProviderAddField 新增 ModelConfig 入口
cli/tui/form/provider_state.rs 修改    OpenCode 字段列表改为 3 层入口
cli/tui/form/provider_json.rs  修改    序列化支持多 model + currentModel
cli/tui/form/provider_state_loading.rs 修改  加载所有 model 到 draft
cli/tui/app/types.rs           修改    新增 ModelConfigList/Detail 状态
cli/tui/route.rs               修改    新增 ModelConfigList / ModelConfigDetail 路由
cli/tui/ui.rs                  修改    render_content 分发新路由
cli/tui/ui/forms/provider.rs   修改    渲染 ModelConfig 入口行
cli/tui/ui/providers.rs        新增    ModelConfigList 页面渲染
cli/tui/ui/providers.rs        新增    ModelConfigDetail 页面渲染
cli/tui/runtime_actions/providers.rs 修改 ModelConfig 按键处理
cli/tui/app/overlay_handlers/pickers.rs 修改 Model ID picker 嵌入 detail 页
cli/tui/runtime_systems/types.rs 修改  fetch 结果写入 draft 而非直接写 field
services/provider/mod.rs       修改    保存时处理 multi-model + currentModel
database/dao/providers.rs      可能修改 存储结构变化适配
cli/i18n/texts/provider_editor.rs 修改 新增 ModelConfig 相关文案
```

### 3.2 数据结构变更

```text
当前（单 model）：
Provider.settings_config = {
  "npm": "@ai-sdk/openai-compatible",
  "options": { "baseURL": "...", "apiKey": "..." },
  "models": {
    "gpt-4o": { "name": "GPT-4o", "limit": { "context": 128000 } }
  }
}

目标（multi-model + currentModel）：
Provider.settings_config = {
  "npm": "@ai-sdk/openai-compatible",
  "options": { "baseURL": "...", "apiKey": "..." },
  "models": {
    "gpt-5.4":   { "name": "GPT-5.4", "limit": { "context": 200000, "output": 65536 } },
    "gpt-4.1":   { "name": "GPT-4.1" },
    "gpt-4o-mini": { "name": "GPT-4O-MINI", "limit": { "context": 128000 } }
  },
  "currentModel": "gpt-5.4"
}
```

**新增内部 draft 结构**：

```text
OpenCodeModelDraft（内存态，不直接落盘）
├── model_id: String              ← object key，也是纯 model id
├── model_name: String            ← 展示名
├── input_limit: Option<u64>      ← limit.context
├── output_limit: Option<u64>     ← limit.output
└── original_model_id: Option<String>  ← rename 追踪
```

```text
OpenCodeProviderDraft（shared editable session）
├── provider_id: String
├── name: String
├── base_url: String
├── api_key: String
├── npm: String
├── models_by_id: IndexMap<String, OpenCodeModelDraft>
├── current_model: String
├── dirty: bool
└── last_loaded_from_db: Value     ← DB snapshot hash，用于重入检测
```

### 3.3 页面层级与路由

```text
当前路由：
Route::ProviderDetail { id }
  └── 单页面，所有字段平铺（包括 4 个 model 字段）

目标路由：
Route::ProviderDetail { id }
  └── Provider Form（Name / BaseURL / APIKey / ModelConfig 入口行）
        │
        │ Enter on ModelConfig
        ▼
Route::OpenCodeModelConfigList { provider_id }
  └── Model 列表（以 model name 展示）
        │
        │ Enter on model / N for new
        ▼
Route::OpenCodeModelConfigDetail { provider_id, model_id }
  └── Model 详情（ModelName / ModelID / InputLimit / OutputLimit）
        │
        │ Enter on ModelID
        ▼
Overlay::ModelFetchPicker（复用现有 picker 基础设施）
```

### 3.4 数据流盒图

```text
┌─────────────────────────────────────────────────────────────────┐
│                    OpenCode Provider 编辑流                       │
│                                                                 │
│  进入 ProviderDetail                                            │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────┐                        │
│  │ 初始化 OpenCodeProviderDraft        │                        │
│  │ 从 DB 读取 Provider.settings_config │                        │
│  │ 解析 models HashMap → models_by_id  │                        │
│  └─────────────────────────────────────┘                        │
│       │                                                         │
│       ▼                                                         │
│  Provider Form 展示 draft 的 base fields                        │
│  ModelConfig 行展示 model count                                  │
│       │                                                         │
│       ├── Enter on field → 底部输入框编辑 → Enter 提交到 draft    │
│       ├── Enter on ModelConfig → 进入 ModelConfigList            │
│       └── Ctrl+S → 完整提交                                      │
│                │                                                │
│                ▼                                                │
│  ┌─────────────────────────────────────┐                        │
│  │ 保存流程                            │                        │
│  │ 1. draft → Provider.settings_config │                        │
│  │ 2. ProviderService::update()        │                        │
│  │    a. 写 DB (items 表)              │                        │
│  │    b. 写 opencode.json              │                        │
│  │ 3. 成功 → 从 DB 重读刷新 draft      │                        │
│  │    失败 → 保留 draft，显示错误       │                        │
│  └─────────────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

### 3.5 Model ID Fetch 集成

```text
当前 fetch 流程：
Form field 按 Enter
    → Action::ProviderModelFetch
    → runtime_systems/workers.rs 后台线程
    → fetch_provider_models_for_tui()
    → 结果写入 Overlay::ModelFetchPicker
    → 用户选择 → 直接写入 TextInput field

目标 fetch 流程（嵌入 ModelConfigDetail 页）：
ModelConfigDetail 页 ModelID 字段按 Enter
    → 前置检查：draft.base_url 非空 + draft.api_key 满足鉴权
    │
    ├── 不满足 → 显示 blocked feedback（"Base URL or API Key is missing"）
    │
    └── 满足 → 复用现有 fetch 基础设施
              → 结果写入 Overlay::ModelFetchPicker（复用）
              → 过滤：从 fetch 结果中移除当前 draft.models_by_id 中已存在的 model ID
                 （已配置的 model 不重复出现在 picker 中）
              → 已配置的 model ID 在 picker 中标记为 [configured]（仅视觉提示，不可选）
              → 用户选择 → 写入 draft.models_by_id[current].model_id
              → 若 model_name 为空，自动回填为 model_id 大写
              → draft.dirty = true
```

**关键复用点**：

- `runtime_systems/types.rs` 的 `fetch_provider_models_for_tui()` 完全复用
- `runtime_systems/types.rs` 的 `parse_model_ids_from_response()` 完全复用
- `runtime_systems/workers.rs` 的 `ModelFetchSystem` 后台线程完全复用
- `overlay_handlers/pickers.rs` 的 `handle_model_fetch_picker_key()` 需要小改：结果写入 draft 而非 TextInput

### 3.5A 新建 Model 校验规则

```text
新建 model 的保存校验：

[Ctrl+S in ModelConfigDetail]
    │
    ▼
[model_id 为空?]
    │
    ├── Yes → 显示错误 "Model ID is required"
    │         留在 detail 页，不清空用户输入
    │
    └── No → [model_id 与同 provider 下其他 model 重复?]
              │
              ├── Yes → 显示错误 "Duplicated model id `xxx`"
              │         留在 detail 页，不清空用户输入
              │
              └── No → [model_name 为空?]
                        │
                        ├── Yes → 自动回填为 model_id 大写
                        │
                        └── No → 保持用户输入

校验错误展示方式：底部输入区显示红色错误文案，不使用 toast（避免丢失上下文）
```

### 3.5B Model ID Rename 原子性

```text
rename 判定条件：
├── original_model_id 有值（编辑已有 model）
├── current model_id != original_model_id（用户修改了 ID）
└── 两个条件同时满足 → 本次保存是 rename 操作

新建判定条件：
├── original_model_id 为 None（新创建的 model）
└── 本次保存是 insert 操作，不触发 rename 逻辑

rename 原子性保证：
1. 从 draft.models_by_id 中移除 original_model_id key
2. 插入 new model_id key（保留所有其他字段）
3. 若 original_model_id == current_model → current_model = new model_id
4. 上述三步在 draft 层面一次性完成
5. 再通过 ProviderService::update() 写入 DB + JSON（ProviderService 本身有事务保证）
6. 任何一步失败 → draft 保留用户输入，DB 和 JSON 不变

original_model_id 设置时机：
├── 进入 ModelConfigDetail 编辑已有 model → original_model_id = model_id
├── 进入 ModelConfigDetail 新建 model → original_model_id = None
└── 保存成功后 → original_model_id = 新的 model_id（重置为当前值）

### 3.6 currentModel 管理规则

```text
删除 model 后的回退逻辑：

[Delete Model]
    │
    ▼
[Is deleted model == currentModel?]
    │
    ├── No → 直接删除，currentModel 不变
    │
    └── Yes → [remaining models 非空?]
              │
              ├── Yes → currentModel = remaining modelIds 中 ASCII 最小项
              │
              └── No  → currentModel = ""

rename model 后的修复逻辑：

[Save with new model_id]
    │
    ▼
[old model_id == currentModel?]
    │
    ├── Yes → currentModel = new model_id（同一次提交）
    │
    └── No  → currentModel 不变
```

### 3.7 优化建议

1. **消除 ProviderAddFormState 中的单 model 字段**：OpenCode 不再使用 `opencode_model_id/name/context_limit/output_limit` 四个 TextInput，改为从 draft 读写
2. **ModelConfig 入口行复用 ProviderAddField 模式**：新增 `ProviderAddField::OpenCodeModelConfig`，在字段列表中替代 4 个 model 字段
3. **picker 结果回调抽象**：当前 picker 结果直接写入 `TextInput`，需要抽象为 `PickerResult` trait，支持写入 draft
4. **保存时统一走 ProviderService**：当前 OpenCode 保存直接调 `opencode_config::set_typed_provider()`，应统一走 `ProviderService::update()` + `write_live_snapshot()`

---

## 4. OMO Agent Config 设计

### 4.1 接入点总览

```text
dev 分支需要全部新建：
─────────────────────────────────────────────────
database/schema.rs             修改    追加 omo_templates / omo_template_bindings 表
database/dao/omo.rs            新增    OMO DAO（CRUD / import / sync）
database/dao/mod.rs            修改    pub mod omo
settings.rs                    修改    OmoAgentBinding 结构体 + oh-my-openagent.json 读写
cli/tui/route.rs               修改    新增 OmoAgentConfig / OmoTemplateEditor 路由
cli/tui/app/menu.rs            修改    NavItem 新增 OmoAgentConfig（OpenCode only）
cli/tui/app/types.rs           修改    OMO 相关状态结构体
cli/tui/app.rs                 修改    App 新增 omo_editor / omo_draft 字段
cli/tui/ui.rs                  修改    render_content 分发 OMO 路由
cli/tui/ui/omo_landing.rs      新增    OMO Landing 页面渲染
cli/tui/ui/omo_editor.rs       新增    OMO Template Editor 页面渲染
cli/tui/ui/omo_stale.rs        新增    Stale/Orphan 状态渲染
cli/tui/runtime_actions/omo.rs 新增    OMO 按键处理（Landing + Editor）
cli/tui/app/helpers.rs         修改    build_omo_* 辅助函数
cli/tui/data.rs                修改    ConfigSnapshot 新增 OMO 字段
cli/i18n/texts/omo.rs          新增    OMO 专用文案
services/config.rs             修改    新增 sync_omo_template_to_live()
```

### 4.2 数据结构

```text
OmoAgentBinding（已有定义，直接复用）
├── provider_id: String
└── model_id: String
```

```text
OmoTemplateDraft（内存态，editor 的 shared draft）
├── template_id: Option<i64>       ← None = 新建
├── template_name: String
├── is_active: bool
├── bindings: IndexMap<String, OmoAgentBinding>   ← agent_key → binding
├── agent_catalog: Vec<String>     ← 从内置 catalog 加载
├── dirty: bool
└── original_snapshot: (String, IndexMap<String, OmoAgentBinding>)  ← 脏检测用
```

```text
OmoStaleState（per-agent stale 检测结果）
├── agent_key: String
├── binding: OmoAgentBinding
├── status: OmoBindingStatus
│   ├── Bound              ← 正常
│   ├── MissingProvider    ← provider 不存在于当前 DB providers
│   ├── MissingModel       ← provider 存在但 model 不可用
│   └── OrphanAgentKey     ← agent_key 不在内置 catalog 中
└── resolved: bool         ← 用户已处理（rebind/clear/discard）
```

### 4.3 DB Schema

```text
新增表：omo_templates
┌─────────────────────────────────────────────────┐
│ id              INTEGER PRIMARY KEY AUTOINCREMENT│
│ name            TEXT NOT NULL                     │
│ is_active       BOOLEAN NOT NULL DEFAULT 0        │
│ source          TEXT NOT NULL DEFAULT 'db'         │
│ last_sync_error TEXT                              │
│ last_synced_at  INTEGER                           │
│ created_at      INTEGER NOT NULL                  │
│ updated_at      INTEGER NOT NULL                  │
└─────────────────────────────────────────────────┘
唯一索引：is_active = 1 只允许一行

新增表：omo_template_bindings
┌─────────────────────────────────────────────────┐
│ template_id     INTEGER NOT NULL (FK → omo_templates.id) │
│ agent_key       TEXT NOT NULL                     │
│ provider_id     TEXT NOT NULL                     │
│ model_id        TEXT NOT NULL                     │
│ binding_state   TEXT NOT NULL DEFAULT 'bound'     │
│ created_at      INTEGER NOT NULL                  │
│ updated_at      INTEGER NOT NULL                  │
│ PRIMARY KEY (template_id, agent_key)              │
└─────────────────────────────────────────────────┘

新增 migration：v_current → v_next
├── CREATE TABLE omo_templates
├── CREATE TABLE omo_template_bindings
├── CREATE UNIQUE INDEX idx_omo_templates_single_active
└── INSERT legacy import marker check
```

### 4.4 路由与页面层级

```text
当前（dev）：
无 OMO 相关路由

目标路由：
Route::OmoAgentConfig
  └── OMO Landing（template 列表 + template 级动作）
        │
        ├── Enter / n → 进入 editor
        ├── s → 切换 active + sync + 刷新
        ├── c → 复制 + 进入 editor
        ├── r / Ctrl+J → 重读 DB + sync JSON + 刷新 UI
        ├── Del → 确认 → 删除 → 重选 active
        │
        ▼
Route::OmoTemplateEditor
  └── 全页 Template Editor
        ├── Template Name
        ├── Agents List（左侧）
        ├── Agent Details（右侧）
        │     ├── Provider 字段 → Enter → Overlay::OmoProviderPicker
        │     │     数据来源：DB 中当前可用的 OpenCode providers
        │     └── ModelID 字段  → Enter → Overlay::OmoModelPicker
        │           数据来源：选中 provider 的 opencode.json 中 models map
        │           读取方式：opencode_config::get_typed_providers() → provider.models.keys()
        ├── Ctrl+J → 在 editor 内触发 live sync（不影响当前 draft）
        └── Save Template（底部）
```

### 4.5 数据流盒图

```text
┌─────────────────────────────────────────────────────────────────┐
│                    OMO Template 编辑流                            │
│                                                                 │
│  进入 OmoAgentConfig（从左侧导航）                                │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────┐                        │
│  │ OMO Landing 加载                    │                        │
│  │ 1. DB: 读取 omo_templates 列表      │                        │
│  │ 2. DB: 读取当前 active template     │                        │
│  │ 3. 检测 oh-my-openagent.json 存在性  │                        │
│  │ 4. 检测 settings.json 旧 bindings   │                        │
│  └─────────────────────────────────────┘                        │
│       │                                                         │
│       ├── 检测到旧 bindings 且未迁移                              │
│       │   → 显示 Migration Notice                                │
│       │   → Enter → 导入到 DB template                           │
│       │                                                         │
│       ├── Enter on template → 进入 editor                        │
│       ├── n → 新建 template → 进入 editor                        │
│       │   （agent list 预填充 catalog 中所有 agent，binding 为空）│
│       ├── s → 切换 active → sync JSON → 刷新列表                 │
│       ├── c → 复制 template → 进入 editor                        │
│       ├── r / Ctrl+J → 重读 DB + sync JSON + 刷新 UI             │
│       └── Del → 确认 → 删除 → 重选 active                       │
│                                                                 │
│  ┌─────────────────────────────────────┐                        │
│  │ OMO Template Editor 加载            │                        │
│  │ 1. 从 DB 读取 template bindings     │                        │
│  │ 2. 加载内置 agent catalog           │                        │
│  │ 3. 构建 OmoTemplateDraft            │                        │
│  │ 4. 检测 stale/orphan 状态           │                        │
│  └─────────────────────────────────────┘                        │
│       │                                                         │
│       ├── 焦点在 TemplateName → Enter 编辑 → Enter 提交          │
│       │   编辑行为：TemplateName 有独立的编辑 buffer              │
│       │   Enter 进入编辑态 → 输入 → Enter 提交到 draft           │
│       │   提交后 draft.template_name 更新 → dirty 检测生效       │
│       ├── 焦点在 Agent → Enter → 切到右侧 Details                │
│       ├── 焦点在 Provider → Enter → Provider Picker             │
│       │   → 选择后清空 model_id → dirty                          │
│       ├── 焦点在 ModelID → Enter → Model Picker                 │
│       │   → 选择后写入 model_id → dirty                          │
│       └── 焦点在 Save Template → Enter → 保存                    │
│                │                                                │
│                ▼                                                │
│  ┌─────────────────────────────────────┐                        │
│  │ 保存流程                            │                        │
│  │ 1. 验证 template name 不冲突        │                        │
│  │ 2. 写 DB: omo_templates + bindings  │                        │
│  │ 3. 若 active: sync oh-my-openagent  │                        │
│  │ 4. 成功 → 重读 DB 刷新 draft        │                        │
│  │    DB ok + sync fail → 降级提示      │                        │
│  │    DB fail → 保留 draft，显示错误    │                        │
│  └─────────────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘

Landing r / Ctrl+J 合并语义（以 PRD 为准，小写快捷键）：
├── 从 DB 重新读取 omo_templates 列表和 active template
├── 触发 active template → oh-my-openagent.json sync
├── 重新检测 live JSON 状态
├── 刷新 UI（template 列表 + active marker + live target 状态）
└── 不改变当前选中的 template 焦点位置

Editor 内 Ctrl+J 语义：
├── 不影响当前 draft（draft 继续保持编辑态）
├── 从 DB 读取当前 active template 的已提交 bindings
├── 触发 sync 到 oh-my-openagent.json
├── 显示 sync 结果 toast（成功 / 失败）
└── 若当前编辑的 template 就是 active template，sync 的是 DB 已提交版本，
    不是 draft 当前编辑态（避免未保存修改被意外写入 live）

新建 template 时的 agent list 预填充行为（已定案：选项 A）：
├── agent list 展示 catalog 中的所有 agent（coder, reviewer, planner, architect, ...）
├── 所有 agent 的 binding 状态为 <unbound>（provider_id 和 model_id 为空）
├── 用户可以直接在列表中选中 agent 开始编辑 binding
└── 保存时未绑定的 agent 仍然保留为 binding 记录（provider_id/model_id 为空字符串）

Template Name 编辑行为：
├── TemplateName 有独立的编辑 buffer（类似 TextInput）
├── Enter 进入编辑态 → 用户输入 → Enter 提交
├── 提交时写入 draft.template_name
├── Esc 退出编辑态，保留未提交的 buffer（不丢失输入）
└── dirty 检测：draft.template_name 与 original_snapshot.0 比较

Stale 检测的 draft 语义：
├── stale 检测基于 draft（包括未保存的修改），不是 DB
├── 用户修改 provider 但未保存 → stale 状态立即重算
├── 用户切换 agent 时不需要重算（agent 间切换不改变 draft 内容）
├── 只有以下事件触发重算：provider 数据变更 / save / reload / rebind / clear
└── 未解决的 stale → template 视为 dirty，不允许静默消失
```

```text
内置 OMO Agent Catalog（硬编码或配置文件）
├── coder
├── reviewer
├── planner
├── architect
└── ...（可扩展）

加载时机：
├── OMO Landing 加载时
├── Template Editor 初始化时
└── Stale/Orphan 检测时

用途：
├── 定义 editor 中的规范性 agent 列表
├── 检测 orphan agent key（live JSON 中有但 catalog 中无）
└── 迁移导入时的默认 binding 集合

存储位置选项：
├── 选项 A：硬编码在 Rust 代码中（当前 main 的做法）
├── 选项 B：仓库内 JSON 配置文件
└── 选项 C：从 oh-my-openagent.json 的 agents key 推导
推荐：选项 A（与当前 main 一致，简单可靠，后续可升级为 B）
```

### 4.7 Stale / Orphan 检测流程

```text
检测触发时机：
├── 进入 OMO Landing 时
├── 进入 Template Editor 时
├── Provider 数据变更后
├── Save / Reload / Sync 后
└── 显式 rebind / clear / discard 后

检测流程：

For each agent_key in template bindings:
    │
    ▼
[agent_key 在内置 catalog 中?]
    │
    ├── No → OrphanAgentKey
    │
    └── Yes → [provider_id 存在于当前 providers?]
              │
              ├── No → MissingProvider
              │
              └── Yes → [model_id 可用于该 provider?]
                        │
                        ├── No → MissingModel
                        │
                        └── Yes → Bound

关闭规则：
├── OrphanAgentKey：用户显式丢弃，或 catalog 更新后键被纳入
├── MissingProvider：用户 rebind 到合法 provider，或 clear binding
├── MissingModel：用户 rebind 到合法 model，或 clear binding
└── 任何 unresolved stale → template 视为 dirty，不允许静默消失
```

### 4.8 Live Sync 流程

```text
sync_omo_template_to_live(template_id):

[读取 DB active template 的 bindings]
    │
    ▼
[读取 oh-my-openagent.json（不存在则创建骨架）]
    │
    ▼
[遍历 bindings]
    │
    ├── agents.<agent_key>.model = "<provider_id>/<model_id>"
    │
    └── catalog 中有但 bindings 中无的 key
        → 保留 JSON 中已有的值（不覆盖）
    │
    ▼
[写入 oh-my-openagent.json]
    │
    ├── 成功 → 更新 DB last_synced_at
    │
    └── 失败 → 更新 DB last_sync_error
              → 显示 "DB committed, JSON sync failed"
```

### 4.9 迁移流程

```text
首次进入 OMO Landing 的迁移检测：

[DB 中有 omo_templates 记录?]
    │
    ├── Yes → 跳过迁移，直接展示
    │
    └── No → [settings.json 中有 omoAgentBindings?]
              │
              ├── Yes → 显示 Migration Notice
              │         │
              │         ├── Enter → 执行导入
              │         │   1. 创建 template "Imported Legacy"
              │         │   2. source = 'legacy_settings_import'
              │         │   3. 复制所有 bindings
              │         │   4. 标记迁移完成
              │         │   5. 进入 editor 让用户确认
              │         │
              │         └── Esc → 跳过，进入空 landing
              │
              └── No → [oh-my-openagent.json 存在且有 agents?]
                        │
                        ├── Yes → 可选：提示从 JSON 发现 agent keys
                        │         但不自动导入（JSON 不是编辑态真值）
                        │
                        └── No → 进入空 landing，引导用户新建 template

OMO Landing 空态 UI：

+--------------------------------------------------------------+
| OMO Agent Config                                             |
|--------------------------------------------------------------|
| No templates found                                           |
|                                                              |
| Press [n] to create a new template                           |
|--------------------------------------------------------------|
| [n] add   [r] refresh   [Esc] Back                           |
+--------------------------------------------------------------+

空态行为：
├── Enter → 无效果（没有可选 template）
├── n → 新建 template → 进入 editor（agent list 预填充 catalog）
├── r / Ctrl+J → 重读 DB + sync（仍然触发，刷新后可能发现迁移数据）
└── Esc → 返回左导航
```

---

## 5. 共用基础设施改动

### 5.1 PRD 一致性修正记录

以下修正以 PRD 为准：

1. **快捷键大小写**：OMO Landing 快捷键统一使用 PRD 定义的小写形式（`n`/`s`/`c`/`r`/`Del`），架构中不再使用大写变体
2. **OMO Model Picker 数据来源**：从当前 provider 的 `opencode.json` 中 `models` map 读取已有 model 列表（选项 A），不做远端 fetch
3. **ModelConfigDetail Ctrl+S 保存范围**：与 ProviderForm 的 Ctrl+S 语义相同，提交整个 provider snapshot（base fields + models），不是只保存当前 model
4. **Reload 与 Ctrl+J 合并**：OMO Landing 的 `r` (Reload) 和 `Ctrl+J` (Sync) 合并为单一操作——从 DB 重读 + 触发 JSON sync + 刷新 UI 状态

### 5.2 Route 枚举扩展

```text
Route 枚举新增：
├── OmoAgentConfig              ← OMO Landing
└── OmoTemplateEditor           ← OMO 全页 Editor

Route 枚举修改：
└── ProviderDetail { id }       ← 内部新增 ModelConfig 入口行

Route 枚举新增（OpenCode）：
├── OpenCodeModelConfigList { provider_id }
└── OpenCodeModelConfigDetail { provider_id, model_id }
```

### 5.3 NavItem 扩展

```text
NavItem 枚举新增：
└── OmoAgentConfig  ← 仅 OpenCode app_type 可见

导航顺序（OpenCode）：
Main → Providers → Mcp → Skills → Prompts → OmoAgentConfig → Config → Settings → Exit
```

### 5.4 Overlay 枚举扩展

```text
Overlay 枚举新增：
├── OmoProviderPicker { ... }   ← OMO editor 中选择 provider
├── OmoModelPicker { ... }      ← OMO editor 中选择 model
└── OmoMigrationNotice          ← 旧 bindings 导入提示

Overlay 枚举修改：
└── ModelFetchPicker            ← 结果回调需支持写入 draft
```

### 5.5 App 结构体扩展（已定案：方案 A）

```text
App 结构体新增两个 draft 字段，各自独立管理编辑会话：

pub struct App {
    // ... existing fields
    pub opencode_draft: Option<OpenCodeProviderDraft>,   // Provider 编辑会话
    pub omo_draft: Option<OmoTemplateDraft>,             // OMO 编辑会话
}

生命周期：
├── opencode_draft
│   ├── 进入 ProviderDetail 时：从 DB 初始化
│   ├── ModelConfigList / ModelConfigDetail：直接读 app.opencode_draft
│   ├── 保存成功后：从 DB 重读刷新 draft
│   ├── 离开整个 provider 编辑会话（回 Providers 列表）时：销毁
│   └── 切换不同 provider 时：销毁旧 draft，创建新 draft
│
└── omo_draft
    ├── 进入 OmoTemplateEditor 时：从 DB template 初始化
    ├── Editor 内所有操作：直接读写 app.omo_draft
    ├── 保存成功后：从 DB 重读刷新 draft
    └── 离开 editor（回 OMO Landing）时：销毁

两个 draft 互不干扰，各自有独立的 dirty 检测和 leave confirm。
```

### 5.6 Picker 结果回调抽象

```text
当前 picker 结果写入方式：
match overlay {
    ModelFetchPicker => form.field_xxx.set(selected_value)
}

目标：引入 PickerTarget 枚举

enum PickerTarget {
    FormField(ProviderAddField),           ← 现有行为
    OpenCodeDraftModelId,                  ← 写入 OpenCodeProviderDraft
    OmoDraftProvider { agent_key },        ← 写入 OmoTemplateDraft provider
    OmoDraftModelId { agent_key },         ← 写入 OmoTemplateDraft model_id
}

picker 选择确认后：
match picker_target {
    FormField(field) => form.set_field(field, value),
    OpenCodeDraftModelId => draft.set_current_model_id(value),
    OmoDraftProvider { key } => draft.set_provider(key, value),
    OmoDraftModelId { key } => draft.set_model_id(key, value),
}
```

---

## 6. 文件变更清单

### 6.1 OpenCode Provider 改动

```text
修改文件：
├── src/provider.rs                     +OpenCodeModelEntry, 修改 OpenCodeModel
├── src/opencode_config.rs              +get_current_model(), +set_current_model()
├── src/cli/tui/form.rs                 +OpenCodeModelConfig variant
├── src/cli/tui/form/provider_state.rs  修改 OpenCode field list
├── src/cli/tui/form/provider_json.rs   修改序列化逻辑
├── src/cli/tui/form/provider_state_loading.rs 修改加载逻辑
├── src/cli/tui/route.rs                +OpenCodeModelConfigList, +OpenCodeModelConfigDetail
├── src/cli/tui/app/types.rs            +OpenCodeModelDraft, +OpenCodeProviderDraft
├── src/cli/tui/app.rs                  +opencode_draft field
├── src/cli/tui/ui.rs                   修改 render_content dispatch
├── src/cli/tui/ui/forms/provider.rs    修改 OpenCode 表单渲染
├── src/cli/tui/runtime_actions/providers.rs 修改按键处理
├── src/cli/tui/app/overlay_handlers/pickers.rs 修改 picker 结果回调
├── src/services/provider/mod.rs        修改保存逻辑
├── src/cli/i18n/texts/provider_editor.rs +ModelConfig 文案

新增文件：
├── src/cli/tui/ui/opencode_model_list.rs      ModelConfigList 页面
├── src/cli/tui/ui/opencode_model_detail.rs    ModelConfigDetail 页面
```

### 6.2 OMO Agent Config 改动

```text
修改文件：
├── src/database/schema.rs              +omo_templates, +omo_template_bindings
├── src/database/dao/mod.rs             +pub mod omo
├── src/settings.rs                     +OmoAgentBinding, +oh-my-openagent 读写
├── src/cli/tui/route.rs                +OmoAgentConfig, +OmoTemplateEditor
├── src/cli/tui/app/menu.rs             +OmoAgentConfig nav item
├── src/cli/tui/app/types.rs            +OmoTemplateDraft, +OmoStaleState
├── src/cli/tui/app.rs                  +omo_draft field
├── src/cli/tui/ui.rs                   +render_omo dispatch
├── src/cli/tui/data.rs                 +OmoTemplateRow in ConfigSnapshot
├── src/cli/tui/app/helpers.rs          +build_omo_* 辅助函数
├── src/services/config.rs              +sync_omo_template_to_live()

新增文件：
├── src/database/dao/omo.rs                    OMO DAO
├── src/cli/tui/ui/omo_landing.rs              OMO Landing 页面
├── src/cli/tui/ui/omo_editor.rs               OMO Template Editor 页面
├── src/cli/tui/runtime_actions/omo.rs         OMO 按键处理
├── src/cli/i18n/texts/omo.rs                  OMO 文案
```

---

## 7. 优化建议

### 7.1 架构层面

1. **消除 OMO 逻辑分散**：当前 main 分支的 OMO 逻辑散落在 settings.rs / helpers.rs / overlay_handlers / runtime_actions / editor.rs 共 6 个文件中。建议收敛到独立模块：
   - `cli/tui/ui/omo_landing.rs` — Landing 渲染
   - `cli/tui/ui/omo_editor.rs` — Editor 渲染
   - `cli/tui/runtime_actions/omo.rs` — 按键处理
   - `database/dao/omo.rs` — 持久化

2. **OpenCode draft 统一**：当前 OpenCode 没有 explicit draft 机制，靠 `initial_snapshot` 做脏检测。新增 `OpenCodeProviderDraft` 后，Provider Form / ModelConfigList / ModelConfigDetail 三个页面共享同一份 draft，避免真值分裂。

3. **Picker 回调解耦**：当前 picker 结果直接耦合到 `TextInput`，引入 `PickerTarget` 枚举后，OMO 和 OpenCode 共用同一个 picker 基础设施，只是回调目标不同。

### 7.2 存储层面

1. **消除 settings.json 双写**：OMO 的 settings.json 镜像只在迁移期有用。建议在迁移完成 + 首次 sync 成功后，停止向 settings.json 写入 OMO bindings。

2. **opencode.json 的 currentModel 字段**：当前 `opencode_config.rs` 没有 `currentModel` 的读写方法。需要新增，但只在 provider 级别操作（不干扰其他 provider 的 currentModel）。

3. **DB migration 顺序**：OMO 表的 migration 需要在 OpenCode provider 存储结构变更之前执行，因为 OMO 的 provider_id 外键语义依赖 providers 数据。

### 7.3 TUI 层面

1. **Editor 焦点状态机统一**：OMO Editor 和 OpenCode ModelConfigDetail 都有"左侧列表 → 右侧详情 → 底部输入"的三区布局。可以抽象一个 `ThreePaneEditorFocus` 枚举复用焦点切换逻辑。

2. **Leave confirm 统一**：OMO Editor 和 OpenCode Provider Form 都需要 dirty-leave confirm。建议统一到 `ConfirmAction` 枚举中，复用同一个 confirm overlay 渲染。

3. **Toast 反馈统一**：保存成功 / 失败 / 降级三种状态的 toast 样式和文案模式统一。

---

## 8. 测试分层

```text
单元测试（src-tauri/src/** 内 #[cfg(test)]）：
├── OpenCodeProviderDraft：脏检测、model CRUD、rename、currentModel 回退
├── OmoTemplateDraft：脏检测、binding CRUD、stale 检测
├── opencode_config：currentModel 读写、multi-model 序列化
└── settings.rs：OmoAgentBinding 序列化、oh-my-openagent sync

集成测试（src-tauri/tests/）：
├── OMO：legacy bindings → DB template → oh-my-openagent.json 完整链路
├── OMO：stale/orphan 检测 + closure 规则
├── OMO：DB committed + JSON sync failed 降级
├── OpenCode：provider save → opencode.json 多 model + currentModel
└── OpenCode：modelId rename 原子性 + currentModel 修复

TUI 交互测试（cli/tui/tests.rs）：
├── OMO：Landing 进入 / 返回 / template 切换
├── OMO：Editor 焦点链 TemplateName → AgentsList → Details → Save
├── OMO：Provider/Moid picker 打开 / 选择 / 取消
├── OpenCode：ProviderForm → ModelConfigList → ModelConfigDetail 导航
├── OpenCode：Model ID picker fetch → select → write-back
└── 通用：dirty leave confirm

手工验收（cargo run interactive）：
├── 完整 OMO 新建 template → 编辑 → 保存 → sync 流程
├── 完整 OpenCode 新建 provider → 添加多 model → 保存 → opencode.json 验证
├── 迁移：旧 bindings → 导入 → 确认 → 编辑
└── 异常：fetch 失败 / sync 失败 / stale binding 修复
```

---

## 9. 依赖顺序

```text
阶段 1：DB 基础（无 UI 依赖）
├── OMO 表 migration
├── OMO DAO
└── OpenCode currentModel 读写

阶段 2：数据层（依赖阶段 1）
├── OmoTemplateDraft + CRUD
├── OpenCodeProviderDraft + multi-model
└── oh-my-openagent.json sync 函数

阶段 3：OpenCode Provider UI（依赖阶段 2）
├── ModelConfigList 页面
├── ModelConfigDetail 页面
├── Model ID picker 嵌入
└── 保存链路

阶段 4a：OMO Landing + 迁移（依赖阶段 1）
├── OMO Landing 页面（template 列表 + template 级动作）
├── 迁移检测 + Migration Notice
├── r / Ctrl+J 合并 reload+sync
└── 旧 bindings 导入流程

阶段 4b：OMO Template Editor（依赖阶段 2 + 4a）
├── 全页 Template Editor
├── Provider/Model picker 嵌入
├── Stale/Orphan 检测
├── Ctrl+J editor 内 sync
└── 保存链路（DB + JSON sync）

阶段 5：收尾（依赖阶段 3 + 4a + 4b）
├── settings.json 双写收敛
├── 回归测试
└── 手工验收
```

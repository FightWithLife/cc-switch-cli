# Phase 1：OpenCode Provider 改造

> 架构设计：`../code-architecture/20260429_code_architecture_design.md`
>
> 产品 PRD：`../product-prd/20260429_omo_opencode_product_prd.md`
>
> 范围：OpenCode Provider 从单 model 编辑升级为 `ProviderForm → ModelConfigList → ModelConfigDetail` 三层页面

## 阶段总览

```text
Phase 1 拆分为 6 个阶段，按依赖顺序执行：

1.1 数据结构    ← 无依赖，可立即开始
1.2 Draft 状态  ← 依赖 1.1
1.3 路由与导航  ← 依赖 1.1
1.4 ModelConfigList 页面 ← 依赖 1.2 + 1.3
1.5 ModelConfigDetail 页面 ← 依赖 1.4
1.6 保存链路与收尾 ← 依赖 1.5
```

---

## 1.1 数据结构

**目标**：定义 OpenCode multi-model 所需的 Rust 结构体，不涉及 TUI。

**改动文件**：

```text
src/provider.rs                     +OpenCodeModelDraft 结构体
src/cli/tui/app/types.rs            +OpenCodeProviderDraft 结构体
src/cli/tui/form.rs                 +ProviderAddField::OpenCodeModelConfig variant
src/opencode_config.rs              +get_current_model() / set_current_model()
```

**具体任务**：

- [ ] 在 `provider.rs` 中新增 `OpenCodeModelDraft` 结构体：
  ```rust
  pub struct OpenCodeModelDraft {
      pub model_id: String,
      pub model_name: String,
      pub input_limit: Option<u64>,
      pub output_limit: Option<u64>,
      pub original_model_id: Option<String>,
  }
  ```

- [ ] 在 `cli/tui/app/types.rs` 中新增 `OpenCodeProviderDraft` 结构体：
  ```rust
  pub struct OpenCodeProviderDraft {
      pub provider_id: String,
      pub name: String,
      pub base_url: String,
      pub api_key: String,
      pub npm: String,
      pub models_by_id: IndexMap<String, OpenCodeModelDraft>,
      pub current_model: String,
      pub dirty: bool,
      pub last_loaded_from_db: Value,
  }
  ```
  包含方法：`is_dirty()` / `set_dirty()` / `model_count()` / `has_model(id)` / `add_model()` / `remove_model()` / `rename_model()`

- [ ] 在 `form.rs` 的 `ProviderAddField` 枚举中新增 `OpenCodeModelConfig` variant

- [ ] 在 `opencode_config.rs` 中新增 `get_current_model(provider_id)` 和 `set_current_model(provider_id, model_id)` 函数

- [ ] 为 `OpenCodeProviderDraft` 和 `OpenCodeModelDraft` 编写单元测试：
  - 脏检测（修改任一字段 → dirty = true）
  - model CRUD（add / remove / rename）
  - rename 原子性（old key 消失，new key 出现，currentModel 修复）
  - duplicate model ID 检测

**完成标志**：

- `cargo build` 通过
- `cargo test` 通过（含新增单元测试）
- `OpenCodeProviderDraft` 可以独立构造、操作、检测 dirty，不依赖 TUI

---

## 1.2 Draft 初始化与序列化

**目标**：实现 `OpenCodeProviderDraft` 与 `Provider.settings_config` 的双向转换。

**改动文件**：

```text
src/cli/tui/form/provider_state_loading.rs  修改  populate_opencode_form → 初始化 draft
src/cli/tui/form/provider_json.rs           修改  序列化支持 multi-model + currentModel
src/cli/tui/form/provider_state.rs          修改  OpenCode 字段列表改为 [Name, BaseURL, APIKey, NPM, ModelConfig]
```

**具体任务**：

- [ ] 重写 `populate_opencode_form()`：从 `Provider.settings_config` 解析 `models` HashMap 和 `currentModel`，初始化 `OpenCodeProviderDraft`

- [ ] 重写 `to_provider_json_value()` 的 OpenCode 分支：从 `OpenCodeProviderDraft` 序列化为 `Provider.settings_config` JSON，包含：
  - `npm` 字段
  - `options.baseURL` / `options.apiKey`
  - `models` HashMap（遍历 `models_by_id`）
  - `currentModel` 字段

- [ ] 修改 `provider_state.rs` 的 OpenCode 字段列表：
  ```rust
  AppType::OpenCode => {
      fields.push(ProviderAddField::Name);
      fields.push(ProviderAddField::OpenCodeBaseUrl);
      fields.push(ProviderAddField::OpenCodeApiKey);
      fields.push(ProviderAddField::OpenCodeNpmPackage);
      fields.push(ProviderAddField::OpenCodeModelConfig);  // 替代原来的 4 个 model 字段
  }
  ```

- [ ] 编写单元测试：
  - 从单 model JSON → draft 初始化（向后兼容）
  - 从多 model JSON → draft 初始化
  - draft → JSON 序列化 → 反序列化 roundtrip
  - currentModel 字段正确读写
  - 空 models map 处理

**完成标志**：

- `cargo test` 通过
- 可以手动构造一个 `Provider`，通过 `populate_opencode_form` 得到 draft，再通过 `to_provider_json_value` 序列化回去，数据不丢失

---

## 1.3 路由与导航

**目标**：新增 `OpenCodeModelConfigList` 和 `OpenCodeModelConfigDetail` 路由，ProviderForm 中 ModelConfig 入口行可导航。

**改动文件**：

```text
src/cli/tui/route.rs                +OpenCodeModelConfigList { provider_id }
                                      +OpenCodeModelConfigDetail { provider_id, model_id }
src/cli/tui/app.rs                  +opencode_draft: Option<OpenCodeProviderDraft>
src/cli/tui/ui.rs                   修改 render_content dispatch
src/cli/tui/ui/forms/provider.rs    修改 OpenCode 表单渲染：ModelConfig 入口行
src/cli/tui/runtime_actions/providers.rs  修改 按键处理：Enter on ModelConfig → 导航
```

**具体任务**：

- [ ] 在 `route.rs` 中新增两个路由 variant

- [ ] 在 `app.rs` 的 `App` 结构体中新增 `opencode_draft: Option<OpenCodeProviderDraft>` 字段

- [ ] 在 `ui.rs` 的 `render_content()` 中新增两个路由的 dispatch 分支（先渲染占位页面）

- [ ] 修改 `ui/forms/provider.rs`：当 `app_type == OpenCode` 时，渲染 ModelConfig 入口行（替代原来的 4 个 model 字段），展示 model count

- [ ] 修改 `runtime_actions/providers.rs`：
  - 进入 `ProviderDetail` 时初始化 `app.opencode_draft`
  - `Enter` on `OpenCodeModelConfig` → 导航到 `Route::OpenCodeModelConfigList`
  - `Esc` on `ProviderDetail` 时销毁 draft

- [ ] 验证导航链路：`Providers 列表 → ProviderDetail → Enter on ModelConfig → ModelConfigList → Esc → ProviderDetail → Esc → Providers 列表`

**完成标志**：

- `cargo run` 进入 interactive mode
- OpenCode Provider Detail 页面中可以看到 ModelConfig 入口行
- Enter 进入 ModelConfigList（占位页面），Esc 返回
- 返回 Providers 列表时 draft 被销毁

---

## 1.4 ModelConfigList 页面

**目标**：实现完整的 ModelConfigList 页面，展示 model 列表，支持新建、编辑、删除。

**改动文件**：

```text
src/cli/tui/ui/opencode_model_list.rs       新增 ModelConfigList 页面渲染
src/cli/tui/runtime_actions/providers.rs    修改 ModelConfigList 按键处理
src/cli/tui/app/overlay_handlers/pickers.rs 修改 删除确认对话框
```

**具体任务**：

- [ ] 新增 `opencode_model_list.rs`：
  - 渲染 model 列表（以 model name 为主展示，未配置 name 时展示 model id 大写）
  - 高亮当前选中项
  - 底部显示快捷键提示：`[Enter] edit model   [n] new model   [Del] delete   [Esc] back`

- [ ] 在 `runtime_actions/providers.rs` 中实现 ModelConfigList 按键：
  - `↑/↓`：导航列表
  - `Enter`：进入选中 model 的 ModelConfigDetail
  - `n`：新建 model → 导航到 ModelConfigDetail（空状态）
  - `Del`：弹出删除确认
  - `Esc`：返回 ProviderForm（dirty 时先 confirm）

- [ ] 删除确认逻辑：
  - 确认后从 `draft.models_by_id` 中移除
  - 触发 currentModel 回退规则（§3.6）
  - `draft.dirty = true`

- [ ] 空列表状态：列表为空时显示 "No models configured. Press [n] to add one."

**完成标志**：

- `cargo run` 进入 interactive mode
- ModelConfigList 正确展示已有 model（以 model name）
- `n` 可以进入空白 detail，`Del` 可以删除并触发 currentModel 回退
- `Esc` 返回 ProviderForm

---

## 1.5 ModelConfigDetail 页面

**目标**：实现完整的 ModelConfigDetail 页面，包含字段编辑、Model ID fetch/picker、校验。

**改动文件**：

```text
src/cli/tui/ui/opencode_model_detail.rs    新增 ModelConfigDetail 页面渲染
src/cli/tui/runtime_actions/providers.rs   修改 ModelConfigDetail 按键处理
src/cli/tui/app/overlay_handlers/pickers.rs 修改 picker 结果写入 draft
src/cli/tui/runtime_systems/types.rs       修改 fetch 结果回调支持 draft
```

**具体任务**：

- [ ] 新增 `opencode_model_detail.rs`：
  - 渲染 4 个字段：Model Name / Model ID / Input Limit / Output Limit
  - 底部输入区：当前字段编辑态 / picker 状态 / 错误信息
  - 快捷键提示：`[Enter] edit/open picker   [Ctrl+S] save   [Esc] back`

- [ ] 在 `runtime_actions/providers.rs` 中实现 ModelConfigDetail 按键：
  - `↑/↓`：在 4 个字段间切换焦点
  - `Enter` on Model Name / Input Limit / Output Limit：底部输入区进入编辑态
  - `Enter` on Model ID：
    - 前置检查 draft.base_url 非空
    - 发起 fetch → 显示 loading → 弹出 ModelFetchPicker
  - `Ctrl+S`：触发保存（保存逻辑在 §1.6）
  - `Esc`：返回 ModelConfigList（dirty 时先 confirm）

- [ ] 修改 `overlay_handlers/pickers.rs`：
  - ModelFetchPicker 结果写入 `draft.models_by_id[current].model_id`
  - 从 fetch 结果中过滤掉 draft 中已存在的 model ID
  - 已配置的 model ID 标记为 `[configured]`

- [ ] 字段校验：
  - Input Limit / Output Limit：空值允许，非空时必须为正整数（`1..=u32::MAX`）
  - Model ID：保存时非空校验 + 重复校验

- [ ] 新建 vs 编辑区分：
  - 进入 detail 时根据 `model_id` 是否在 `draft.models_by_id` 中判断
  - 编辑已有：`original_model_id` 初始化为当前 `model_id`
  - 新建：`original_model_id` = None，所有字段为空

**完成标志**：

- `cargo run` 进入 interactive mode
- ModelConfigDetail 正确展示 4 个字段
- Model ID 按 Enter 可以触发 fetch + picker
- picker 结果正确写入 draft
- 底部输入区可以编辑 Model Name / Input Limit / Output Limit
- 校验错误正确展示
- Esc 返回 ModelConfigList

---

## 1.6 保存链路与收尾

**目标**：实现完整的保存流程（draft → DB → opencode.json），处理 rename 原子性、currentModel 修复、错误降级。

**改动文件**：

```text
src/services/provider/mod.rs              修改 保存逻辑支持 multi-model + currentModel
src/cli/tui/runtime_actions/providers.rs  修改 Ctrl+S 保存处理
src/cli/tui/form/provider_json.rs         修改 draft → JSON 序列化
src/cli/i18n/texts/provider_editor.rs     +ModelConfig 相关文案
```

**具体任务**：

- [ ] 在 `runtime_actions/providers.rs` 中实现保存流程：
  1. 从 `app.opencode_draft` 序列化为 `Provider.settings_config`
  2. 调用 `ProviderService::update()`
  3. 成功 → 从 DB 重读刷新 draft，显示 toast "Saved"
  4. DB 失败 → 保留 draft，显示 toast "Save failed: <reason>"
  5. DB 成功但 JSON sync 失败 → 显示 toast "DB committed, JSON sync failed"

- [ ] rename 原子性：在 draft → JSON 序列化时处理：
  1. 若 `original_model_id` 有值且 != `model_id`：
     - 从 `models` map 移除 `original_model_id` key
     - 插入 `model_id` key
  2. 若 `original_model_id == current_model`：`current_model = model_id`
  3. 保存成功后：`original_model_id = Some(model_id)`

- [ ] currentModel 管理（删除 model 后的回退）：
  - 在 draft 层面实现（`remove_model()` 方法中处理）
  - 保存时写入 JSON

- [ ] 新建 model 校验：
  - `model_id` 为空 → 底部显示 "Model ID is required"，阻止保存
  - `model_id` 重复 → 底部显示 "Duplicated model id `xxx`"，阻止保存
  - Input Limit / Output Limit 非法 → 底部显示 "Must be positive integer"

- [ ] ProviderForm 的 Ctrl+S 也要能保存：
  - 与 ModelConfigDetail 的 Ctrl+S 语义相同（保存整个 provider snapshot）
  - 保存后 draft 刷新

- [ ] 在 `i18n/texts/provider_editor.rs` 中新增所有 ModelConfig 相关文案

- [ ] 回归验证：
  - 现有的 Claude / Codex / Gemini provider 编辑不受影响
  - OpenCode 单 model provider（旧数据）可以正确加载为 draft 并编辑
  - 新建 OpenCode provider → 添加多 model → 保存 → 重启 → 数据不丢失

**完成标志**：

- `cargo fmt --check` 通过
- `cargo test` 通过
- `cargo clippy` 无新增 warning
- 手工验收完整链路：
  1. 新建 OpenCode provider → 填写 Name / Base URL / API Key / NPM
  2. Enter on ModelConfig → ModelConfigList（空）→ n → ModelConfigDetail
  3. 填写 Model Name → Enter on Model ID → fetch → picker → 选择 → 回写
  4. 填写 Input Limit / Output Limit → Ctrl+S → 保存成功
  5. 返回 ModelConfigList → 看到新 model
  6. 再添加一个 model → 保存 → 验证 opencode.json 中有两个 model + currentModel
  7. 删除 currentModel → 验证 currentModel 自动回退
  8. 修改 model ID → 保存 → 验证旧 key 消失、新 key 存在、currentModel 正确

---

## Phase 1 完成标准

```text
功能验收：
├── ProviderForm 中有 Model Config 入口行
├── ModelConfigList 以 model name 展示，支持新建/编辑/删除
├── ModelConfigDetail 有 Model Name / Model ID / Input Limit / Output Limit
├── Model ID 通过 fetch/picker 选择（非自由文本输入主路径）
├── fetch loading / empty / error / blocked 状态可区分
├── picker 结果写回纯 model id（不写回 provider/model）
├── Model Name 为空时保存用 model id 大写作为默认名
├── limit 为空表示未设置，非空必须是正整数
├── duplicate model id 阻止保存
├── 删除或改名 current model 后不会留下悬空 current model
├── dirty 离开必须确认
├── 保存成功 / 保存失败 / DB committed + JSON sync failed 三种状态可区分

回归验收：
├── Claude / Codex / Gemini provider 编辑不受影响
├── 旧单 model OpenCode provider 可以正确加载和编辑
├── cargo fmt / test / clippy 全部通过
```

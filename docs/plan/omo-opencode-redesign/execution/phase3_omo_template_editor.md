# Phase 3：OMO Template Editor + Stale 检测

> 架构设计：`../code-architecture/20260429_code_architecture_design.md`
>
> 产品 PRD：`../product-prd/20260429_omo_opencode_product_prd.md`
>
> 范围：全页 Template Editor、Provider/Model picker、Stale/Orphan 检测、保存链路
>
> 依赖：Phase 2 完成（Landing + DB + DAO + Draft + Sync 已就绪）

## 阶段总览

```text
Phase 3 拆分为 4 个阶段：

3.1 Editor 页面骨架    ← 依赖 Phase 2
3.2 Provider/Model Picker ← 依赖 3.1
3.3 Stale/Orphan 检测   ← 依赖 3.1
3.4 保存链路 + 收尾     ← 依赖 3.2 + 3.3
```

---

## 3.1 Editor 页面骨架

**目标**：实现 OMO Template Editor 的页面布局、焦点状态机、基础按键处理。不含 picker 和 stale 检测。

**改动文件**：

```text
src/cli/tui/route.rs               修改  +Route::OmoTemplateEditor
src/cli/tui/ui/omo_editor.rs       新增  Template Editor 页面渲染
src/cli/tui/runtime_actions/omo.rs 修改  Editor 按键处理
src/cli/tui/app/types.rs           修改  +OmoTemplateEditorFocus 枚举
src/cli/tui/app.rs                 修改  +omo_draft: Option<OmoTemplateDraft>
```

**具体任务**：

- [ ] 在 `route.rs` 中新增 `Route::OmoTemplateEditor`

- [ ] 在 `app.rs` 的 `App` 中新增 `omo_draft: Option<OmoTemplateDraft>`

- [ ] 在 `types.rs` 中新增焦点枚举：
  ```rust
  pub enum OmoTemplateEditorFocus {
      TemplateName,
      AgentsList,
      DetailsPane,        // Agent Details 区域
      SaveTemplate,       // 底部保存动作
  }
  ```

- [ ] 新增 `ui/omo_editor.rs`，渲染四区布局：
  ```
  ┌─────────────────────────────────────────────────────────────────┐
  │ Edit OMO Template: <template_name>                              │
  │-----------------------------------------------------------------│
  │ > Template Name: <name>                                         │
  │-----------------------------------------------------------------│
  │ Agents List                | Agent Details                      │
  │----------------------------|------------------------------------│
  │ > coder                    | Agent    : coder                   │
  │   reviewer                 | Provider : <provider>              │
  │   planner                  | Model ID : <model_id>              │
  │   architect                | Final    : <provider>/<model_id>   │
  │                            | Status   : <bound/unbound/stale>   │
  │-----------------------------------------------------------------│
  │ [Save Template]                                                 │
  │ [Enter] edit/select/save   [Up/Down] move focus   [Esc] back    │
  └─────────────────────────────────────────────────────────────────┘
  ```

- [ ] 焦点状态机实现：
  - 初始焦点：`TemplateName`
  - `↓` from TemplateName → AgentsList
  - `↑` from AgentsList first item → TemplateName
  - `Enter` on agent → DetailsPane（焦点切到右侧第一个字段）
  - `↓` from DetailsPane:Provider → DetailsPane:ModelID
  - `↓` from DetailsPane:ModelID → SaveTemplate
  - `↑` from SaveTemplate → DetailsPane:ModelID
  - `↑` from DetailsPane:Provider → AgentsList

- [ ] TemplateName 编辑态：
  - `Enter` 进入编辑态（底部输入区显示当前名称）
  - 输入 → `Enter` 提交到 `draft.template_name`
  - `Esc` 退出编辑态，保留 buffer
  - 提交后 `dirty = true`（若名称变化）

- [ ] Agent Details 展示：
  - 选中 agent 时右侧显示当前 binding
  - `provider` / `model_id` 显示为可聚焦字段
  - `Final` 显示组装后的 `provider/model`
  - `Status` 显示 `bound` / `unbound` / `stale`

- [ ] Esc 返回行为：
  - 从 TemplateName 编辑态按 Esc → 退出编辑态，不离开 editor
  - 从 AgentsList / DetailsPane / SaveTemplate 按 Esc → 尝试返回 Landing
  - 若 dirty → leave confirm
  - 若 clean → 直接返回 Landing，销毁 draft

- [ ] `r`/`Ctrl+J` 在 Editor 内的行为：
  - 从 DB 读取当前 active template 的已提交 bindings
  - 触发 sync 到 oh-my-openagent.json
  - 显示 sync 结果 toast
  - 不影响当前 draft

**完成标志**：

- `cargo run` 进入 interactive mode
- Editor 正确渲染四区布局
- 焦点可以在四个区域间正确切换
- TemplateName 可以编辑
- Agent Details 正确展示 binding 信息
- Esc dirty leave confirm 正确工作

---

## 3.2 Provider / Model Picker

**目标**：在 Editor 内集成 Provider Picker 和 Model Picker，实现 binding 编辑。

**改动文件**：

```text
src/cli/tui/app/types.rs           修改  +Overlay::OmoProviderPicker + OmoModelPicker
src/cli/tui/ui/overlay/pickers.rs  修改  渲染 OMO picker
src/cli/tui/app/overlay_handlers/pickers.rs  修改  OMO picker 按键处理
src/cli/tui/runtime_actions/omo.rs 修改  Enter on Provider/ModelID → 打开 picker
src/cli/tui/app/helpers.rs         修改  build_omo_provider_options()
```

**具体任务**：

- [ ] 在 `types.rs` 中新增 overlay：
  ```rust
  Overlay::OmoProviderPicker {
      agent_key: String,
      search: String,
      options: Vec<OmoProviderOption>,
      selected: usize,
  }
  Overlay::OmoModelPicker {
      agent_key: String,
      provider_id: String,
      search: String,
      model_ids: Vec<String>,
      selected: usize,
  }
  ```

- [ ] 在 `helpers.rs` 中实现：
  - `build_omo_provider_options(db) -> Vec<OmoProviderOption>` — 从 DB 中读取所有 OpenCode providers
  - `build_omo_model_options(provider_id) -> Vec<String>` — 从 `opencode_config::get_typed_providers()` 读取该 provider 的 models keys

- [ ] Provider Picker 交互：
  - 焦点在 DetailsPane:Provider → `Enter` → 打开 picker
  - picker 展示可用 provider 列表，支持搜索
  - 选择后：`draft.set_binding(agent_key, OmoAgentBinding { provider_id, model_id: "" })`
  - 同时清空该 agent 的 model_id
  - `draft.dirty = true`
  - `Esc` 取消，回到 DetailsPane

- [ ] Model Picker 交互：
  - 焦点在 DetailsPane:ModelID → `Enter` → 检查 provider 是否已设置
  - 未设置 → 显示 blocked feedback "Select provider first"
  - 已设置 → 打开 picker，展示该 provider 的 model ID 列表（从 opencode.json 读取）
  - 选择后：`draft.set_binding(agent_key, OmoAgentBinding { provider_id, model_id })`
  - `draft.dirty = true`
  - `Esc` 取消，回到 DetailsPane

- [ ] Picker 渲染复用现有 `TuiPicker` 基础设施：
  - 搜索框 + 列表 + 高亮 + Enter 确认 + Esc 取消
  - 与 OpenCode ModelFetchPicker 共用同一套渲染逻辑

- [ ] Picker 结果回调通过 `PickerTarget` 枚举分派：
  ```rust
  PickerTarget::OmoDraftProvider { agent_key }
  PickerTarget::OmoDraftModelId { agent_key }
  ```

**完成标志**：

- `cargo run` 进入 interactive mode
- Enter on Provider 打开 provider picker → 选择 → binding 更新
- provider 变更后 model_id 被清空
- Enter on ModelID 打开 model picker → 选择 → binding 更新
- 未选 provider 时 ModelID 显示 blocked feedback
- dirty 状态正确追踪

---

## 3.3 Stale / Orphan 检测

**目标**：实现 per-agent 的 stale/orphan 状态检测和 UI 展示。

**改动文件**：

```text
src/cli/tui/app/types.rs           修改  +OmoBindingStatus + OmoStaleState
src/cli/tui/app/helpers.rs         修改  +detect_omo_stale_states()
src/cli/tui/ui/omo_stale.rs        新增  Stale 状态渲染
src/cli/tui/ui/omo_editor.rs       修改  Agent Details 中展示 stale 状态
src/cli/tui/runtime_actions/omo.rs 修改  stale agent 的特殊按键处理
```

**具体任务**：

- [ ] 在 `types.rs` 中新增：
  ```rust
  pub enum OmoBindingStatus {
      Bound,
      MissingProvider,
      MissingModel,
      OrphanAgentKey,
  }
  pub struct OmoStaleState {
      pub agent_key: String,
      pub binding: OmoAgentBinding,
      pub status: OmoBindingStatus,
      pub resolved: bool,
  }
  ```

- [ ] 在 `helpers.rs` 中实现 `detect_omo_stale_states()`：
  ```
  For each agent_key in draft.bindings:
      if agent_key not in catalog → OrphanAgentKey
      else if provider_id not in current providers → MissingProvider
      else if model_id not available for provider → MissingModel
      else → Bound
  ```

- [ ] 检测触发时机：
  - 进入 Editor 时
  - Provider 数据变更后
  - Save / Reload 后
  - 显式 rebind / clear 后
  - 切换 agent 时不重算

- [ ] 在 `ui/omo_editor.rs` 中展示 stale 状态：
  - Agent Details 的 Status 字段显示具体状态
  - stale agent 在左侧列表中标记（如 `⚠` 前缀）
  - stale agent 选中时右侧显示具体 warning 信息

- [ ] 新增 `ui/omo_stale.rs`，渲染 stale 状态详情：
  ```
  Agent       : reviewer
  Provider    : old-provider
  Model ID    : old-model
  Status      : missing provider
  [Enter] Rebind   [Ctrl+X] Clear   [Esc] Back
  ```

- [ ] Stale agent 的特殊按键：
  - `Enter` on stale agent → 可以正常进入 DetailsPane
  - DetailsPane 中 Enter on Provider → 打开 picker 进行 rebind
  - `Ctrl+X` on stale agent → 清空该 agent 的 binding
  - rebind 或 clear 后 → 重算 stale 状态

- [ ] 未解决的 stale → template 视为 dirty：
  - `is_dirty()` 除了检查 binding 变化外，还需检查是否有 unresolved stale
  - 保存时若有 unresolved stale → 允许保存（只是 warning），但显示提示

- [ ] stale 不会静默消失：
  - reload / re-enter / sync retry 不自动清除 stale
  - 只有显式 rebind / clear / discard 成功后才标记 `resolved = true`

**完成标志**：

- `cargo run` 进入 interactive mode
- stale agent 在列表中有视觉标记
- 选中 stale agent 时右侧显示具体 stale 信息
- rebind 后 stale 消失
- clear binding 后 stale 消失
- 未处理的 stale 不因 reload 消失

---

## 3.4 保存链路 + 收尾

**目标**：实现 Template Editor 的完整保存流程，集成所有组件。

**改动文件**：

```text
src/cli/tui/runtime_actions/omo.rs 修改  保存处理
src/cli/tui/app/overlay_handlers/dialogs.rs  修改  删除确认 + leave confirm
src/cli/i18n/texts/omo.rs          修改  Editor 相关文案
```

**具体任务**：

- [ ] 保存流程实现：
  1. 焦点在 SaveTemplate → `Enter` 触发
  2. 验证 template name 不为空
  3. 验证 template name 不与已有 template 冲突（排除自身）
  4. 若 `template_id` 为 None → 创建新 template（DB insert）
  5. 若 `template_id` 有值 → 更新已有 template
  6. 保存所有 bindings 到 DB
  7. 若当前 template 是 active → sync oh-my-openagent.json
  8. 成功 → 重读 DB 刷新 draft，toast "Saved"
  9. DB ok + sync fail → toast "DB committed, JSON sync failed"
  10. DB fail → 保留 draft，toast "Save failed: <reason>"

- [ ] 新建 template 保存后的特殊行为：
  - 保存成功后自动成为 active template
  - 立即触发 sync

- [ ] Leave confirm 实现：
  ```
  +--------------------------------------------------------------+
  | Unsaved changes detected                                     |
  |--------------------------------------------------------------|
  | Save before leaving this editor?                             |
  |                                                              |
  | [Enter] Save and leave   [n] Discard   [Esc] Stay            |
  +--------------------------------------------------------------+
  ```
  - Enter → 保存 → 返回 Landing
  - n → 丢弃修改 → 返回 Landing → 销毁 draft
  - Esc → 留在 Editor

- [ ] 保存后 draft 刷新：
  - 从 DB 重读 template + bindings
  - 重建 `original_snapshot`
  - `dirty = false`

- [ ] 销毁 draft 时机：
  - 返回 Landing 时（无论是否保存）
  - 切换到其他页面时
  - leave confirm 中选择 Discard 时

**完成标志**：

- `cargo fmt --check` 通过
- `cargo test` 通过
- `cargo clippy` 无新增 warning
- 手工验收完整链路：
  1. 从 Landing 进入 Editor → 编辑 template name → 添加 binding → Save → 成功
  2. 新建 template → 命名 → 添加 bindings → Save → 自动成为 active → sync
  3. 编辑 active template → 修改 provider → model_id 清空 → rebind → Save → sync
  4. Stale binding → rebind → stale 消失 → Save
  5. dirty 离开 → leave confirm → Esc 留下 / n 丢弃 / Enter 保存离开
  6. Ctrl+J 在 Editor 内触发 sync → toast 显示结果

---

## Phase 3 完成标准

```text
功能验收：
├── Template Editor 是新增和编辑 template 的唯一主路径
├── 初始焦点在 Template Name
├── 左侧 agent 列表，右侧 agent details
├── Provider 和 Model ID 通过 picker 编辑
├── Provider 变更后 model_id 清空
├── Save Template 是焦点驱动的 Enter 动作
├── Ctrl+J 在 editor 内触发 live sync（不影响 draft）
├── Esc 返回 Landing；dirty 时 leave confirm
├── stale/orphan 状态正确检测和展示
├── stale 不会因 reload/re-enter 静默消失
├── 保存成功 / 失败 / 降级三种状态可区分
├── dirty 离开必须确认

回归验收：
├── Phase 1 OpenCode Provider 不受影响
├── Phase 2 OMO Landing 不受影响
├── 新建 / 编辑 / 复制 / 切换 / 删除 template 全链路正常
```

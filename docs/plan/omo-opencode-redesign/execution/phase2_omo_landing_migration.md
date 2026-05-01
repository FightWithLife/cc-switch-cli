# Phase 2：OMO Landing + 迁移

> 架构设计：`../code-architecture/20260429_code_architecture_design.md`
>
> 产品 PRD：`../product-prd/20260429_omo_opencode_product_prd.md`
>
> 范围：OMO Agent Config 的 Landing 页面、DB schema、DAO、迁移检测、reload+sync
>
> 依赖：Phase 1 完成（OpenCode provider 数据结构已就绪，OMO 的 provider_id 依赖 providers 数据）

## 阶段总览

```text
Phase 2 拆分为 4 个阶段：

2.1 DB 基础      ← 无 UI 依赖，可立即开始
2.2 DAO + 数据层  ← 依赖 2.1
2.3 Landing 页面  ← 依赖 2.2
2.4 迁移与 sync   ← 依赖 2.3
```

---

## 2.1 DB 基础

**目标**：创建 OMO 相关的数据库表和 migration。

**改动文件**：

```text
src/database/schema.rs      修改  新增 omo_templates + omo_template_bindings 表
src/database/dao/mod.rs     修改  pub mod omo
src/database/dao/omo.rs     新增  OMO DAO（CRUD / import / sync state）
src/settings.rs             修改  +OmoAgentBinding 结构体 + oh-my-openagent.json 读写函数
```

**具体任务**：

- [ ] 在 `schema.rs` 中新增 migration：
  ```sql
  -- omo_templates
  CREATE TABLE IF NOT EXISTS omo_templates (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      is_active BOOLEAN NOT NULL DEFAULT 0,
      source TEXT NOT NULL DEFAULT 'db',
      last_sync_error TEXT,
      last_synced_at INTEGER,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
  );
  CREATE UNIQUE INDEX idx_omo_templates_single_active
      ON omo_templates(is_active) WHERE is_active = 1;

  -- omo_template_bindings
  CREATE TABLE IF NOT EXISTS omo_template_bindings (
      template_id INTEGER NOT NULL,
      agent_key TEXT NOT NULL,
      provider_id TEXT NOT NULL,
      model_id TEXT NOT NULL,
      binding_state TEXT NOT NULL DEFAULT 'bound',
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      PRIMARY KEY (template_id, agent_key),
      FOREIGN KEY (template_id) REFERENCES omo_templates(id) ON DELETE CASCADE
  );
  CREATE INDEX idx_omo_template_bindings_template
      ON omo_template_bindings(template_id);
  ```

- [ ] 在 `dao/omo.rs` 中实现 DAO 方法：
  - `get_all_templates() -> Vec<OmoTemplateRow>`
  - `get_active_template() -> Option<OmoTemplateRow>`
  - `get_template_by_id(id) -> Option<(OmoTemplateRow, Vec<(String, OmoAgentBinding)>)>`
  - `create_template(name, source) -> i64`
  - `update_template_name(id, name)`
  - `set_active_template(id)` — 先清除旧 active，再设置新 active
  - `delete_template(id)` — CASCADE 自动删除 bindings
  - `upsert_binding(template_id, agent_key, provider_id, model_id)`
  - `delete_binding(template_id, agent_key)`
  - `get_bindings(template_id) -> Vec<(String, OmoAgentBinding)>`
  - `set_sync_state(template_id, synced_at, error)`

- [ ] 在 `settings.rs` 中新增：
  - `OmoAgentBinding` 结构体（provider_id, model_id）
  - `get_oh_my_openagent_config_path() -> PathBuf`
  - `read_oh_my_openagent_config() -> Result<Value>`
  - `write_oh_my_openagent_config(config: &Value) -> Result<()>`
  - `get_omo_agent_bindings_from_settings() -> Option<HashMap<String, OmoAgentBinding>>`（读旧层）

- [ ] 在 `dao/omo.rs` 中实现迁移标记：
  - `is_legacy_import_completed() -> bool`（读 settings 表中的 marker）
  - `mark_legacy_import_completed()`（写 marker）

- [ ] 编写单元测试：
  - CRUD 循环：create template → upsert bindings → get → delete
  - active template 唯一性：set_active 后旧 active 自动清除
  - CASCADE 删除：delete template 后 bindings 自动清除
  - sync state 读写

**完成标志**：

- `cargo build` 通过
- `cargo test` 通过（含 OMO DAO 单元测试）
- migration 可以在空数据库上正确执行

---

## 2.2 数据层：OmoTemplateDraft + Sync

**目标**：定义 `OmoTemplateDraft` 结构体，实现 DB ↔ Draft 双向转换，实现 oh-my-openagent.json sync 函数。

**改动文件**：

```text
src/cli/tui/app/types.rs           修改  +OmoTemplateDraft 结构体
src/cli/tui/app/helpers.rs         修改  +build_omo_draft_from_db() 辅助函数
src/services/config.rs             修改  +sync_omo_template_to_live()
```

**具体任务**：

- [ ] 在 `types.rs` 中新增 `OmoTemplateDraft`：
  ```rust
  pub struct OmoTemplateDraft {
      pub template_id: Option<i64>,       // None = 新建
      pub template_name: String,
      pub is_active: bool,
      pub bindings: IndexMap<String, OmoAgentBinding>,
      pub agent_catalog: Vec<String>,     // 硬编码 catalog
      pub dirty: bool,
      pub original_snapshot: (String, IndexMap<String, OmoAgentBinding>),
  }
  ```
  方法：`is_dirty()` / `set_binding(key, binding)` / `clear_binding(key)` / `get_binding(key)`

- [ ] 在 `helpers.rs` 中实现：
  - `build_omo_draft_from_db(db, template_id) -> OmoTemplateDraft` — 从 DB 读取 template + bindings，初始化 draft
  - `build_omo_draft_new() -> OmoTemplateDraft` — 新建空白 draft，agent list 预填充 catalog
  - `build_omo_draft_from_legacy(bindings) -> OmoTemplateDraft` — 从旧 bindings 构建 draft

- [ ] 在 `services/config.rs` 中实现 `sync_omo_template_to_live()`：
  1. 读取 DB active template 的 bindings
  2. 读取 oh-my-openagent.json（不存在则创建骨架）
  3. 遍历 bindings → 写入 `agents.<key>.model = "<provider_id>/<model_id>"`
  4. catalog 中有但 bindings 中无的 key → 保留 JSON 已有值（不覆盖）
  5. 写入 JSON 文件
  6. 成功 → 更新 DB `last_synced_at`
  7. 失败 → 更新 DB `last_sync_error`

- [ ] 编写单元测试：
  - Draft 脏检测：修改 binding → dirty = true
  - Draft → DB 保存 → 重读 roundtrip
  - sync 函数：正常 sync / JSON 文件不存在时创建 / JSON 写入失败降级

**完成标志**：

- `cargo test` 通过
- Draft 可以从 DB 初始化、编辑、脏检测、保存回 DB
- sync 函数可以正确写入 oh-my-openagent.json

---

## 2.3 Landing 页面

**目标**：实现 OMO Landing 页面，包含 template 列表、template 级动作、导航集成。

**改动文件**：

```text
src/cli/tui/route.rs               修改  +Route::OmoAgentConfig
src/cli/tui/app/menu.rs            修改  +NavItem::OmoAgentConfig（OpenCode only）
src/cli/tui/ui.rs                  修改  +render_omo dispatch
src/cli/tui/ui/omo_landing.rs      新增  OMO Landing 页面渲染
src/cli/tui/runtime_actions/omo.rs 新增  Landing 按键处理
src/cli/tui/data.rs                修改  ConfigSnapshot 新增 OMO 字段
src/cli/tui/app/types.rs           修改  +OmoTemplateRow 结构体
src/cli/i18n/texts/omo.rs          新增  OMO 文案
```

**具体任务**：

- [ ] 在 `route.rs` 中新增 `Route::OmoAgentConfig`

- [ ] 在 `menu.rs` 中新增 `NavItem::OmoAgentConfig`，仅 OpenCode app_type 可见，位置在 Prompts 下方

- [ ] 在 `data.rs` 的 `ConfigSnapshot` 中新增：
  - `omo_template_rows: Vec<OmoTemplateRow>`
  - `omo_active_template_name: Option<String>`
  - `omo_live_json_status: LiveJsonStatus`（Ready / Absent / ParseError）

- [ ] 新增 `ui/omo_landing.rs`：
  - 渲染 template 列表（`[active]` 内联标记）
  - 底部快捷键提示
  - Live target 状态行
  - 空态 UI（"No templates found. Press [n] to create."）

- [ ] 新增 `runtime_actions/omo.rs` Landing 按键：
  - `Enter`：进入选中 template 的 editor（Route::OmoTemplateEditor）
  - `n`：新建 template → 进入 editor
  - `s`：切换 active template → sync → 刷新列表
  - `c`：复制 template → 进入 editor
  - `r` / `Ctrl+J`：重读 DB + sync JSON + 刷新 UI
  - `Del`：确认 → 删除 → 重选 active
  - `Esc`：返回左导航

- [ ] 在 `i18n/texts/omo.rs` 中新增所有 Landing 相关文案

- [ ] 验证导航：`左导航 Enter → OMO Landing → Enter on template → (占位 editor) → Esc → Landing → Esc → 左导航`

**完成标志**：

- `cargo run` 进入 interactive mode
- OpenCode 左侧导航中出现 OMO Agent Config
- Landing 正确展示 template 列表（空态时显示引导文案）
- 所有 template 级动作可执行（Enter/n/s/c/r/Del/Esc）
- r/Ctrl+J 正确触发 reload + sync

---

## 2.4 迁移与 Sync 集成

**目标**：实现旧 bindings → DB template 的迁移流程，集成 oh-my-openagent.json sync 到 Landing 生命周期。

**改动文件**：

```text
src/cli/tui/ui/omo_landing.rs      修改  迁移检测 + Migration Notice
src/cli/tui/runtime_actions/omo.rs 修改  迁移确认处理
src/cli/tui/app/types.rs           修改  +Overlay::OmoMigrationNotice
src/cli/tui/app/overlay_handlers/dialogs.rs  修改  迁移确认 overlay
```

**具体任务**：

- [ ] 在 Landing 加载流程中新增迁移检测：
  1. DB 中有 omo_templates → 跳过迁移
  2. DB 无 templates + settings.json 有 omoAgentBindings → 显示 Migration Notice
  3. DB 无 templates + settings.json 无 bindings + oh-my-openagent.json 有 agents → 可选提示
  4. 都没有 → 空 landing

- [ ] 实现 Migration Notice overlay：
  ```
  +--------------------------------------------------------------+
  | Migration Notice                                             |
  |--------------------------------------------------------------|
  | Existing local OMO bindings were found.                      |
  | Import them into DB-backed OMO templates before sync?        |
  |                                                              |
  | [Enter] Yes, import   [Esc] No, not now                      |
  +--------------------------------------------------------------+
  ```

- [ ] 迁移执行流程：
  1. 读取 `settings.json.omoAgentBindings`
  2. 创建 template "Imported Legacy"，source = 'legacy_settings_import'
  3. 复制所有 bindings 到 DB
  4. 标记 `omo_legacy_import_completed`
  5. 导航到 editor 让用户确认

- [ ] sync 集成到 Landing 生命周期：
  - 进入 Landing 时：读取 DB templates + 检测 live JSON 状态
  - `s` 切换后：立即 sync
  - `r`/`Ctrl+J`：reload + sync
  - sync 结果作为次级状态反馈（不与 `[active]` 竞争）

- [ ] 删除 active template 后的处理：
  - 确认对话框
  - 删除后若有剩余 template → 按 template name ASCII 升序选最小项为新 active → sync
  - 删除后无剩余 → active 为空，live sync 进入"无 active template"状态

- [ ] 编写集成测试：
  - 旧 bindings → 导入 → DB template 存在 → marker 已设置
  - 重复进入 Landing 不会重复导入
  - sync 正确写入 oh-my-openagent.json

**完成标志**：

- `cargo test` 通过
- 手工验收：有旧 bindings 时进入 Landing → Migration Notice → Enter → 导入成功 → editor 中看到 bindings
- 手工验收：切换 template → sync → oh-my-openagent.json 更新
- 手工验收：删除 active template → 自动选择新 active

---

## Phase 2 完成标准

```text
功能验收：
├── OMO Agent Config 出现在 OpenCode 左侧导航
├── Landing 只展示 template 列表和 template 级动作
├── 当前 active template 以 [active] 内联标记展示
├── sync 状态作为次级反馈展示
├── 旧 bindings 可被用户确认导入
├── 导入后不重复生成等价 template
├── 切换 template 后自动 sync
├── 删除 active template 后自动选择新 active
├── r/Ctrl+J 正确执行 reload + sync
├── DB committed + JSON sync failed 可区分

回归验收：
├── Phase 1 的 OpenCode Provider 改造不受影响
├── 非 OpenCode app_type 不显示 OMO 导航项
```

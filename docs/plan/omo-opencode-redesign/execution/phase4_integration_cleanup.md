# Phase 4：集成、收敛与收尾

> 架构设计：`../code-architecture/20260429_code_architecture_design.md`
>
> 产品 PRD：`../product-prd/20260429_omo_opencode_product_prd.md`
>
> 范围：跨模块集成验证、settings.json 双写收敛、回归测试、文档固化
>
> 依赖：Phase 1 + Phase 2 + Phase 3 全部完成

## 阶段总览

```text
Phase 4 拆分为 3 个阶段：

4.1 跨模块集成验证  ← 依赖 Phase 1-3
4.2 存储收敛        ← 依赖 4.1
4.3 回归测试 + 文档  ← 依赖 4.2
```

---

## 4.1 跨模块集成验证

**目标**：验证 OMO 和 OpenCode Provider 两个模块之间的交互正确性。

**验证场景**：

- [ ] **OMO provider picker 使用 OpenCode provider 数据**：
  1. 新建一个 OpenCode provider "test-provider"，添加 model "test-model"，保存
  2. 进入 OMO Template Editor
  3. 选中一个 agent → Enter on Provider → picker 中应出现 "test-provider"
  4. 选择 "test-provider" → Enter on ModelID → picker 中应出现 "test-model"
  5. 选择 "test-model" → binding 正确设置

- [ ] **OpenCode provider 删除后 OMO stale 检测**：
  1. OMO template 中有一个 agent 绑定到 "test-provider/test-model"
  2. 删除 OpenCode provider "test-provider"
  3. 进入 OMO Template Editor
  4. 该 agent 应显示 stale（missing provider）

- [ ] **OpenCode provider 的 model 删除后 OMO stale 检测**：
  1. OMO template 中有一个 agent 绑定到 "test-provider/test-model"
  2. 编辑 OpenCode provider，删除 "test-model"
  3. 进入 OMO Template Editor
  4. 该 agent 应显示 stale（missing model）

- [ ] **OMO sync 与 OpenCode opencode.json 共存**：
  1. OMO sync 写入 oh-my-openagent.json
  2. OpenCode provider 保存写入 opencode.json
  3. 两个文件互不干扰
  4. 重启后两个文件内容都正确

- [ ] **OpenCode 新建 provider 后 OMO model picker 立即可用**：
  1. 新建 OpenCode provider + model → 保存
  2. 不重启，直接进入 OMO Editor
  3. provider picker 中应出现新 provider
  4. model picker 中应出现新 model

**完成标志**：

- 所有跨模块验证场景通过
- OMO 和 OpenCode 的数据互不干扰但可以正确引用

---

## 4.2 存储收敛

**目标**：消除 settings.json 双写，确保 DB 是唯一编辑态真值。

**改动文件**：

```text
src/settings.rs               修改  迁移完成后停止向 settings.json 写 OMO bindings
src/cli/tui/runtime_actions/omo.rs  修改  保存流程中移除 settings.json 镜像
```

**具体任务**：

- [ ] 确认 settings.json 的 OMO 镜像只在迁移期写入：
  - 迁移完成 + 首次 sync 成功后，`set_omo_agent_bindings()` 不再写 settings.json
  - DB 是唯一编辑态真值
  - oh-my-openagent.json 是 live sync target

- [ ] 确认 opencode.json 的 currentModel 字段正确：
  - 删除 provider 后 currentModel 不悬空
  - rename model 后 currentModel 正确修复
  - 重启后 opencode.json 内容与 DB 一致

- [ ] 确认旧 settings.json.omoAgentBindings 不会反向覆盖 DB：
  - DB 已有 template 时，settings.json 中的旧 bindings 不会被重新加载为 editable state
  - 读取优先级：DB > settings.json（仅迁移输入）

**完成标志**：

- 迁移完成后 settings.json 中的 omoAgentBindings 不再更新
- DB 是唯一编辑态真值
- oh-my-openagent.json 是唯一 live sync target

---

## 4.3 回归测试 + 文档

**目标**：补齐测试覆盖，固化文档。

**改动文件**：

```text
src/cli/tui/tests.rs              修改  TUI 交互测试
src-tauri/tests/                  修改  集成测试
docs/plan/omo-opencode-redesign/code-architecture/  修改  架构文档更新
```

**具体任务**：

- [ ] **单元测试补齐**：
  - OpenCodeProviderDraft：脏检测、model CRUD、rename、currentModel 回退
  - OmoTemplateDraft：脏检测、binding CRUD、stale 检测
  - opencode_config：currentModel 读写、multi-model 序列化
  - settings.rs：OmoAgentBinding 序列化、oh-my-openagent sync

- [ ] **集成测试补齐**：
  - OMO：legacy bindings → DB template → oh-my-openagent.json 完整链路
  - OMO：stale/orphan 检测 + closure 规则
  - OMO：DB committed + JSON sync failed 降级
  - OpenCode：provider save → opencode.json 多 model + currentModel
  - OpenCode：modelId rename 原子性 + currentModel 修复

- [ ] **TUI 交互测试补齐**：
  - OMO：Landing 进入 / 返回 / template 切换
  - OMO：Editor 焦点链 TemplateName → AgentsList → Details → Save
  - OMO：Provider/Model picker 打开 / 选择 / 取消
  - OpenCode：ProviderForm → ModelConfigList → ModelConfigDetail 导航
  - OpenCode：Model ID picker fetch → select → write-back
  - 通用：dirty leave confirm

- [ ] **手工验收路径**（最终 cargo run 验证）：
  1. OpenCode：新建 provider → 添加 2 个 model → 保存 → opencode.json 验证
  2. OpenCode：删除 currentModel → currentModel 自动回退
  3. OpenCode：rename model → 旧 key 消失、新 key 存在
  4. OMO：从 Landing 新建 template → 编辑 → 保存 → sync
  5. OMO：旧 bindings 迁移 → 导入 → 编辑
  6. OMO：stale binding → rebind → stale 消失
  7. OMO：dirty 离开 → leave confirm
  8. OMO：DB committed + JSON sync failed 降级提示
  9. 跨模块：OMO picker 引用 OpenCode provider/model
  10. 跨模块：删除 OpenCode provider → OMO stale 检测

- [ ] **CI 检查**：
  - `cargo fmt --check` 通过
  - `cargo test` 全部通过
  - `cargo clippy` 无新增 warning

- [ ] **文档固化**：
  - 更新架构文档中的实际实现差异（如果有）
  - 更新 PRD 中的非目标范围说明
  - 确认 `README.md` 指向正确的文档入口

**完成标志**：

- 所有测试通过
- 手工验收路径全部走通
- CI 检查全部通过
- 文档与实现一致

---

## Phase 4 完成标准

```text
功能验收（PRD §8 全量对照）：

OMO §8.1：
✅ 用户能从 OpenCode 左侧导航进入 OMO Agent Config
✅ Landing 只展示 template 列表和 template 级动作
✅ 用户能打开、新建、复制、切换、删除 template
✅ Template Editor 具备 Template Name、Agents List、Agent Details、Save Template
✅ 用户能通过 picker 修改 Provider 与 Model ID
✅ Provider 改变后，当前 Model ID 被清空并要求重新选择
✅ 旧 bindings 可被用户确认导入
✅ stale/orphan 状态不会在未处理时静默消失
✅ 保存成功、保存失败、DB 已提交但 JSON 同步失败三种状态可区分
✅ dirty 离开必须确认

OpenCode §8.2：
✅ Provider Form 中有 Model Config 入口
✅ Model Config List 以 model name 展示
✅ 用户能新建、编辑、删除 model config
✅ Model Detail 固定包含 Model Name、Model ID、Input Limit、Output Limit
✅ Model ID 通过 fetch/picker 选择
✅ fetch loading、empty、error、blocked 状态可区分
✅ Model ID 写回纯 model id
✅ Model Name 为空时保存使用 model id 大写作为默认名
✅ limit 为空表示未设置，非空必须是正整数
✅ duplicate model id 阻止保存
✅ 删除或改名 current model 后不会留下悬空 current model
✅ dirty 离开必须确认

CI 验收：
✅ cargo fmt --check
✅ cargo test
✅ cargo clippy
```

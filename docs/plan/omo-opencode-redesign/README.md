# OMO / OpenCode Redesign 文档入口

本目录是 OMO Agent Config 与 OpenCode Provider Model Config 的新设计文档体系。

## 当前入口

- 产品 PRD：`product-prd/20260429_omo_opencode_product_prd.md`
- 代码架构规范：`code-architecture/20260429_code_architecture_design.md`
- 执行计划：
  - Phase 1：`execution/phase1_opencode_provider_upgrade.md`
  - Phase 2：`execution/phase2_omo_landing_migration.md`
  - Phase 3：`execution/phase3_omo_template_editor.md`
  - Phase 4：`execution/phase4_integration_cleanup.md`

## 文档职责

### 产品 PRD

`product-prd/` 只描述目标产品体验：

- 用户目标；
- 页面结构；
- UI text 图；
- 用户交互逻辑盒图；
- 空态、异常态、确认态；
- 产品验收标准。

PRD 不描述当前仓库实现进展，不定义 Rust 类型、文件拆分、数据库 schema 或模块调用关系。

### 代码架构规范

`code-architecture/` 描述从现状到 PRD 目标态的技术路径：

- 现状审计结论；
- 总体架构盒图；
- 数据结构变更；
- 页面层级与路由；
- 数据流、错误流、测试分层；
- 文件变更清单；
- 依赖顺序。

### 执行计划

`execution/` 按 Phase 拆分具体执行步骤：

- Phase 1：OpenCode Provider 改造（multi-model + 三层页面 + fetch/picker + 保存链路）
- Phase 2：OMO Landing + 迁移（DB schema + DAO + Landing + 迁移检测 + sync）
- Phase 3：OMO Template Editor + Stale 检测（全页 editor + picker + stale/orphan + 保存）
- Phase 4：集成、收敛与收尾（跨模块验证 + 存储收敛 + 回归测试 + 文档固化）

## 维护规则

- 新产品决策写入 `product-prd/`。
- 新实现决策写入 `code-architecture/`。
- 执行进度和任务变更写入 `execution/`。
- 不再把产品、架构、执行进度混写到同一份文档。
- 若产品 PRD 与代码架构规范冲突，先更新产品决策，再调整架构方案。

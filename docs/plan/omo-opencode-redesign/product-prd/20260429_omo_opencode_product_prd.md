# OMO / OpenCode Redesign 产品 PRD

## 1. 产品目标

本 PRD 定义 OMO Agent Config 与 OpenCode Provider Model Config 的目标产品体验。

核心目标：

- 让 OpenCode 用户能在 TUI 内独立管理 OMO agent template；
- 让用户能维护 provider 下的多个 model config，而不是只编辑单个 Model ID；
- 用清晰的页面层级、键位提示和状态反馈减少误操作；
- 区分“编辑真值”和“live 文件同步状态”，避免用户误以为 JSON 文件就是编辑源；
- 所有关键页面都要有可理解的 text UI 和可验证的交互路径。

本 PRD 不描述当前代码实现，不引用当前进度，不定义代码架构。

## 2. 用户与场景

### 2.1 用户角色

- OpenCode 日常用户：需要切换 provider、模型、OMO agent 模板；
- 配置维护者：需要整理 provider 下的模型列表和限额参数；
- 高级用户：需要从旧 OMO 绑定迁移到新模板体系，并处理 stale binding。

### 2.2 核心场景

- 用户进入 OpenCode TUI 后，从左侧导航进入 OMO Agent Config；
- 用户浏览、切换、复制、删除 OMO template；
- 用户进入 template editor，为不同 agent 选择 provider 和 model；
- 用户从旧 OMO bindings 导入初始 template；
- 用户看到 live JSON 同步失败时，能明确知道 DB 已保存但 JSON 未同步；
- 用户在 Provider Form 中进入 Model Config，维护多个 model；
- 用户在 Model Detail 中 fetch 可用模型并写回纯 model id；
- 用户离开有未保存修改的页面时得到确认提示。

## 3. 信息架构总览

```text
OpenCode
  |
  +-- Main
  +-- Providers
  |     |
  |     +-- Provider Form
  |           |
  |           +-- Model Config List
  |                 |
  |                 +-- Model Config Detail
  |                       |
  |                       +-- Model ID Picker
  |
  +-- Mcp
  +-- Skills
  +-- Prompts
  +-- OMO Agent Config
  |     |
  |     +-- OMO Landing
  |           |
  |           +-- OMO Template Editor
  |                 |
  |                 +-- Provider Picker
  |                 +-- Model Picker
  |
  +-- Config
  +-- Settings
  +-- Exit
```

## 4. OMO Agent Config PRD

### 4.1 左侧导航

```text
+---------------------------+
| OpenCode                  |
|---------------------------|
| Main                      |
| Providers                 |
| Mcp                       |
| Skills                    |
| Prompts                   |
| > OMO Agent Config        |
| Config                    |
| Settings                  |
| Exit                      |
+---------------------------+
```

交互要求：

- `OMO Agent Config` 是 OpenCode 下的独立导航项；
- 位置在 `Prompts` 下方；
- 用户按 `Enter` 进入 OMO Landing；
- 不允许把 OMO 入口藏在 Config 或 Prompts 的弹层里。

### 4.2 OMO Landing

```text
+--------------------------------------------------------------------------------+
| OMO Agent Config                                                               |
|--------------------------------------------------------------------------------|
| [Enter] details   [s] switch   [c] copy   [r] refresh   [n] add   [Del] delete |
|--------------------------------------------------------------------------------|
| Templates                                                                      |
| > [active] team-default                                                        |
|            prod-safe                                                           |
|            fast-iter                                                           |
|            imported-legacy                                                     |
|--------------------------------------------------------------------------------|
| Live target: ready                                                             |
| Ctrl+J sync active template to oh-my-openagent.json                            |
+--------------------------------------------------------------------------------+
```

页面要求：

- Landing 只展示 template 列表和 template 级动作；
- 当前 active template 在列表内以内联标记展示；
- sync 状态只能作为次级状态展示，不能和 active 标记竞争；
- 页面不展示 agent list、agent details、provider/model 字段编辑；
- 页面不提供 template 命名 modal。

交互逻辑盒图：

```text
[OMO Landing]
        |
        +-- Enter --> [Open selected Template Editor]
        |
        +-- n ------> [Open blank Template Editor]
        |
        +-- c ------> [Copy selected Template] -> [Open copied Template Editor]
        |
        +-- s ------> [Switch active Template] -> [Sync live target] -> [Refresh active marker]
        |
        +-- Del ----> [Delete Confirm] -> [Delete Template] -> [Recompute active Template]
        |
        +-- r ------> [Reload Templates and live status]
        |
        +-- Ctrl+J -> [Sync active Template to live target]
```

### 4.3 OMO Template Editor

```text
+--------------------------------------------------------------------------------------------------+
| Edit OMO Template: team-default                                                                  |
|--------------------------------------------------------------------------------------------------|
| > Template Name: team-default                                                                    |
|--------------------------------------------------------------------------------------------------|
| Agents List                                 | Agent Details                                      |
|---------------------------------------------|----------------------------------------------------|
| > coder                                     | Agent    : coder                                   |
|   reviewer                                  | > Provider : synai                                 |
|   planner                                   |   Model ID : gpt-5.4                               |
|   architect                                 |   Final    : synai/gpt-5.4                         |
|                                             |   Status   : bound                                 |
|---------------------------------------------|----------------------------------------------------|
| [Save Template]                                                                                  |
| [Enter] edit/select/save   [Up/Down] move focus   [Ctrl+J] sync   [Esc] back                     |
+--------------------------------------------------------------------------------------------------+
```

页面要求：

- Template Editor 是新增和编辑 template 的唯一主路径；
- 初始焦点在 `Template Name`；
- 左侧是 agent 列表，右侧是当前 agent 详情；
- Provider 和 Model ID 是右侧详情中的可编辑字段；
- Save Template 是焦点驱动动作，焦点在其上按 `Enter` 保存；
- `Ctrl+J` 可在 editor 内触发 live sync；
- `Esc` 返回 landing；dirty 时先弹离开确认。

焦点逻辑盒图：

```text
[Template Name]
        |
        | Down
        v
[Agents List] -- Enter on selected agent --> [Agent Details: Provider]
        ^                                             |
        | Up from first item                          | Down
        |                                             v
        +------------------------------------- [Agent Details: Model ID]
                                                      |
                                                      | Down
                                                      v
                                               [Save Template]
```

字段编辑盒图：

```text
[Provider field]
        |
        | Enter
        v
[Provider Picker]
        |
        | Select provider
        v
[Set provider] -> [Clear current model id] -> [Mark template dirty]
```

```text
[Model ID field]
        |
        | Enter
        v
[Has provider?]
        |
        +-- No  --> [Blocked feedback: select provider first]
        |
        +-- Yes --> [Model Picker for selected provider]
                     |
                     | Select model
                     v
              [Set pure model id] -> [Mark template dirty]
```

保存盒图：

```text
[Save Template]
        |
        | Enter
        v
[Validate template name and bindings]
        |
        +-- Invalid --> [Stay editor + show feedback]
        |
        +-- Valid ----> [Save template]
                         |
                         +-- Save failed --> [Stay editor + show error]
                         |
                         +-- Save ok ------> [Sync oh-my-openagent.json]
                                             |
                                             +-- Sync ok -----> [Saved feedback]
                                             |
                                             +-- Sync failed -> [DB committed, JSON sync failed]
```

### 4.4 OMO Provider Picker

```text
+--------------------------------------------------------------+
| Select Provider                                              |
|--------------------------------------------------------------|
| Search: syn                                                  |
|--------------------------------------------------------------|
| > synai                                                      |
|   openai                                                     |
|   anthropic                                                  |
|--------------------------------------------------------------|
| Source: OpenCode providers                                   |
| [Enter] Select   [Esc] Cancel                                |
+--------------------------------------------------------------+
```

要求：

- 从当前可用 OpenCode providers 中选择；
- 支持搜索；
- 取消后回到 Provider 字段；
- 选择后清空当前 agent 的 Model ID。

### 4.5 OMO Model Picker

```text
+--------------------------------------------------------------+
| Select Model ID                                              |
|--------------------------------------------------------------|
| Provider: synai                                              |
| Search: gpt                                                  |
|--------------------------------------------------------------|
| > gpt-5.4                                                    |
|   gpt-4.1                                                    |
|   gpt-4o-mini                                                |
|--------------------------------------------------------------|
| [Enter] Select   [Esc] Cancel                                |
+--------------------------------------------------------------+
```

要求：

- 只展示当前 provider 下可用 model；
- 未选择 provider 时不打开 picker，展示 blocked feedback；
- 选择后只写回纯 model id；
- 最终同步到 live target 时，展示值可组装为 `provider/model`。

### 4.6 OMO Migration Notice

```text
+--------------------------------------------------------------+
| Migration Notice                                             |
|--------------------------------------------------------------|
| Existing local OMO bindings were found.                      |
| Import them into DB-backed OMO templates before sync?        |
|                                                              |
| [Enter] Yes, import   [Esc] No, not now                      |
+--------------------------------------------------------------+
```

要求：

- 只在检测到旧 bindings 且尚未迁移时出现；
- 用户确认后生成可编辑 template；
- 用户拒绝后本次跳过，不阻塞进入新页面；
- 已迁移后不得重复生成等价 template。

### 4.7 OMO Stale / Orphan 状态

```text
+--------------------------------------------------------------+
| Stale Binding                                                |
|--------------------------------------------------------------|
| Agent       : reviewer                                       |
| Provider    : old-provider                                   |
| Model ID    : old-model                                      |
| Status      : missing provider                               |
|                                                              |
| [Enter] Rebind   [Ctrl+X] Clear   [Esc] Back                 |
+--------------------------------------------------------------+
```

状态类型：

- `orphan agent key`
- `missing provider`
- `missing model`

关闭规则：

- orphan agent key 需要用户显式丢弃或映射；
- missing provider 需要重新绑定 provider 或清空 binding；
- missing model 需要重新选择 model 或清空 binding；
- reload、re-enter、sync retry 不应静默清除 warning。

### 4.8 OMO 保存与离开确认

```text
+----------------------------------------------+
| Save successful                              |
+----------------------------------------------+
```

```text
+----------------------------------------------+
| DB committed, JSON sync failed               |
+----------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Unsaved changes detected                                     |
|--------------------------------------------------------------|
| Save before leaving this editor?                             |
|                                                              |
| [Enter] Save and leave   [n] Discard   [Esc] Stay            |
+--------------------------------------------------------------+
```

要求：

- 保存成功和 sync 失败必须可区分；
- DB 已保存但 live sync 失败时不能显示为完全成功；
- dirty 离开必须确认。

## 5. OpenCode Provider Model Config PRD

### 5.1 Provider Form

```text
+------------------------------------------------------------------+
| Provider Form                                                    |
|------------------------------------------------------------------|
| Name            : synai                                          |
| Base URL        : https://api.example.com/v1                     |
| API Key         : ********                                       |
| NPM Package     : @ai-sdk/openai-compatible                      |
| > Model Config  : 3 models configured                            |
|------------------------------------------------------------------|
| [Enter] edit/open   [Ctrl+S] save provider   [Esc] back          |
+------------------------------------------------------------------+
```

页面要求：

- Provider Form 保留 Name、Base URL、API Key；
- NPM Package 作为可选高级字段，保留在 API Key 下方；默认值为 `@ai-sdk/openai-compatible`，大多数用户不需要修改，但使用 Anthropic 原生接口等场景需要改为 `@ai-sdk/anthropic`；
- NPM Package 下方展示 Model Config 入口；
- Model Config 行展示当前 model 数量；
- 焦点在 Model Config 上按 `Enter` 进入 Model Config List；
- Provider Form 保存时提交 provider 基础字段和 model 集合。

交互盒图：

```text
[Provider Form]
        |
        +-- Edit Name/Base URL/API Key/NPM Package --> [Update provider draft]
        |
        +-- Enter on Model Config -------> [Model Config List]
        |
        +-- Ctrl+S ----------------------> [Save provider and models]
        |
        +-- Esc (clean) -----------------> [Back to Providers list]
        |
        +-- Esc (dirty) -----------------> [Leave confirm]
```

### 5.2 Model Config List

```text
+------------------------------------------------------------------+
| Model Config: synai                                              |
|------------------------------------------------------------------|
| > GPT-5.4                                                        |
|   GPT-4.1                                                        |
|   GPT-4O-MINI                                                    |
|------------------------------------------------------------------|
| [Enter] edit model   [n] new model   [Del] delete   [Esc] back   |
+------------------------------------------------------------------+
```

页面要求：

- 列表以 model name 作为主展示；
- 未配置 name 的 model 展示为 model id 大写；
- `Enter` 进入选中 model 的 detail；
- `n` 新建 model 并进入 detail；
- `Del` 删除前必须确认；
- `Esc` 返回 Provider Form；若 shared draft 为 dirty（包括从 ModelConfigDetail 退回但未保存的情况），先进入 leave confirm。

交互盒图：

```text
[Model Config List]
        |
        +-- Enter selected --> [Model Config Detail: existing]
        |
        +-- n --------------> [Model Config Detail: new]
        |
        +-- Del ------------> [Delete confirm]
        |                       |
        |                       v
        |                 [Remove model]
        |                       |
        |                       v
        |                 [Repair current model if needed]
        |
        +-- Esc ------------> [Provider Form or Leave confirm]
```

删除当前 model 的回退规则：

- 若剩余 model 非空，current model 改为剩余 model id 中 ASCII/字节序最小项；
- 若没有剩余 model，current model 清空。

### 5.3 Model Config Detail

```text
+------------------------------------------------------------------+
| Model Config Detail: synai                                       |
|------------------------------------------------------------------|
| > Model Name    : GPT-5.4                                        |
|   Model ID      : gpt-5.4                                        |
|   Input Limit   : 200000                                         |
|   Output Limit  : 65536                                          |
|------------------------------------------------------------------|
| Input Area: current field buffer / picker status                 |
| [Enter] edit/open picker   [Ctrl+S] save   [Esc] back            |
+------------------------------------------------------------------+
```

字段要求：

- Model Name 通过底部输入区编辑；
- Model ID 按 `Enter` 打开 fetch/picker，不做自由文本输入主路径；
- Input Limit 和 Output Limit 通过底部输入区编辑；
- limit 可为空，空表示未设置；
- 非空 limit 必须是正整数；
- 重复 Model ID 阻止保存；
- 保存失败留在 detail 页，不丢用户输入。

字段交互盒图：

```text
[Model Name / Input Limit / Output Limit]
        |
        | Enter
        v
[Bottom input editing]
        |
        | Enter
        v
[Validate field] -> [Update draft]
```

```text
[Model ID]
        |
        | Enter
        v
[Check Base URL and auth]
        |
        +-- Not ready --> [Blocked feedback]
        |
        +-- Ready -----> [Fetch models]
                       |
                       +-- Loading --> [Loading state]
                       +-- Empty ----> [Empty state]
                       +-- Error ----> [Error state + retry]
                       +-- Success --> [Model ID Picker]
                                       |
                                       | Select
                                       v
                                [Write pure model id]
```

保存交互盒图：

```text
[Ctrl+S in Model Detail]
        |
        v
[Validate model fields]
        |
        +-- Invalid --> [Stay detail + show feedback]
        |
        +-- Valid ----> [Save provider snapshot]
                         |
                         +-- Save failed --> [Keep draft + show feedback]
                         |
                         +-- Save ok ------> [Sync opencode.json]
                                             |
                                             +-- Sync ok -----> [Saved feedback]
                                             |
                                             +-- Sync failed -> [DB committed, JSON sync failed]
```

### 5.4 Model ID Picker

```text
+--------------------------------------------------------------+
| Select Model ID                                              |
|--------------------------------------------------------------|
| Search: gpt                                                  |
|--------------------------------------------------------------|
| > gpt-5.4                                                    |
|   gpt-4.1                                                    |
|   gpt-4o-mini                                                |
|--------------------------------------------------------------|
| [Enter] Select   [Esc] Cancel                                |
+--------------------------------------------------------------+
```

要求：

- picker 支持搜索；
- 选择结果写回纯 model id；
- 不写回 `provider/model`；
- 若 Model Name 为空，保存时用 model id 大写作为默认展示名。

### 5.5 Fetch 状态

```text
+--------------------------------------------------------------+
| Select Model ID                                              |
|--------------------------------------------------------------|
| Fetching models from provider...                             |
|                                                              |
| Please wait                                                  |
+--------------------------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Select Model ID                                              |
|--------------------------------------------------------------|
| No models returned from provider                             |
|                                                              |
| [Esc] Back to detail                                         |
+--------------------------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Failed to fetch models                                       |
|--------------------------------------------------------------|
| Reason: unauthorized / network error / invalid response      |
|                                                              |
| [r] Retry   [Esc] Back                                       |
+--------------------------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Cannot fetch models                                          |
|--------------------------------------------------------------|
| Base URL or API Key is missing for current provider draft.   |
|                                                              |
| [Esc] Back to detail                                         |
+--------------------------------------------------------------+
```

### 5.6 输入错误与删除确认

```text
+--------------------------------------------------------------+
| Cannot save model config                                     |
|--------------------------------------------------------------|
| Reason: duplicated model id `gpt-5.4`                        |
|                                                              |
| [Esc] Back                                                   |
+--------------------------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Invalid numeric value                                        |
|--------------------------------------------------------------|
| Input Limit / Output Limit must be positive integers         |
|                                                              |
| [Esc] Back                                                   |
+--------------------------------------------------------------+
```

```text
+--------------------------------------------------------------+
| Delete model config?                                         |
|--------------------------------------------------------------|
| This action removes `GPT-5.4` from current provider.         |
|                                                              |
| [Enter] Delete   [Esc] Cancel                                |
+--------------------------------------------------------------+
```

## 6. 跨模块边界

| 项目 | OMO Agent Config | OpenCode Provider Model Config |
| --- | --- | --- |
| 用户入口 | OMO Agent Config 导航项 | Provider Form 的 Model Config 行 |
| 主编辑对象 | OMO template | provider 下的 model config |
| Provider/Model 语义 | agent binding 使用 provider + model | model config 使用纯 model id |
| Live 文件 | `oh-my-openagent.json` | `opencode.json` |
| Model ID 写回 | template 内保存纯 model id，live sync 可组装 provider/model | 始终纯 model id |
| 主要风险 | stale/orphan binding、sync 失败 | duplicate model id、limit 非法、current model 悬空 |

## 7. 非目标范围

本 PRD 不覆盖：

- 代码架构；
- Rust 类型与文件拆分；
- 数据库 schema 细节；
- 迁移脚本实现方式；
- 当前仓库已有实现的保留或删除策略；
- Git 提交拆分；
- 构建、测试命令细节。

## 8. 产品验收标准

### 8.1 OMO

- 用户能从 OpenCode 左侧导航进入 OMO Agent Config；
- Landing 只展示 template 列表和 template 级动作；
- 用户能打开、新建、复制、切换、删除 template；
- Template Editor 具备 Template Name、Agents List、Agent Details、Save Template；
- 用户能通过 picker 修改 Provider 与 Model ID；
- Provider 改变后，当前 Model ID 被清空并要求重新选择；
- 旧 bindings 可被用户确认导入；
- stale/orphan 状态不会在未处理时静默消失；
- 保存成功、保存失败、DB 已提交但 JSON 同步失败三种状态可区分；
- dirty 离开必须确认。

### 8.2 OpenCode Provider

- Provider Form 中有 Model Config 入口；
- Model Config List 以 model name 展示；
- 用户能新建、编辑、删除 model config；
- Model Detail 固定包含 Model Name、Model ID、Input Limit、Output Limit；
- Model ID 通过 fetch/picker 选择；
- fetch loading、empty、error、blocked 状态可区分；
- Model ID 写回纯 model id；
- Model Name 为空时保存使用 model id 大写作为默认名；
- limit 为空表示未设置，非空必须是正整数；
- duplicate model id 阻止保存；
- 删除或改名 current model 后不会留下悬空 current model；
- dirty 离开必须确认。

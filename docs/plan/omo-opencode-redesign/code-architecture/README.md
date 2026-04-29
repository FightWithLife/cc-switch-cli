# 代码架构规范

本目录用于下一步重新设计 OMO Agent Config 与 OpenCode Provider Model Config 的代码架构。

## 本轮状态

本轮先不展开代码架构设计，只保留目录入口。

## 下一步应补充

- 总体代码架构盒图；
- 新增文件清单；
- 新增结构体与职责；
- OMO 核心逻辑盒图；
- OpenCode Provider 核心逻辑盒图；
- DB 与 live JSON 的数据流；
- 错误与降级状态流；
- 测试分层与验收入口。

## 边界

- 本目录不承载产品交互需求。
- 产品目标、UI text 图和用户交互逻辑以 `../product-prd/20260429_omo_opencode_product_prd.md` 为准。
- 架构设计必须服务 PRD，不反向改变 PRD 的用户体验目标。

# Agent Work

## 1. 定位
- `docs/agent/agent_work.md` 是开发版文档的总入口。
- 它用于管理协作过程中产生的规则、Prompt、分析、迁移说明、构建辅助说明和其他过程性文档。

## 2. 文档架构

### 2.1 发布版文档
- `README.md`
- `TESTING.md`

发布版文档只记录发布版自身内容：
- RTOS 运行环境
- RTOS 架构与目录
- 功能状态
- 使用方法
- 参数调节方法
- 测试矩阵
- 通过标准
- 最近一次 RTOS 测试结果

### 2.2 开发版文档
- `docs/agent/agent_work.md`
- `docs/agent/CODEX_LOG.md`
- `docs/agent/Prompt.md`
- `docs/dev/DEVELOPMENT_PLAN.md`
- `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`
- `docs/agent/SCHEDULER_O1_MIGRATION.md`
- 后续新增的分析类、迁移类、构建辅助类、协作类文档

开发版文档与发布版文档同级管理，不属于发布版内容。

## 3. 同步规则
- RTOS 本体、接口、目录、使用方法、参数、测试标准、最近一次 RTOS 验收结果的更新，同步到 `README.md` 和/或 `TESTING.md`。
- 协作规则、Prompt、分析报告、迁移说明、开发计划、构建辅助文档的更新，只同步到开发版文档体系。
- 发布版文档不需要记录、解释或引用开发版文档。
- `docs/agent/agent_work.md` 本身的更新，也不需要同步到 `README.md` 和 `TESTING.md`。

## 4. 当前归类
- `docs/agent/CODEX_LOG.md`: 开发过程日志
- `docs/agent/Prompt.md`: 协作 Prompt 与执行约束
- `docs/dev/DEVELOPMENT_PLAN.md`: 开发计划与讨论
- `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`: 串口 bench 数据分析与后续安排
- `docs/agent/SCHEDULER_O1_MIGRATION.md`: 调度器改造说明
- `scripts/bench/collect_release_bench.ps1`: release bench 多轮采集与归档辅助脚本

## 5. 维护约束
- 不再把开发版文档入口、开发版规则、分析文档、迁移说明写入 `README.md`。
- 不再把开发版过程说明写入 `TESTING.md`。
- 如果开发版文档中的结论已经成为稳定的 RTOS 对外事实，再选择性同步到发布版文档。

## 6. 本次调整
- 明确发布版文档与开发版文档同级管理。
- 明确发布版文档只保留发布版自身内容。
- 明确开发版文档不再反向影响发布版文档结构。



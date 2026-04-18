# 开发日志

## 2026-04-15 文档与目录整理
- 新增 `docs/` 分层：`docs/release/`、`docs/dev/`、`docs/agent/`、`docs/data/`
- 根目录只保留发布入口文档：`README.md`、`TESTING.md`
- 历史协作文档迁移到 `docs/agent/`
- 原始分析材料迁移到 `docs/data/`
- 修复 `docs/dev/PLAN.md` 的 UTF-8 编码问题，保留为当前多板封装计划文档

## 2026-04-15 产物目录收口
- 新增统一产物根目录 `runs/`
- 历史目录迁移如下：
  - `app_soak_runs` -> `runs/soak`
  - `app_smoke_runs` -> `runs/smoke`
  - `bench_runs` -> `runs/bench`
  - `board_builds` -> `runs/build`
  - `board_flash_runs` -> `runs/flash`
  - `regression_runs` -> `runs/regression`

## 2026-04-15 脚本目录重组
- `scripts/build/`: `build_board.ps1`、`flash_board.ps1`、`new_board_scaffold.ps1`
- `scripts/test/`: `run_app_smoke.ps1`、`run_host_tests.ps1`、`run_multiboard_regression.ps1`、`soak_default_app.ps1`、`start_24h_soak.ps1`
- `scripts/bench/`: `collect_release_bench.ps1`
- `scripts/config/`: `board_profiles.json`
- `scripts/lib/`: `board_profiles.ps1`

## 2026-04-15 脚本路径修正
- 所有主脚本改为按自身目录推导 `repoRoot`
- 板配置统一切换到 `scripts/config/board_profiles.json`
- `build/flash/smoke/soak/bench/regression` 默认输出目录全部切换到 `runs/*`
- 子脚本调用改为新路径：
  - `scripts/build/*`
  - `scripts/test/*`
  - `scripts/bench/*`

## 2026-04-15 发布面文档收口
- `README.md` 只保留项目结构、能力边界、使用入口、默认 APP 与 bench 的最小说明
- `TESTING.md` 只保留测试范围、环境基线、测试矩阵、当前通过情况、基线目录与关键结论
- 历史长过程日志归档，不再继续堆入根目录正式文档

## 2026-04-18 阶段 1 收口计划文档
- 新增 `docs/dev/PHASE1_ENGINEERING_CLOSEOUT.md`
- 将“阶段 1：工程底座收口”细化为正式开发文档
- 文档内容包括：目标、退出条件、工作包、验收要求、时间成本分析与风险缓冲

- 扩展 docs/dev/PHASE1_ENGINEERING_CLOSEOUT.md，补充背景说明、范围边界、工作包解释、阶段衔接与更细的时间成本分析，并修复该文件编码异常

## 2026-04-18 RTOS 总览文档
- 新增 `docs/dev/RTOS_TECHNICAL_STATUS.md`
- 总结当前 RTOS 的定位、结构、运行形态、核心功能、模块原理、实现方式与完成度
- 面向非项目负责人和新接手开发者提供系统级技术全景说明

## 2026-04-18 模块关系图与数据流图文档
- 新增 `docs/dev/RTOS_ARCHITECTURE_DIAGRAMS.md`
- 使用 Mermaid 图补充说明模块分层、启动路径、调度链路、默认 APP 数据流、bench 链路、健康监控与多板支持路径
- 用于帮助非负责人和新成员快速建立工程整体认知

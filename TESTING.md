# CortexOS 测试文档

## 1. 文档目的
- 给出可复现的测试流程、通过标准和最新结果。
- `TESTING.md` 只保留规范化测试信息，不再保存逐次开发流水日志。
- 详细迭代过程统一记录到 `CODEX_LOG.md`。

## 2. 测试范围
- 构建可用性：默认固件、bench 固件（debug/release）。
- 烧录可用性：`probe-rs download` 下载与校验。
- 串口可观测性：boot 信息、bench 指标输出。
- 基准功能完整性：各 benchmark 阶段都能输出统计并结束。

## 3. 测试环境基线
- 日期：2026-03-13
- 主机：Windows + PowerShell
- 工具链：Rust stable + cargo + probe-rs
- 目标：`thumbv7em-none-eabihf`
- 板卡：STM32F411RETx（Nucleo-F411RE）
- 串口：USART2，`115200 8N1`
- SWD 建议速率：`100`（当前链路更稳定）

## 4. 测试矩阵与通过标准
| ID | 测试项 | 命令/步骤 | 通过标准 |
|---|---|---|---|
| T01 | 默认固件构建 | `cargo build` | 构建成功，产出 `target/thumbv7em-none-eabihf/debug/CortexOS` |
| T02 | bench debug 构建 | `cargo build --features bench` | 构建成功，bench 路径可编译 |
| T03 | bench release 构建 | `cargo build --release --features bench` | 构建成功，产出 release bench 固件 |
| T04 | bench release 烧录 | `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS` | 下载+校验成功 |
| T05 | 串口 boot 验收 | 上电/复位后观察串口 | 出现 `boot ok (F411)`、向量与基础初始化信息 |
| T06 | bench 阶段完整性 | 运行 bench 固件观察串口 | 各阶段输出并出现 `bench complete` |
| T07 | 队列拆分指标 | bench 串口输出 | 同时出现 `queue_wake_latency` 与 `queue_end_to_end_latency` |

## 5. Bench 输出验收点
- 必须包含：
- `context_switch_a_to_b`
- `semaphore_give_to_taskb_wake`
- `sleep_1tick_extra`
- `tim2_irq_to_task`
- `queue_wake_latency`
- `queue_end_to_end_latency`
- `mutex_lock_unlock`
- `soft_timer_callback_to_task`
- `scheduler_scale`
- `scheduler_o1_check`
- 最终结束标志：`bench complete`

## 6. 最近一次执行结果（2026-03-13）
| ID | 结果 | 备注 |
|---|---|---|
| T01 | PASS | 构建通过 |
| T02 | PASS | 构建通过 |
| T03 | PASS | 构建通过 |
| T04 | NOT RUN | 本次未执行硬件烧录 |
| T05 | NOT RUN | 本次未执行串口观测 |
| T06 | NOT RUN | 本次未执行整轮 bench 跑测 |
| T07 | NOT RUN | 本次未执行串口指标验收 |
- 文档更新说明（2026-03-13）：本次 `Prompt.md` 重写属于 docs-only 变更，测试结果沿用本节最近一次执行结果。
- 文档更新说明（2026-03-17）：新增 `DEVELOPMENT_PLAN.md`，属于 docs-only 变更，本次未执行新的构建或硬件测试。
- 文档更新说明（2026-03-17）：补充了调度器 O(1) 改造计划说明，属于 docs-only 变更，本次未执行新的构建或硬件测试。

## 7. 已知限制与说明
- `cargo test` / `cargo check --all-targets` 在裸机 `no_std` 目标下会触发 `can't find crate for test`，不作为本项目通过标准。
- 当前存在较多 `unused` 警告，属于开发阶段可接受状态，不影响固件生成。
- 若 `probe-rs run` 会话被占用或断开，优先改用 `probe-rs download --verify` + 手动复位。

## 8. 维护规范
- 每次代码变更后，只更新：
- 测试矩阵（若新增/删除测试项）
- 最近一次执行结果
- 已知限制
- 不在 `TESTING.md` 追加流水式开发日志；流水日志写入 `CODEX_LOG.md`。

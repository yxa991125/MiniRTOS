# CortexOS 测试文档

## 1. 文档目的
- 给出可复现的测试流程、通过标准和最新结果。
- `TESTING.md` 只保留规范化测试信息。

## 2. 测试范围
- 构建可用性：默认固件、bench 固件（debug / release）
- 烧录可用性：`probe-rs download --verify`
- 串口可观测性：boot 信息、默认 APP 命令应答、bench 阶段与指标输出
- 自动采集可用性：`scripts/collect_release_bench.ps1`
- 长稳能力：默认 `release` 固件的 watchdog / health / soak 验证

## 3. 测试环境基线
- 日期：2026-03-31
- 主机：Windows + PowerShell
- 工具链：Rust stable + cargo + probe-rs
- 目标：
- `board-f411-nucleo`: `thumbv7em-none-eabihf`
- `board-f103c8-bluepill`: `thumbv7m-none-eabi`（运行时 BSP 已接入，板级验收进行中）
- 板卡：`STM32F411RETx`（Nucleo-F411RE）
- 串口：USART2，`115200 8N1`
- SWD 建议速度：`100`

## 4. 测试矩阵与通过标准
| ID | 测试项 | 命令/步骤 | 通过标准 |
|---|---|---|---|
| T01 | 默认固件构建 | `cargo build` | 构建成功 |
| T02 | bench debug 构建 | `cargo build --features bench` | 构建成功 |
| T03 | bench release 构建 | `cargo build --release --features bench` | 构建成功 |
| T04 | bench release 烧录 | `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS` | 下载与校验成功 |
| T05 | 串口 boot 验收 | 上电或复位后观察串口 | 出现 `boot ok (F411)` 与初始化信息 |
| T06 | bench 阶段完整性 | 运行 bench 固件观察串口 | 出现全部阶段输出并以 `bench complete` 结束 |
| T07 | 队列拆分指标 | bench 串口输出 | 同时出现 `queue_wake_latency` 与 `queue_end_to_end_latency` |
| T08 | timeout wheel 专项验证 | bench 串口输出 | 同时出现五项 `timeout_wheel_*` 输出 |
| T09 | release bench 连续采集 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COMx -Runs 5/10` | 生成 `run_*.log`、`summary.csv`、`baseline_summary.csv` |
| T10 | 阻塞式 mutex / PI 指标 | bench 串口输出 | 出现 `mutex_waiter_wake_latency`、`priority_inheritance_enter_latency`、`priority_inheritance_exit_latency` 与 `bench:mutex_priority_inheritance supported=1` |
| T11 | 脚本抽取 timeout / scheduler 指标 | 运行采集脚本 | 生成 timeout / scheduler 相关 CSV |
| T12 | queue / `IrqMutex` 长尾复验 | 对比两批 `summary.csv` | queue 不再稳定复现高尾，`IrqMutex` 仅保留孤立尖峰 |
| T13 | `IrqMutex` 尖峰归因输出 | 运行诊断批次 | 出现 `mutex_lock_unlock_attribution` 并生成归因 CSV |
| T14 | `queue / IRQ / semaphore` 归因输出 | 运行扩展 attribution 诊断批次 | 出现四条 attribution 输出，并生成 `latency_attribution.csv`、`latency_attribution_summary.csv` |
| T15 | `queue / IRQ` clean breakdown 输出 | 运行细分诊断批次 | 出现 `*_clean_breakdown` 输出，并生成 `clean_breakdown.csv`、`clean_breakdown_summary.csv` |
| T16 | 公共唤醒恢复路径优化回归 | 对比 `20260325_140028` 与优化后批次 | `tim2_irq_to_task` clean spike 降为 `0`，且 `queue` 指标与 clean spike 下降 |
| T17 | 队列 ISR 快速发送路径回归 | 对比 `20260325_164200` 与 `send_from_isr` 优化后批次 | `queue` clean spike 进一步下降，且不再稳定复现 |
| T18 | 诊断接口构建验证 | `cargo build` / `cargo build --features bench` / `cargo build --release --features bench` | 新增栈水位、运行统计、trace hook 接口全部成功编译并可通过 `kernel` 门面访问 |
| T19 | 默认固件 release 构建 | `cargo build --release` | 构建成功 |
| T20 | 默认 APP 串口命令验收 | 烧录 `release` 默认固件后发送 `PING/ECHO/LED/PWM/STAT` | 命令应答正确，LED/PWM 实际生效 |
| T21 | watchdog / 心跳链路验收 | 运行默认 `release` 固件，观察周期健康输出 | 有健康日志、关键任务心跳持续更新、watchdog 不误触发 |
| T22 | `24h` soak 验收 | 默认 `release` 固件长时间运行 | 无 `fault:`、无非预期复位、无任务失活、无持续增长 overflow |
| T23 | 默认 APP 短时 soak 样本 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COMx -DurationSec 60` | 生成 `session.log` / `summary.csv` / `summary.json`，命令全部通过且无 fault |
| T24 | 默认 APP 中时长 soak 样本 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COMx -DurationSec 600` | `10min` 持续命令轮询全通过，健康/喂狗稳定且无 fault、overflow、drop |
| T25 | 默认 APP 日志行级原子性回归 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COMx -DurationSec 10` | 命令应答与健康日志不再发生行级串扰，`commands_failed=0` |
| T26 | host 侧协议 / ring buffer 测试 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_host_tests.ps1` | `host_tests` 中纯逻辑用例全部通过 |
| T27 | 后台 soak 启动脚本烟雾验证 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/start_24h_soak.ps1 -Port COMx -DurationSec 5` | 生成 `job.json`，并由子进程产出 `session.log`、`summary.csv`、`summary.json` |
| T28 | 长周期 release bench 复验 | 先顺序构建/烧录 bench，再执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COMx -Runs 30 -ReadTimeoutMs 180000 -NoFlash` | `30/30` 轮完整结束于 `bench complete`，并生成完整 CSV |
| T29 | `board-f411-nucleo` 显式 app 构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile debug/release -Mode app` | 显式 `board-f411-nucleo` / `target` / `features` 的 app 构建成功 |
| T30 | `board-f411-nucleo` 显式 bench 构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench` | 显式 `board-f411-nucleo,bench` / `target` 构建成功 |
| T31 | `board-f103c8-bluepill` compile-only debug 构建（可选） | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103c8-bluepill -Profile debug -Mode app` | 在保守 `FLASH=64K` 下可能超限；不作为常规门禁 |
| T32 | `board-f103c8-bluepill` compile-only release 构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app` | 显式 `board-f103c8-bluepill` / `thumbv7m-none-eabi` 构建成功 |
| T33 | `STM32F103RCT6` 别名板构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app` | 别名路径构建成功，产物位于 `target/thumbv7m-none-eabi/release/CortexOS` |
| T34 | `STM32F103RCT6` 别名板烧录 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter` | `probe-rs` 下载与校验成功 |
| T35 | `STM32F103RCT6` 串口烟雾 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_app_smoke.ps1 -Board f103rct6-generic -Port COMx` | `PING/ECHO/STAT` 至少通过 3/3（当前轮作为接线/串口映射验收） |
| T36 | `STM32F103` UART probe 固件 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe` + `flash_board.ps1` + `serial_io_test.ps1` | 能看到 `uart probe mode ready` 或 `uart probe heartbeat`，并能收到 `rx:` 回显（当前固件会同时覆盖 `USART1/2/3`） |
| T37 | `STM32F103` UART probe LED 验证 | 烧录 `uart-probe` 后观察板载 LED | LED 周期性闪烁（用于确认固件在目标板持续运行） |
| T38 | 多板回归脚本（compile-only） | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench` | 统一完成 F411/F103 compile-only 回归并输出 `summary.csv/json` |
| T39 | 多板回归脚本 probe 预检 | `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -F103Port COMx -FlashOnSmoke true -AutoDisableFlashWhenProbeMissing` | 回归结果中出现 `probe_precheck_for_flash`，用于区分“probe 不可用”与“串口占用”等失败原因 |

## 5. Bench 输出验收点
- `context_switch_a_to_b`
- `semaphore_give_to_taskb_wake`
- `semaphore_give_to_taskb_wake_attribution`
- `sleep_1tick_extra`
- `tim2_irq_to_task`
- `tim2_irq_to_task_attribution`
- `tim2_irq_to_task_clean_breakdown`
- `queue_wake_latency`
- `queue_wake_latency_attribution`
- `queue_wake_latency_clean_breakdown`
- `queue_end_to_end_latency`
- `queue_end_to_end_latency_attribution`
- `queue_end_to_end_latency_clean_breakdown`
- `mutex_lock_unlock`
- `mutex_lock_unlock_attribution`
- `mutex_waiter_wake_latency`
- `priority_inheritance_enter_latency`
- `priority_inheritance_exit_latency`
- `soft_timer_callback_to_task`
- `timeout_wheel_*`
- `scheduler_scale`
- `scheduler_o1_check`
- `bench complete`

## 6. 最近一次执行结果
### 6.1 当前构建结果（2026-03-31）

| ID | 结果 | 备注 |
|---|---|---|
| T01 | PASS | `cargo build` 通过 |
| T02 | PASS | `cargo build --features bench` 通过 |
| T03 | PASS | `cargo build --release --features bench` 通过 |
| T18 | PASS | 诊断接口、心跳、系统健康、看门狗接入后仍可完成三组构建 |
| T19 | PASS | `cargo build --release` 通过 |
| T20 | PARTIAL | 已验证 `PING/ECHO/STAT` 串口应答和 `LED/PWM` 命令 `OK` 应答；`LED/PWM` 物理输出未目视/示波器复核 |
| T21 | PASS | 板级 10s+ 运行中健康日志持续输出，`wd=true`、`feeds` 递增、`stale=0`，未观察到 watchdog 误触发 |
| T22 | NOT RUN | `24h` soak 尚未执行 |
| T23 | PASS | `app_soak_runs/20260325_201735/` 中 `60s` 样本 `45/45` 命令通过，`fault=0`，无 overflow / cmd_drop |
| T24 | PASS | `app_soak_runs/20260325_202530/` 中 `600s` 样本 `440/440` 命令通过，`fault=0`，无 stale / overflow / cmd_drop |
| T25 | PASS | `app_soak_runs/20260325_212839/` 中 `10s` 回归 `8/8` 命令通过，已消除健康日志与命令应答串扰导致的假失败 |
| T26 | PASS | `scripts/run_host_tests.ps1` 运行 `5` 个 host 侧用例，覆盖协议解析、超长行恢复与 ring buffer 回卷 |
| T27 | PASS | `app_soak_runs/20260331_150313/` 中后台启动脚本 `5s` 烟雾样本生成了 `job.json`、`session.log`、`summary.csv`、`summary.json` |
| T28 | PASS | `bench_runs/20260331_143830/` 完成 `30` 轮 `-NoFlash` 长周期 bench 复验并生成完整 CSV |
| T29 | PASS | `board_builds/20260331_221644_f411-nucleo_debug_app/` 与 `board_builds/20260331_222147_f411-nucleo_release_app/` 完成显式 board/target/app 构建 |
| T30 | PASS | `board_builds/20260331_221710_f411-nucleo_release_bench/` 通过 `scripts/build_board.ps1` 完成显式 board/target/bench 构建 |
| T31 | PASS | `board_builds/20260331_225244_f103c8-bluepill_debug_app/` 完成 `board-f103c8-bluepill` compile-only debug 构建 |
| T32 | PASS | `board_builds/20260331_225309_f103c8-bluepill_release_app/` 完成 `board-f103c8-bluepill` compile-only release 构建 |
| T33 | PASS | `board_builds/20260402_180936_f103rct6-generic_release_app/` 完成 `f103rct6-generic` 别名路径 release 构建 |
| T34 | PASS | `board_flash_runs/20260402_181734_f103rct6-generic/` 完成 `STM32F103RC` 烧录与复位 |
| T35 | PASS | 2026-04-10 实板复测通过（用户反馈），`STM32F103RCT6` 串口 smoke 已可正常收发 |
| T36 | PASS | `board_builds/20260402_202644_f103rct6-generic_release_uart-probe/` + `board_flash_runs/20260402_202302_f103rct6-generic/` 已完成；在 `COM14` 实测可收到 `boot ok (F103)` / `uart probe heartbeat`，发送 `hello` 可回显 `rx: hello` |
| T37 | PARTIAL | `board_builds/20260402_191506_f103rct6-generic_release_uart-probe/` 与 `board_flash_runs/20260402_191516_f103rct6-generic/` 已更新；代码已扩展为 `PC13 + PA1` 双 LED 候选，待你实板目视确认 |
| T38 | PASS | `regression_runs/20260410_165750/`（含 `-IncludeBench`）与 `regression_runs/20260410_170900/`（参数 `-FlashOnSmoke true` 解析复测）均通过，统一回归脚本可用 |
| T39 | PARTIAL | `regression_runs/20260410_182054/` 已记录 `probe_precheck_for_flash=PASS`；但 `smoke_f103_app` 因 `COM14` 被占用失败（`Access to the port 'COM14' is denied`） |

### 6.2 bench 稳定硬件基线（2026-03-31）
- 当前稳定性能基线目录：`bench_runs/20260331_143830/`
- `30/30` 日志包含 `boot ok (F411)`、完整 bench 阶段与 `bench complete`
- `30/30` 日志无 `fault:`
- timeout wheel 五项验证全部 `fail=0`
- `scheduler_o1_check` 在 `30/30` 轮中均为 `likely_o1`
- 长周期结果显示 queue / semaphore / tim2 的尾延迟仍可观测，但已从当前功能收尾中剥离，转入后续性能调优议题

### 6.3 诊断与默认 APP 接入（2026-03-25，构建验证）
- 本轮新增能力：
- `kernel::register_task_heartbeat()` / `kernel::register_current_heartbeat()` / `kernel::task_heartbeat()`
- `kernel::system_health()`
- `kernel::feed_watchdog_if_healthy()`
- 启动阶段 reset reason 识别与打印
- 默认 APP 任务链：`uart_rx_task` / `app_cmd_task` / `uart_tx_task` / `health_task`
- 默认命令集：`PING` / `ECHO <text>` / `LED ON|OFF|TOGGLE` / `PWM <0-100>` / `STAT`
- 当前状态：代码已完成、默认 APP 已完成板级烟雾验证，`24h` soak 尚未执行

### 6.4 默认 APP 板级烟雾测试（2026-03-25）
- 固件：`target/thumbv7em-none-eabihf/release/CortexOS`
- 串口：`COM6 @ 115200 8N1`
- 操作与结果：
- 复位后收到：`boot ok (F411)`、`reset=software`、`app tasks created: rx=1 cmd=2 tx=3 health=4`
- `PING` -> `PONG`
- `ECHO hello` -> `hello`
- `LED ON` -> `OK`
- `LED TOGGLE` -> `OK`
- `PWM 50` -> `OK`
- `STAT` -> 返回 uptime、watchdog、UART 计数、drop 计数和 5 个任务快照
- 健康日志观察：
- `wd=true`
- `feeds` 从 `20 -> 40 -> 44` 持续增长
- `stale=0`
- `rxov=0`、`txov=0`、`cmd_drop=0`
- 结论：
- 默认 APP 的串口命令链路、健康任务、心跳和条件喂狗路径已经在板级跑通
- `LED/PWM` 的物理输出效果仍需人工或示波器复核，因此 `T20` 先记为 `PARTIAL`

### 6.5 默认 APP 短时 soak 样本（2026-03-25）
- 命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 60`
- 输出目录：`app_soak_runs/20260325_201735/`
- 结果摘要：
- `boot_seen=true`
- `task_banner_seen=true`
- `commands_sent=45`
- `commands_passed=45`
- `commands_failed=0`
- `fault_lines=0`
- `error_lines=0`
- `max_stale=0`
- `max_rx_overflow=0`
- `max_tx_overflow=0`
- `max_cmd_drop=0`
- `max_feeds=260`
- 结论：
- 当前默认 APP 已经通过 `60s` 板级短时 soak 样本，串口命令轮询、健康日志与条件喂狗路径在持续负载下正常
- `24h` soak 仍未执行，因此 `T22` 继续保持 `NOT RUN`

### 6.6 默认 APP 中时长 soak 样本（2026-03-25）
- 命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 600`
- 输出目录：`app_soak_runs/20260325_202530/`
- 结果摘要：
- `boot_seen=true`
- `task_banner_seen=true`
- `commands_sent=440`
- `commands_passed=440`
- `commands_failed=0`
- `fault_lines=0`
- `error_lines=0`
- `max_stale=0`
- `max_rx_overflow=0`
- `max_tx_overflow=0`
- `max_cmd_drop=0`
- `max_feeds=2440`
- 结论：
- 默认 APP 已经通过 `10min` 板级持续命令轮询样本，当前串口链路、心跳、健康任务和条件喂狗在中时长运行下稳定
- `24h` soak 尚未执行，因此 `T22` 继续保持 `NOT RUN`

### 6.7 默认 APP 日志行级原子性回归（2026-03-25）
- 背景：
- 原短时样本 `app_soak_runs/20260325_212619/` 中出现过健康日志与 `PONG` 串接在同一行的现象，导致脚本将一次 `PING` 误判为失败
- 修复：
- `src/log.rs` 改为行级缓冲发送，按换行符刷出，避免 `fmt::Write` 分段调用时与其他输出交错
- 回归命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 10`
- 回归目录：`app_soak_runs/20260325_212839/`
- 结果：
- `commands_sent=8`
- `commands_passed=8`
- `commands_failed=0`
- `fault_lines=0`
- `max_stale=0`
- 结论：
- 当前默认 APP 的串口日志与命令应答已经恢复为行级原子输出，短时 soak 不再出现由串口行串扰导致的假失败

### 6.8 host 侧纯逻辑测试（2026-03-31）
- 命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_host_tests.ps1`
- 覆盖范围：
- `src/app_protocol.rs`：`PING / ECHO / LED / PWM / STAT` 解析、超长行丢弃恢复、半包与粘包处理
- `src/ipc/ringbuf_core.rs`：满/空、回卷、顺序保持
- 结果：
- `5/5` 用例通过
- 结论：
- host 侧协议解析与 ring buffer 回归已经接入工程化测试链路，`README` 中“工程化测试”待办关闭

### 6.9 后台 soak 启动脚本烟雾验证（2026-03-31）
- 命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/start_24h_soak.ps1 -Port COM6 -DurationSec 5`
- 输出目录：`app_soak_runs/20260331_150313/`
- 结果：
- 后台启动脚本成功生成 `job.json`
- 子进程成功生成 `session.log`、`summary.csv`、`summary.json`
- `summary.csv` 显示 `commands_sent=4`、`commands_failed=0`、`fault_lines=0`
- 结论：
- 后台启动方式已经可用，后续执行完整 `24h` soak 时不再依赖保持交互式 PowerShell 窗口常驻

### 6.10 长周期 release bench 复验（2026-03-31）
- 目录：`bench_runs/20260331_143830/`
- 流程：
- 先顺序执行 `cargo build --release --features bench`
- 手动烧录一次 bench 固件
- 再执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COM6 -Runs 30 -ReadTimeoutMs 180000 -NoFlash`
- 关键结果：
- `context_switch_a_to_b`：`avg_p50=441cy`
- `queue_wake_latency`：`avg_p50=694cy`，`avg_p95=747cy`
- `queue_end_to_end_latency`：`avg_p50=890cy`，`avg_p95=956cy`
- `semaphore_give_to_taskb_wake`：`avg_p50=820cy`
- `tim2_irq_to_task`：`avg_p50=752cy`
- `scheduler_o1_check`：`30/30 likely_o1`
- 结论：
- 更长周期 bench 已经完成，现阶段剩余尾延迟问题作为后续性能调优项维护，不再阻塞默认 APP 与系统功能收尾

### 6.11 F103 串口 smoke 回归（2026-04-14）
- 背景：
- `run_app_smoke.ps1` 在 F103 路径上出现“日志已看到应答但统计失败”的假失败，同时命令尾部阶段存在偶发阻塞
- 修复：
- `src/bsp/f103c8_bluepill.rs` / `src/bsp/f411_nucleo.rs`：修正 USART IRQ 错误位处理，避免在错误分支和 RXNE 分支重复读取 `DR`
- `src/app.rs`：默认 APP 任务优先级调整为 `cmd(1) > rx(2) > tx(3) > health(4)`，确保命令执行不被服务任务拖延
- `scripts/run_app_smoke.ps1`：串口读行统一 `TrimEnd(\"\\r\",\"\\n\")`，消除 `\r\n` 导致的匹配假失败
- 回归命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_app_smoke.ps1 -Board f103rct6-generic -Port COM16 -ReadTimeoutMs 4000 -StartupWindowMs 3000 -Flash`
- 结果：
- PASS，目录：`app_smoke_runs/20260414_145748_f103rct6-generic/`
- `commands_sent=5`、`commands_passed=5`、`commands_failed=0`

### 6.12 多板回归入口复验（2026-04-14）
- 命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -F103Port COM16 -FlashOnSmoke true`
- 结果：
- PASS，目录：`regression_runs/20260414_145802/`
- `pass=4`、`fail=0`、`skip=2`
- 说明：
- 本次验证覆盖 `f411` 双 profile compile-only + `f103` release build + `f103` 在线 smoke（含自动烧录）

### 6.13 多板回归脚本泛化回归（2026-04-14）
- 变更点：
- 板配置改为统一读取 `scripts/board_profiles.json`
- 新增 `scripts/lib/board_profiles.ps1` 供 `build/flash/smoke/regression` 复用
- `flash_board.ps1`、`run_app_smoke.ps1` 增加 `-Probe`，支持多 ST-Link 非交互定向
- `run_multiboard_regression.ps1` 增加泛化参数：
- `-BuildMatrix`
- `-SmokeBoardPorts`
- `-SmokeBoardProbes`
- 回归样本：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench`
- PASS：`regression_runs/20260414_155345/`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SmokeBoardPorts "f103rct6-generic:COM17" -SmokeBoardProbes "f103rct6-generic:0483:3748" -FlashOnSmoke true`
- PASS：`regression_runs/20260414_155252/`
- 说明：
- 双 probe 同时在线且未指定 `--probe` 时，`probe-rs` 会进入交互选择；当前脚本已支持通过 `-Probe` / `-SmokeBoardProbes` 消除该问题

### 6.14 构建链路泛化回归（2026-04-14）
- 变更点：
- `build.rs` 从“硬编码 F411/F103”改为“自动识别唯一 `CARGO_FEATURE_BOARD_*` 并映射 `memory/<board>.x`”
- 回归命令：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode app`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench`
- 结果：
- PASS，目录：`regression_runs/20260414_160139/`
- 说明：
- 新增板时无需再修改 `build.rs` 的板名分支，只需补齐 feature 与 `memory/<board>.x`

## 7. 已知限制与说明
- `cargo test` / `cargo check --all-targets` 在裸机 `no_std` 目标上会触发 `can't find crate for test`，不作为本项目通过标准
- 当前仍存在较多 `unused` 警告，不影响固件生成
- 默认 APP 的 `LED/PWM` 物理输出复核和完整 `24h` soak 仍待执行
- 当前 bench 稳定基线为 `bench_runs/20260331_143830/`

## 8. 维护规范
- 每次 RTOS 代码变更后，更新：
- 测试矩阵
- 最近一次执行结果
- 已知限制

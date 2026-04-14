# CortexOS

面向低端 `Cortex-M` 开发板的最小化 `no_std` RTOS 原型工程。当前第一阶段多板封装已经落地 `board-f411-nucleo`，并已为 `board-f103c8-bluepill` 完成运行时 BSP（`USART1/USART2 + LED + PWM + watchdog`）与首轮板级 smoke 回归。默认固件提供基础控制 APP：`UART 行协议收发 + LED/PWM 控制 + 健康状态回传`。

## 运行环境
- 主机系统：Windows + PowerShell
- Rust 目标：
- `board-f411-nucleo`: `thumbv7em-none-eabihf`
- `board-f103c8-bluepill`: `thumbv7m-none-eabi`
- 工具链：`rustc/cargo` + `probe-rs`
- MCU：`STM32F411RETx`（Nucleo-F411RE）
- 调试接口：SWD，建议从 `--speed 100` 起步
- 串口：USART2（PA2/PA3），默认 `115200 8N1`
- 默认 PWM 输出：TIM1 CH1 / `PA8`
- 默认板载 LED：`PA5`

## RTOS 框架结构
- 启动层：`cortex-m-rt` + `src/arch/cortex_m/boot.S` + `src/arch/cortex_m/pendsv.S`
- 内核门面：`src/kernel.rs`
- 平台门面：`src/platform/*`
- 板级资源层：`src/bsp/*`
- 调度层：`src/task/scheduler.rs` + `src/task/tcb.rs` + `src/task/context.rs`
- 时间系统：`src/timer/systick.rs` + `src/timer/soft_timer.rs` + `src/timer/hw_timer.rs`
- 同步与 IPC：`src/sync/*` + `src/ipc/*`
- 设备与驱动：`src/device/*` + `src/driver/*`
- 应用层：`src/app.rs`（默认 UART 控制 APP） / `src/bench.rs`（基准测试固件）

当前内核/系统策略：
- ready 选择：`priority bitmap + ready queue`
- 超时唤醒：`timeout wheel`
- idle 策略：idle 任务不进入 ready queue，且不参与时间片轮转
- 同步原语：`IrqMutex`、`BlockingMutex`、`Semaphore`、`Event`
- 诊断能力：任务栈水位、任务运行 tick、trace 计数器、任务心跳、系统健康快照、reset reason、fault dump
- 生存性：默认 `release` 非 bench 固件启用 `IWDG`，由健康任务在全部关键任务心跳正常时喂狗
- 默认 APP 通信链路：USART2 RX 中断 -> RX ring buffer -> `uart_rx_task` -> 命令队列 -> `app_cmd_task` -> TX 队列 -> `uart_tx_task`
- 日志输出：行级缓冲发送，避免健康日志与命令应答在串口上发生字节级交错

## 目录
- `src/main.rs`：系统初始化、板级资源注入、任务创建、启动调度器
- `src/platform/`：UART / watchdog / 诊断 / 控制能力的板无关门面
- `src/bsp/f411_nucleo.rs`：`board-f411-nucleo` 板级资源绑定、UART/LED/PWM/watchdog 实现
- `src/bsp/f103c8_bluepill.rs`：`board-f103c8-bluepill` 运行时 BSP（已完成首轮板级验收）
- `src/kernel.rs`：内核 API 门面、系统健康与看门狗接口
- `src/task/`：任务、TCB、上下文切换、调度器、诊断数据结构
- `src/timer/`：SysTick、软定时器、TIM2/TIM3
- `src/sync/`：`IrqMutex` / `BlockingMutex` / `Semaphore` / `Event`
- `src/ipc/`：环形缓冲区、消息队列
- `src/device/uart.rs`：USART2 IRQ 驱动收发服务、队列统计
- `src/device/gpio.rs`：GPIO 输出/输入包装
- `src/device/pwm.rs`：PWM 通道包装
- `src/app.rs`：默认 UART 行协议控制 APP
- `src/bench.rs`：RTOS 基准测试固件入口
- `memory/f411-nucleo.x`：`board-f411-nucleo` 链接脚本
- `memory/f103c8-bluepill.x`：`board-f103c8-bluepill` 保守内存链接脚本（`FLASH=64K` / `RAM=20K`）
- `scripts/build_board.ps1`：按板 compile-only 构建脚本
- `scripts/build_board.md`：按板构建脚本文档
- `scripts/flash_board.ps1`：按板烧录/复位脚本
- `scripts/flash_board.md`：按板烧录脚本文档
- `scripts/board_profiles.json`：板配置清单（board/feature/target/chip/probe 协议/能力）
- `scripts/lib/board_profiles.ps1`：脚本侧板配置解析库（供 build/flash/smoke/regression 复用）
- `scripts/run_app_smoke.ps1`：默认 APP 串口烟雾测试脚本
- `scripts/run_app_smoke.md`：默认 APP 烟雾脚本文档
- `serial_io_test.ps1`：最小串口收发测试脚本（手工发送命令并读取回包）
- `scripts/collect_release_bench.ps1`：多轮 `release bench` 采集脚本
- `scripts/collect_release_bench.md`：采集脚本文档
- `scripts/run_host_tests.ps1`：host 侧纯逻辑测试脚本
- `scripts/run_host_tests.md`：host 测试脚本文档
- `scripts/run_multiboard_regression.ps1`：多板回归入口（compile-only + 可选 smoke）
- `scripts/run_multiboard_regression.md`：多板回归脚本文档
- `scripts/new_board_scaffold.ps1`：新增开发板模板生成脚本
- `scripts/new_board_scaffold.md`：新增开发板模板脚本文档
- `scripts/soak_default_app.ps1`：默认 APP 长稳/命令轮询脚本
- `scripts/soak_default_app.md`：默认 APP soak 脚本文档
- `scripts/start_24h_soak.ps1`：后台启动 `24h` soak 的脚本
- `scripts/start_24h_soak.md`：后台 soak 启动说明
- `host_tests/`：面向主机环境的协议解析与 ring buffer 回归测试
- `USER_GUIDE.md`：面向用户和应用开发者的使用说明文档
- `DEVELOPER_GUIDE.md`：面向维护人员的完整技术文档
- `BOARD_PORTING_GUIDE.md`：新增开发板的接入方法与验收流程
- `.cargo/config.toml`：仅保留通用 `rustflags` 与本地 alias，不再写死板级 target/runner

## 待办事项
- `P0` 第二块板串口链路验收（已完成，2026-04-10）：`STM32F103` 实板 `PING/ECHO/STAT` 烟雾通过
- `P0` 第二块板资源实测（已完成，2026-04-10）：`LED/PWM` 板级验证通过
- `P1` 多板回归脚本固化（已完成，2026-04-14）：`scripts/run_multiboard_regression.ps1` 已覆盖 F103 在线 smoke（含 `-FlashOnSmoke true` 路径）
- `P1` 封装后稳定性验收：默认 APP 执行 `1h soak + 多轮重复`，确认无 fault、无异常复位、关键计数无持续恶化

## 测试说明
- 默认固件（debug）：`cargo build`
- 默认固件（release）：`cargo build --release`
- bench 固件（debug）：`cargo build --features bench`
- bench 固件（release）：`cargo build --release --features bench`
- 当前推荐的 `board-f411-nucleo` 显式构建入口：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
- 当前推荐的 `board-f103c8-bluepill` compile-only 构建入口：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app`
- `debug` 构建在保守 `FLASH=64K` 配置下可能超限，作为可选检查而非常规门禁
- 当前推荐的 `STM32F103RCT6` 实板构建入口（映射到同一 `board-f103c8-bluepill` feature）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app`
- 当前推荐的串口链路排障固件构建入口（绕过 RTOS/APP，仅验证 UART）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe`
- `uart-probe` 预期行为：串口输出 `uart probe mode ready`，并周期性输出 `uart probe heartbeat`；板载 LED 周期闪烁
- `uart-probe` 当前心跳频率已提高（约每 1~3 秒一条）；即使串口在上电后才打开，也应能持续看到 `uart probe heartbeat`
- `uart-probe` 当前会同时在 `USART1(PA9/PA10)`、`USART2(PA2/PA3)`、`USART3(PB10/PB11)` 输出并接收，用于快速定位板载 USB-UART 实际路由
- 说明：`probe-rs download` 的烧录阶段 MCU 不执行应用代码，LED 闪烁只会在烧录完成并运行固件后出现
- `uart-probe` 当前为独立最小路径（不进入 RTOS 调度），用于规避启动期大内存清零导致的干扰，专门做“固件是否运行 + 串口链路 + LED 控制”排障
- 当前推荐的 `board-f411-nucleo` 烧录入口：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/flash_board.ps1 -Board f411-nucleo -Image target/thumbv7em-none-eabihf/release/CortexOS -ResetAfter`
- 当前推荐的 `STM32F103RCT6` 烧录入口：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter`
- 当前推荐的默认 APP 串口烟雾验证入口：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_app_smoke.ps1 -Board f411-nucleo -Port COMx`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_app_smoke.ps1 -Board f103rct6-generic -Port COMx`
- `run_app_smoke.ps1` 在 `-Flash` 场景下会先烧录，再打开串口并执行一次 `probe-rs reset`，减少启动 banner 丢失
- 手工串口收发验证（建议先关闭其他串口工具）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\serial_io_test.ps1 -Port COMx -BaudRate 115200`
- host 侧纯逻辑测试：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_host_tests.ps1`
- 多板回归入口（推荐日常）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench`
- 多板回归入口（带 F103 实板 smoke）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -F103Port COMx -FlashOnSmoke true`
- 多板回归入口（通用多板参数，推荐）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SmokeBoardPorts "f103rct6-generic:COM17,f411-nucleo:COM6" -SmokeBoardProbes "f103rct6-generic:0483:3748,f411-nucleo:0483:374b:SERIAL" -FlashOnSmoke true`
- 新增开发板模板（先预演，不写文件）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/new_board_scaffold.ps1 -Board myboard -Chip STM32XXXX -Target thumbv7m-none-eabi -RegisterInBoardProfiles -DryRun`
- 若当前没有可用 ST-Link，可自动降级为“不烧录，仅串口 smoke”：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -F103Port COMx -FlashOnSmoke true -AutoDisableFlashWhenProbeMissing`
- 多探针场景建议在烧录/烟雾脚本显式使用 `-Probe`，避免 `probe-rs` 进入交互式 probe 选择
- 连续 `release bench` 采集：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COMx -Runs 10`
- 已预烧录 bench 后的快速重采样：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COMx -Runs 30 -ReadTimeoutMs 180000 -NoFlash`
- 默认 APP 短时 soak：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Board f411-nucleo -Port COMx -DurationSec 60`
- 默认 APP 长时 soak：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Board f411-nucleo -Port COMx -DurationSec 86400`
- 默认 APP 后台 `24h` soak 启动：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/start_24h_soak.ps1 -Board f411-nucleo -Port COMx`
- F103 典型 soak（多探针场景）：
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Board f103rct6-generic -Port COMx -Probe VID:PID[:SERIAL] -DurationSec 60`
- 采集脚本执行流程是“烧录 -> 打开串口 -> `probe-rs reset` -> 串口采集”
- 若使用 `-NoFlash`，需要先顺序执行 `cargo build --release --features bench` 并手动烧录一次 bench 固件
- 若只采到部分日志，可追加 `-ReadTimeoutMs 180000`
- 当前稳定基线目录：`bench_runs/20260331_143830/`
- 当前默认 APP soak 样本目录：
- `app_soak_runs/20260325_201735/`（`60s`）
- `app_soak_runs/20260325_202530/`（`600s`）
- `app_soak_runs/20260331_150313/`（后台 soak 启动脚本 `5s` 烟雾样本）
- `app_soak_runs/20260414_163043/`（F103，`20s`，`flashed/reset/probe` 路径）
- `app_soak_runs/20260414_163411/`（F103，后台脚本 `5s`，`NoFlash+NoReset` 烟雾样本）

当前已完成的更长周期 bench 复验：
- 目录：`bench_runs/20260331_143830/`
- 方式：bench 固件预烧录一次后，执行 `30` 轮 `-NoFlash` 快速重采样
- 结果：`30/30` 日志完整结束于 `bench complete`，`scheduler_o1_check=likely_o1`
- 归因结果：长尾仍集中在 `resume` 公共唤醒恢复路径，属于后续性能调优议题，不再阻塞当前功能收尾

默认 APP 板级验收建议：
1. 烧录 `target/thumbv7em-none-eabihf/release/CortexOS`
2. 打开串口 `115200 8N1`
3. 发送 `PING\r\n`，应答 `PONG`
4. 发送 `ECHO hello\r\n`，应答 `hello`
5. 发送 `LED ON|OFF|TOGGLE\r\n`，观察板载 LED
6. 发送 `PWM 0`、`PWM 50`、`PWM 100`，观察 `PA8` PWM 占空比
7. 发送 `STAT\r\n`，确认返回 uptime、watchdog、心跳、栈水位、UART 计数、drop 计数

当前已完成的默认 APP 烟雾验证：
- `PING`、`ECHO hello`、`STAT` 串口应答正常
- `LED ON`、`LED TOGGLE`、`PWM 50` 已返回 `OK`
- 健康日志中 `wd=true`、`feeds` 持续增长、`stale=0`

当前已完成的默认 APP 短时 soak 样本：
- 目录：`app_soak_runs/20260325_201735/`
- 时长：`60s`
- 结果：`45/45` 条命令通过，`fault=0`
- 健康状态：`max_stale=0`、`max_rx_overflow=0`、`max_tx_overflow=0`、`max_cmd_drop=0`
- 看门狗：`wd=true`，`max_feeds=260`

当前已完成的默认 APP 中时长 soak 样本：
- 目录：`app_soak_runs/20260325_202530/`
- 时长：`600s`
- 结果：`440/440` 条命令通过，`fault=0`
- 健康状态：`max_stale=0`、`max_rx_overflow=0`、`max_tx_overflow=0`、`max_cmd_drop=0`
- 看门狗：`wd=true`，`max_feeds=2440`

bench 串口关键输出：
- `context_switch_a_to_b`
- `semaphore_give_to_taskb_wake`
- `sleep_1tick_extra`
- `tim2_irq_to_task`
- `queue_wake_latency`
- `queue_end_to_end_latency`
- `mutex_lock_unlock`
- `mutex_waiter_wake_latency`
- `priority_inheritance_enter_latency`
- `priority_inheritance_exit_latency`
- `soft_timer_callback_to_task`
- `timeout_wheel_*`
- `scheduler_scale`
- `scheduler_o1_check`
- `bench complete`

更多测试规范见 `TESTING.md`。
面向使用者的应用开发说明见 `USER_GUIDE.md`。
更完整的代码结构、模块职责和扩展说明见 `DEVELOPER_GUIDE.md`。

## 使用方法
### 1) 基本流程
1. 连接板卡并确认 SWD 可识别。
2. 执行 `cargo build --release` 或 `cargo build --release --features bench`。
3. 使用 `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify <image>` 烧录对应固件。
4. 打开串口查看输出；默认 APP 通过串口命令交互，bench 固件输出性能数据。

### 2) 参数调节方法
#### 2.1 内核参数调节方法
- SysTick 频率：修改 `src/timer/systick.rs` 中的 `TICK_HZ`
- 时间片：修改 `src/task/scheduler.rs` 中的 `DEFAULT_TIME_SLICE_TICKS`
- timeout wheel 尺寸：修改 `src/task/scheduler.rs` 中的 `TIMEOUT_WHEEL_SIZE`
- 调度容量：修改 `src/task/scheduler.rs` 中的 `MAX_TASKS`
- 任务优先级：调整 `kernel::create_task(..., priority)` 的 `priority`
- 默认 APP 任务栈大小：修改 `src/main.rs` 中的 `STACK_UART_RX_WORDS`、`STACK_APP_CMD_WORDS`、`STACK_UART_TX_WORDS`、`STACK_HEALTH_WORDS`
- 默认 APP 心跳/健康参数：修改 `src/app.rs` 中的 `HEARTBEAT_TIMEOUT_MS`、`WAIT_TIMEOUT_MS`、`HEALTH_PERIOD_MS`、`HEALTH_REPORT_MS`
- 看门狗窗口：修改 `src/main.rs` 中 `kernel::enable_watchdog(..., 1500)` 的超时时间
- 默认 APP 缓冲区尺寸：修改 `src/device/uart.rs` 中 `RX_BUF_SIZE` / `TX_BUF_SIZE`，以及 `src/app.rs` 中 `MAX_LINE_LEN` / `CMD_POOL_DEPTH`

#### 2.2 测试参数调节方法
- SWD 速度：通过 `scripts/flash_board.ps1 -Speed <value>` 调整
- 串口波特率：修改 `src/main.rs` 中 `UartConfig::default().baudrate(...)`
- 基准采样数：修改 `src/bench.rs` 中的 `BENCH_SAMPLES`
- 上下文切换预热丢弃样本：修改 `src/bench.rs` 中的 `CONTEXT_SKIP_SAMPLES`
- 连续采集轮数：运行 `scripts/collect_release_bench.ps1` 时通过 `-Runs` 调整
- 长周期 bench 是否重复烧录：运行 `scripts/collect_release_bench.ps1` 时通过 `-NoFlash` 调整
- 采集超时：运行 `scripts/collect_release_bench.ps1` 时通过 `-ReadTimeoutMs` 调整

### 3) App 规范
- 任务函数签名固定为 `fn(usize) -> !`
- 任务主体应为 `loop { ... }`
- 默认不引入堆；任务栈、命令槽、ring buffer、消息队列全部静态分配
- 中断仅做最小工作：搬运字节、置位事件、投递消息、唤醒任务
- 共享资源通过 `IrqMutex` / `BlockingMutex` / 信号量 / 队列访问
- 关键长期运行任务应注册心跳，并周期调用 `kernel::task_heartbeat()`

### 4) 默认 APP 调用方法和逻辑
默认 APP 固定创建 4 个任务：
- `uart_rx_task`：等待 USART2 RX 中断事件，从 RX ring buffer 组帧
- `app_cmd_task`：解析命令并执行 LED / PWM / 状态查询逻辑
- `uart_tx_task`：统一发送日志和命令应答
- `health_task`：周期打印精简健康信息，并在系统健康时喂狗

默认命令集：
- `PING` -> `PONG`
- `ECHO <text>` -> 返回 `<text>`
- `LED ON` / `LED OFF` / `LED TOGGLE` -> 控制 `PA5`
- `PWM <0-100>` -> 设置 `PA8` PWM 占空比
- `STAT` -> 返回系统健康摘要和任务诊断信息

输入规范：
- 行协议，ASCII 文本，以 `\r\n` 结束
- 空行忽略
- 超长行直接丢弃并累计 `line_drop`
- 命令槽分配失败或队列满会累计 `cmd_drop`

### 5) 诊断接口
- 任务注册心跳：`kernel::register_task_heartbeat(pid, timeout_ms)` / `kernel::register_current_heartbeat(timeout_ms)`
- 任务上报活性：`kernel::task_heartbeat()`
- 系统健康快照：`kernel::system_health()`，返回 `SystemHealth`
- 任务列表：`kernel::list_tasks(&mut ids)`
- 单任务快照：`kernel::task_diagnostics(pid)`，返回 `TaskDiagnostics`
- trace 计数器：`kernel::trace_counters()` / `kernel::clear_trace_counters()`
- trace hook：`kernel::set_trace_hook(Some(hook))`
- watchdog 条件喂狗：`kernel::feed_watchdog_if_healthy()`
- 串口诊断快照：`kernel::log_diagnostics()`

最小任务样例：

```rust
fn my_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(1000);
    loop {
        let _ = kernel::task_heartbeat();
        kernel::sleep_ms(10);
    }
}
```

任务注册：

```rust
kernel::create_task(my_task, 0, stack, 1);
```

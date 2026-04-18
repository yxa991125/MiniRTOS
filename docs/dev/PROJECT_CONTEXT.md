# PROJECT_CONTEXT

## 1. 文档用途
本文件用于两类场景：
- 作为二次开发参考资料，帮助新开发者快速定位模块、接口、构建方式和当前状态。
- 作为交给新 agent 的项目上下文 Prompt，让其在最少上下文下快速进入可执行状态。

本文件属于开发版文档，不承担发布版说明职责。发布版信息以 `README.md` 和 `TESTING.md` 为准。

## 2. 项目一句话概括
`CortexOS` 是一个面向低端 `Cortex-M` 开发板的 `Rust + no_std` RTOS 工程，当前正在从 `F411` 单板实现向多板可扩展框架封装，首阶段已落地 `board-f411-nucleo` 的 `platform + bsp` 分层。

## 3. 当前目标与阶段
当前处于“多板扩展封装第一阶段”的状态，主线是先收敛边界与构建体系，并推进第二块板板级验收。

已基本完成：
- RTOS 核心能力。
- 默认控制 APP。
- 基准测试链路。
- 诊断与健康监测链路。
- soak 测试脚本与 host 纯逻辑测试。

当前阶段目标：
- 将板级能力下沉到 `src/bsp/*`，核心层通过 `src/platform/*` 门面访问板级服务。
- 统一按板构建入口，避免继续依赖单板硬编码 target/runner 假设。
- 保持 `board-f411-nucleo` 的默认 APP 和 bench 可持续构建。
- 推进 `board-f103c8-bluepill` 运行时 BSP 的板级验收（当前已接入串口/LED/PWM/watchdog 基础路径）。

当前收尾事项：
- `LED/PWM` 物理输出复核。
- 长稳 soak 最终验收（当前策略为封装后执行正式长时样本）。

后续阶段方向：
- `board-f103c8-bluepill` 串口链路与资源映射板级验收（当前代码已接入，实板链路仍在确认）。
- 多板封装稳定后，再转入 Rust `safe/unsafe` 安全性重构。

## 4. 目标平台与环境
目标硬件（当前已接入）：
- `board-f411-nucleo`：`STM32F411RETx` / `Cortex-M4F`

目标硬件（当前已接入运行时 BSP，板级验收进行中）：
- `board-f103c8-bluepill`：`STM32F103C8` / `Cortex-M3`（内存固定按保守值 `64K/20K`）

主机环境：
- Windows + PowerShell
- Rust target（当前已接入）: `thumbv7em-none-eabihf`
- Rust target（已接入）: `thumbv7m-none-eabi`
- 烧录/调试: `probe-rs`
- 串口: `USART2 @ 115200 8N1`

关键约束：
- `no_std`
- `no_main`
- 以静态内存和固定容量结构为主
- 不依赖 `MPU/MMU`
- 不引入堆作为默认应用基础设施
- ISR 保持短小，复杂逻辑尽量下放任务上下文

## 5. 固件形态
### 5.1 默认 release 固件
用途：
- 开发板基础控制 APP
- 串口命令收发
- LED/PWM 控制
- 健康状态回传
- 看门狗与任务心跳验证
- soak 测试

默认命令：
- `PING`
- `ECHO <text>`
- `LED ON|OFF|TOGGLE`
- `PWM <0-100>`
- `STAT`

关键硬件映射：
- UART: `USART2`
- LED: `PA5`
- PWM: `TIM1 CH1 / PA8`

### 5.2 bench 固件
用途：
- 调度、同步、IPC、定时器的性能采样与长期基线复验

构建方式：
- `cargo build --features bench`
- `cargo build --release --features bench`

说明：
- bench 固件与默认 APP 固件分离维护。
- bench 固件不承担默认应用控制逻辑。

## 6. 代码结构总览
### 6.1 入口与顶层
- `src/main.rs`
  - 启动编排、SysTick、任务创建；具体板级初始化由 BSP 承担。
- `src/platform/`
  - 平台门面层（UART 角色、watchdog、I/O 健康快照、控制能力转发）。
- `src/bsp/`
  - 板级实现层（当前：`f411_nucleo` + `f103c8_bluepill`）。
- `src/kernel.rs`
  - 面向应用层的内核门面 API，不再直接持有 HAL watchdog 类型。
- `src/app.rs`
  - 默认控制 APP 的任务注册、命令执行、状态输出。
- `src/app_protocol.rs`
  - 默认 APP 的行协议解析与命令解析纯逻辑。
- `src/bench.rs`
  - 性能基准入口与指标实现。

### 6.2 任务与调度
- `src/task/mod.rs`
  - 任务子系统导出。
- `src/task/scheduler.rs`
  - 调度器主实现。
  - 包含 ready queue、timeout wheel、任务状态流转、心跳与诊断状态。
- `src/task/tcb.rs`
  - TCB 结构定义与相关状态字段。
- `src/task/context.rs`
  - 异常/Fault 上下文与诊断辅助。
- `src/task/diagnostics.rs`
  - 任务诊断快照与 trace/统计相关结构。

### 6.3 架构相关
- `src/arch/cortex_m/boot.S`
  - 启动汇编。
- `src/arch/cortex_m/pendsv.S`
  - `PendSV` 上下文切换汇编路径。
- `src/arch/cortex_m/interrupts.rs`
  - 中断辅助。
- `src/arch/cortex_m/cpu.rs`
  - CPU 级辅助。

### 6.4 定时器
- `src/timer/mod.rs`
  - 定时器子系统导出。
- `src/timer/systick.rs`
  - SysTick 驱动与系统 tick。
- `src/timer/soft_timer.rs`
  - 软定时器。
- `src/timer/hw_timer.rs`
  - 硬件定时器基准与辅助。

### 6.5 IPC 与同步
- `src/ipc/mod.rs`
- `src/ipc/mqueue.rs`
  - 固定容量消息队列。
- `src/ipc/ringbuf.rs`
  - IRQ-safe ring buffer 包装。
- `src/ipc/ringbuf_core.rs`
  - 可 host 测试的 ring buffer 核心逻辑。
- `src/sync/mod.rs`
- `src/sync/semaphore.rs`
- `src/sync/event.rs`
- `src/sync/mutex.rs`
  - `IrqMutex`、`BlockingMutex`、优先级继承相关实现。

### 6.6 设备与驱动
- `src/device/mod.rs`
- `src/device/uart.rs`
  - 应用侧 UART 门面包装（底层实现通过 `platform` 转发到 BSP）。
- `src/device/pwm.rs`
  - PWM 通道安全包装。
- `src/device/gpio.rs`
- `src/device/adc.rs`
- `src/driver/mod.rs`
  - 更高层驱动抽象入口。

### 6.7 内存与辅助
- `src/mem/mod.rs`
  - 固定池与静态内存相关能力。
- `src/log.rs`
  - 正常日志输出与紧急输出。

### 6.8 脚本与测试
- `scripts/build/build_board.ps1`
  - 按板 compile-only 构建入口（不烧录，不串口验证）。
- `scripts/build/flash_board.ps1`
  - 按板烧录/复位入口（不做构建）。
- `scripts/test/run_app_smoke.ps1`
  - 默认 APP 串口烟雾验证入口（正式回归不负责 flash）。
- `scripts/bench/collect_release_bench.ps1`
  - 多轮 release bench 采集与汇总。
- `scripts/test/soak_default_app.ps1`
  - 默认 APP soak 测试脚本。
- `scripts/test/start_24h_soak.ps1`
  - 后台启动长时间 soak。
- `scripts/test/run_host_tests.ps1`
  - host 纯逻辑测试入口。
- `host_tests/`
  - 协议解析和 ring buffer 核心的宿主测试工程。

## 7. 当前已实现的核心功能
### 7.1 RTOS 内核能力
- 抢占式任务调度。
- 时间片/轮转策略。
- 任务阻塞、睡眠、超时唤醒。
- ready queue。
- timeout wheel。
- Fault 诊断与启动 reset reason 打印。
- 任务诊断、运行 tick、栈水位、trace 计数。
- 任务心跳与系统健康快照。
- IWDG 条件喂狗链路。

### 7.2 IPC / 同步
- `Semaphore`
- `Event`
- `SyncMsgQueue`
- `IrqMutex`
- `BlockingMutex`
- 基础优先级继承

### 7.3 定时器
- `SysTick`
- 软定时器
- bench 用硬件定时器路径

### 7.4 默认 APP 能力
- 串口行协议接收与命令解析
- 串口统一发送任务
- LED 控制
- PWM 占空比控制
- `STAT` 健康状态回传
- 短时与中时长 soak 已板级验证通过

### 7.6 多板封装第一阶段进展
- 已新增 `platform facade` 与 `bsp/f411_nucleo`，并将 F411 板级 UART/LED/PWM/watchdog 从 `main` 抽离。
- `bsp/f103c8_bluepill` 已从 compile-only 骨架升级为运行时实现（`USART1/USART2`、`PC13 LED`、`TIM1 CH1 PWM`、`IWDG`）。
- `kernel` 已改为通过 `platform` 访问 watchdog 与 UART 健康统计。
- `app` 已改为通过 `platform::controls` 访问 LED/PWM，不再保存 F4 HAL 引脚类型。
- 已新增按板脚本：
- `build_board.ps1`
- `flash_board.ps1`
- `run_app_smoke.ps1`
- 已新增 `f103rct6-generic` 脚本别名，用于 `STM32F103RC*` 实板验证并复用 `board-f103c8-bluepill` feature。

### 7.5 bench 能力
已具备的主要指标：
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
- `scheduler_scale`
- `scheduler_o1_check`
- 各类 attribution / clean spike 归因输出

## 8. 当前验证状态
已完成：
- `cargo build`
- `cargo build --release`
- `cargo build --features bench`
- `cargo build --release --features bench`
- host 纯逻辑测试 `5/5 PASS`
- 长周期 `release bench` 复验：`30/30` 完整通过，`scheduler_o1_check=likely_o1`
- 默认 APP `60s` soak 通过
- 默认 APP `600s` soak 通过

尚未最终关闭：
- `LED/PWM` 物理输出人工复核
- 完整 `24h` soak

## 9. 二次开发的推荐入口
如果要做功能开发，优先从以下位置入手：
- 新增默认命令：`src/app_protocol.rs` + `src/app.rs`
- 新增后台任务：`src/app.rs`
- 新增对外 API：`src/kernel.rs`
- 新增设备能力：`src/device/`
- 新增同步/IPC 原语：`src/sync/` 或 `src/ipc/`
- 新增 bench 指标：`src/bench.rs`
- 新增 host 纯逻辑测试：`host_tests/tests/`

不推荐直接从这里开始：
- `src/task/scheduler.rs`
- `src/task/tcb.rs`
- `src/arch/cortex_m/pendsv.S`

原因：
- 这些是内核与上下文切换核心路径，改动风险最高，应在明确需求和验证路径后再进入。

## 10. 当前工程规则
- 应用层优先通过 `kernel` 门面访问内核能力。
- 默认坚持静态分配、固定容量结构、可预测执行路径。
- 中断只做短路径工作：搬运、置位、唤醒、计数；复杂逻辑下放到任务。
- 发布版文档只看 `README.md`、`TESTING.md`。
- 开发版文档包括：`docs/release/DEVELOPER_GUIDE.md`、`docs/release/USER_GUIDE.md`、`docs/agent/Prompt.md`、`docs/dev/DEVELOPMENT_PLAN.md`、`docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`、`docs/agent/CODEX_LOG.md` 以及本文件。
- 未执行的测试不能写成已通过。
- 如需改动上下文切换、TCB、调度器状态机，必须先确认验证方案。

## 11. 新 agent 接手时应先读什么
建议顺序：
1. `README.md`
2. `TESTING.md`
3. `docs/release/DEVELOPER_GUIDE.md`
4. `docs/release/USER_GUIDE.md`
5. 本文件 `docs/dev/PROJECT_CONTEXT.md`
6. 与当前任务直接相关的源码文件
7. 如涉及历史问题，再看 `docs/agent/CODEX_LOG.md` 或 `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`

如果任务是默认 APP 开发，重点读：
- `src/main.rs`
- `src/app.rs`
- `src/app_protocol.rs`
- `src/device/uart.rs`
- `src/device/pwm.rs`
- `src/kernel.rs`

如果任务是内核/安全重构，重点读：
- `src/kernel.rs`
- `src/task/scheduler.rs`
- `src/task/tcb.rs`
- `src/task/diagnostics.rs`
- `src/arch/cortex_m/pendsv.S`
- `docs/dev/DEVELOPMENT_PLAN.md`

## 12. 当前最重要的事实
- 当前项目已经不是“从零造 RTOS”的阶段，而是“完成发布收尾后，进入安全性研究与重构”的阶段。
- 对低端 Cortex-M 来说，核心问题不是 `MMU` 或大而全系统功能，而是：
  - 稳定性
  - 诊断能力
  - 代码边界
  - `unsafe` 收敛
  - API 安全性
- 项目下一阶段的主线已经明确：
  - 先完成收尾
  - 再做模块封装
  - 再做 TCB/共享状态边界定义
  - 最后做系统性的 `safe/unsafe` 重构

## 13. 可直接交给新 agent 的 Prompt 模板
下面这段文字可以直接作为新 agent 的起始上下文：

```text
项目名：CortexOS。
目标平台：STM32F411RETx / Cortex-M4F / Rust no_std。
当前项目已具备 RTOS 核心能力、bench 基准、诊断与看门狗链路，以及默认 UART 控制 APP。
默认 APP 支持 PING、ECHO、LED、PWM、STAT；UART 为 USART2 115200 8N1；PWM 输出为 TIM1 CH1 / PA8。
当前收尾事项只剩 LED/PWM 物理复核和完整 24h soak。
完成收尾后，项目主线切换为“无 MPU/MMU 低端 Cortex-M 上，基于 Rust safe/unsafe 边界和模块封装的 RTOS 安全性改造”。
工程规则：优先通过 kernel 门面访问内核；静态分配与固定容量优先；ISR 保持短小；未执行测试不能宣称 PASS；修改调度器、TCB、PendSV 前先给出验证方案。
接手后先读 README.md、TESTING.md、docs/release/DEVELOPER_GUIDE.md、docs/release/USER_GUIDE.md、docs/dev/PROJECT_CONTEXT.md，再进入相关源码。
```

## 14. 本文档的使用边界
- 本文档不是发布说明，不替代 `README.md`。
- 本文档不是详细源码手册，不替代 `docs/release/DEVELOPER_GUIDE.md`。
- 本文档不是测试矩阵，不替代 `TESTING.md`。
- 本文档的作用是：让开发者或新 agent 在最短时间内拿到“项目现状 + 代码地图 + 工作规则 + 下一阶段方向”。



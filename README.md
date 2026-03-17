# CortexOS

面向 `STM32F411RETx (Cortex-M4F)` 的最小化 `no_std` RTOS 原型工程。

## 运行环境
- 主机系统: Windows + PowerShell
- Rust 目标: `thumbv7em-none-eabihf`
- 工具链: `rustc/cargo` + `probe-rs`
- MCU: STM32F411RETx（Nucleo-F411RE）
- 调试接口: SWD（建议从 `--speed 100` 起步）
- 串口: USART2（PA2/PA3），默认 `115200 8N1`

## RTOS 框架结构
- 启动层: `cortex-m-rt` + `src/arch/cortex_m/boot.S` + `src/arch/cortex_m/pendsv.S`
- 内核门面: `src/kernel.rs`（任务、延时、定时器、同步调用统一入口）
- 调度层: `src/task/scheduler.rs` + `src/task/tcb.rs` + `src/task/context.rs`（ready queue + timeout wheel）
- 时间系统: `src/timer/systick.rs` + `src/timer/soft_timer.rs` + `src/timer/hw_timer.rs`
- 同步与 IPC: `src/sync/*` + `src/ipc/*`
- 设备/驱动层: `src/device/*` + `src/driver/*`
- 应用层: `src/app.rs`（默认示例） / `src/bench.rs`（基准模式）

## 目录
- `src/main.rs`: 系统初始化、任务创建、启动调度器
- `src/kernel.rs`: 内核 API 门面
- `src/task/`: 任务与调度核心
- `src/timer/`: SysTick、软定时器、TIM2/TIM3
- `src/sync/`: `IrqMutex` / `Semaphore` / `Event`
- `src/ipc/`: 环形缓冲区、消息队列
- `src/device/`: UART/GPIO/PWM/ADC/统一 timer 抽象
- `src/driver/`: motor/encoder/sensor 驱动封装
- `src/app.rs`: 默认应用（串口打印 + LED 闪烁）
- `src/bench.rs`: RTOS 基准测试固件入口
- `memory.x`: 链接脚本
- `.cargo/config.toml`: target、runner、bench alias

## 待办事项
- 超时路径验证: 增加 timeout wheel 在跨桶、跨轮、长延时场景下的回归测试
- 互斥锁增强: 增加阻塞式 mutex 与优先级继承
- 完善驱动: 扩展更多外设中断/DMA 场景
- 工程化测试: 增加 host 侧单元测试与自动化回归脚本
- 诊断能力: 增加栈水位、运行统计、trace 钩子

## 测试说明
- 默认应用构建:
- `cargo build`
- `cargo run`
- 基准固件（debug）:
- `cargo bench-dev`
- 基准固件（release）:
- `cargo bench-release-build`
- `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS`
- 串口观测:
- 打开 USART2 对应串口，`115200 8N1`，查看 boot/bench 输出
- 当前队列延迟指标:
- `queue_wake_latency`（ISR 触发唤醒到任务恢复运行）
- `queue_end_to_end_latency`（ISR 入队到任务 `recv` 完成）
- 结果记录:
- 详细测试过程与结果维护在 `TESTING.md`

## 使用方法
### 1) 基本流程
1. 连接板卡并确认 SWD 可识别。
2. 执行 `cargo run`（默认应用）或 `cargo bench-release-build`（基准构建）。
3. 使用 `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS` 烧录对应固件。
4. 打开串口查看输出。

### 2) 参数调节方法
#### 2.1 内核参数调节方法
- SysTick 频率:
- 修改 `src/timer/systick.rs` 的 `TICK_HZ`
- 同步调整 `src/main.rs` 里 `cp.SYST.set_reload(sysclk_hz / TICK_HZ - 1)` 的配置逻辑
- 时间片: 修改 `src/task/scheduler.rs` 的 `DEFAULT_TIME_SLICE_TICKS`
- 超时轮尺寸: 修改 `src/task/scheduler.rs` 的 `TIMEOUT_WHEEL_SIZE`（建议保持 2 的幂）
- 调度容量: 修改 `src/task/scheduler.rs` 的 `MAX_TASKS`（bench 与默认路径分开配置）
- 任务优先级: 调整 `kernel::create_task(..., priority)` 的 `priority`
- 任务栈大小: 调整 `src/main.rs` 的 `STACK_TASK*_WORDS`
- 软定时器容量: 修改 `src/timer/soft_timer.rs` 中定时器槽位上限常量

#### 2.2 测试参数调节方法
- SWD 速度: 修改 `.cargo/config.toml` 中 runner 的 `--speed`（不稳定时降到 `100`）
- 串口波特率: 修改 `src/main.rs` 的 `UartConfig::default().baudrate(...)`
- 基准采样数: 修改 `src/bench.rs` 的 `BENCH_SAMPLES`
- 当前默认值: `BENCH_SAMPLES = 1000`
- 基准定时器频率: 修改 `src/bench.rs` 的 `IRQ_BENCH_TIMER_HZ`
- 基准缩放测试任务组: 修改 `src/bench.rs` 的 `SCALE_CASES`
- 构建 profile:
- 调试基准用 `cargo bench-dev`
- 部署基准用 `cargo bench-release-build` + `probe-rs download ...release/CortexOS`

### 3) App 规范
- 任务函数签名固定为 `fn(usize) -> !`，任务主体应为 `loop { ... }`
- 不在中断里做复杂逻辑，中断仅做事件/唤醒/投递
- 任务延时与让出 CPU 使用 `kernel::sleep_ms()` / `kernel::yield_now()`
- 共享资源通过 `IrqMutex`/信号量/队列访问，避免裸共享可变状态
- 新应用初始化建议放在 `main` 里，在 `kernel::start()` 之前完成

### 4) App 调用方法和逻辑
- 启动主链路:
- `main -> 时钟/UART/SysTick 初始化 -> kernel::init -> create_task -> kernel::start`
- 默认应用示例:
- `task1`: 周期串口打印
- `task2`: 周期翻转 LED（PA5）
- 新增任务最小样例:

```rust
fn my_task(_arg: usize) -> ! {
    loop {
        // do work
        kernel::sleep_ms(10);
    }
}
```

- 注册任务:

```rust
kernel::create_task(my_task, 0, stack, 1);
```

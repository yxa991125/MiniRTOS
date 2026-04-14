# CortexOS 开发人员技术文档

## 1. 文档目的
本文件面向直接维护 `CortexOS` 代码的开发人员，目标是回答四个问题：
- 工程当前由哪些模块组成。
- 每个模块的职责、边界和依赖关系是什么。
- 默认固件和 bench 固件分别如何工作。
- 当需要扩展功能、定位问题或移植时，应该先看哪些代码。

当前工程的定位不是通用桌面操作系统，而是面向 `STM32F411RETx` 这类低端 `Cortex-M` 开发板的 `no_std` RTOS 原型。设计约束是：
- 不依赖 `MPU/MMU`
- 不引入堆分配
- 尽量使用静态内存和固定容量结构
- 中断只做最小工作，复杂逻辑下沉到任务上下文

## 2. 工程总览
代码入口和模块划分如下：
- `src/main.rs`：板级启动入口、外设初始化、默认固件或 bench 固件的任务创建。
- `src/kernel.rs`：面向应用的内核门面，屏蔽调度器/定时器细节。
- `src/task/`：任务控制块、调度器、异常上下文、诊断结构。
- `src/timer/`：系统节拍、软定时器、硬件定时器中断桥接。
- `src/sync/`：互斥、信号量、事件等同步原语。
- `src/ipc/`：环形缓冲区、消息队列。
- `src/device/`：UART、GPIO、PWM、ADC、定时器抽象。
- `src/driver/`：面向应用的简单电机、编码器、数字传感器包装。
- `src/app.rs`：默认 `release` 固件的 UART 控制 APP。
- `src/bench.rs`：`bench` feature 下的基准测试固件。
- `src/arch/cortex_m/`：和 `Cortex-M` 启动、PendSV、异常、CPU 指令相关的低层代码。
- `scripts/`：bench 批量采集和默认 APP soak 验证脚本。

## 3. 构建形态
工程有两条主要构建路径。

### 3.1 默认固件
- 命令：`cargo build` 或 `cargo build --release`
- 用途：运行默认 UART 控制 APP。
- 主要特性：
- 启用 USART2 中断收发
- 创建 `uart_rx_task`、`app_cmd_task`、`uart_tx_task`、`health_task`
- `release` 且非 `bench` 时默认启用 `IWDG`

### 3.2 bench 固件
- 命令：`cargo build --features bench` 或 `cargo build --release --features bench`
- 用途：输出调度、同步、超时、队列、软定时器等基准指标。
- 主要特性：
- 不走默认 APP 任务链
- 创建 `bench::task_a` / `bench::task_b`
- 打开 DWT cycle counter、TIM2 基准路径和扩展归因逻辑

## 4. 启动与执行流程
### 4.1 上电到 `main`
启动流程由三部分组成：
- `cortex-m-rt`
- `src/arch/cortex_m/boot.S`
- `src/arch/cortex_m/pendsv.S`

`main` 在 `src/main.rs` 中完成以下动作：
1. 识别 reset reason，并清除 RCC reset flags。
2. 配置时钟到 `84MHz`。
3. 初始化用于早期 boot 打印的 `USART2`。
4. 打印 `boot ok`、`reset=`、`MSP/PSP/VTOR`、`PendSV/SysTick vector`、`cpu=`。
5. 初始化 `SysTick` 为 `1kHz`，配置异常优先级。
6. 调用 `kernel::init()` 初始化调度器和软定时器。
7. 根据 feature 选择 bench 路径或默认 APP 路径。
8. 最终调用 `kernel::start()`，进入首任务。

### 4.2 SysTick、PendSV 与任务切换
任务切换的关键链路是：
- `SysTick` 异常在 `src/task/context.rs`
- `scheduler::tick_at(now)` 在 `src/task/scheduler.rs`
- 需要切换时调用 `scheduler::request_context_switch()` 挂起 `PendSV`
- `PendSV` 汇编入口在 `src/arch/cortex_m/pendsv.S`
- 汇编中调用 `__cortexos_switch_context(sp)`
- `__cortexos_switch_context` 再进入 `scheduler::context_switch(sp)`

这里的分工是：
- 汇编层负责保存/恢复寄存器上下文和 PSP/MSP 切换。
- Rust 调度器负责决定“谁是下一个任务”。
- `SysTick` 只做节拍推进、超时处理、时间片消耗和软定时器到期检查。

## 5. 内核门面 `src/kernel.rs`
`kernel.rs` 是给应用层使用的统一入口，避免应用直接依赖调度器细节。API 可分为六类。

### 5.1 任务生命周期
- `create_task(entry, arg, stack, priority)`
- `start()`
- `yield_now()`
- `sleep_ms(ms)`
- `block_current(timeout_ms)`
- `unblock(pid)`
- `delete_task(pid)`
- `exit_current()`

### 5.2 优先级与任务查询
- `set_priority(pid, new_prio)`
- `current_pid()`
- `task_priority(pid)`
- `current_priority()`
- `list_tasks(&mut ids)`

### 5.3 诊断与追踪
- `task_diagnostics(pid)`
- `trace_counters()`
- `clear_trace_counters()`
- `set_trace_hook(Some(hook))`
- `log_diagnostics()`

### 5.4 心跳与系统健康
- `register_task_heartbeat(pid, timeout_ms)`
- `register_current_heartbeat(timeout_ms)`
- `task_heartbeat()`
- `system_health()`

### 5.5 看门狗
- `enable_watchdog(watchdog, timeout_ms)`
- `feed_watchdog_if_healthy()`

### 5.6 定时器门面
- `start_timer_oneshot(...)`
- `start_timer_periodic(...)`
- `cancel_timer(handle)`
- `dispatch_timers()`

门面层本身不实现复杂策略。真正的状态机和数据结构都在 `src/task/scheduler.rs`、`src/timer/soft_timer.rs`、`src/device/*` 里。

## 6. 调度器实现 `src/task/scheduler.rs`
调度器是整个 RTOS 的核心模块，目前的关键策略如下。

### 6.1 任务表与 TCB
任务表：
- `TASKS: [Option<Tcb>; MAX_TASKS]`

`Tcb` 定义在 `src/task/tcb.rs`，核心字段包括：
- `pid`
- `sp`
- `base_priority` / `priority`
- `priority_boosts`
- `state`
- `remaining_slice`
- `wake_tick` / `has_timeout`
- `ready_prev` / `ready_next`
- `timeout_prev` / `timeout_next`
- `stack_start` / `stack_end`
- `entry` / `arg`
- `runtime_ticks`
- `heartbeat_*`

这些字段对应三个维度：
- 运行态调度信息
- timeout wheel 链接信息
- 诊断/健康信息

### 6.2 Ready Queue 与 O(1) 选择
当前就绪选择不是线性扫描任务表，而是：
- `READY_MASK: AtomicU32`
- `ReadyQueues { heads, tails, counts }`

设计含义：
- `READY_MASK` 表示哪些优先级上存在 ready task。
- 每个优先级维护一个 FIFO ready queue。
- `highest_ready_priority(mask)` 直接找到最高优先级。
- 轮转通过把当前任务重新挂回同优先级队尾来实现。

这保证了“选择下一个 ready task”的核心路径是 O(1)。

### 6.3 Timeout Wheel
超时唤醒使用：
- `TIMEOUT_WHEEL_SIZE = 256`
- `TimeoutWheel { heads, tails, last_tick }`

设计含义：
- sleeping/blocked with timeout 的任务不再靠全表扫描唤醒。
- 任务被挂到按 tick 槽位组织的 wheel 中。
- `tick_at(now)` 只处理当前推进到的槽位，而不是扫描全部任务。

这使 `tick_at` 的超时处理成本从“对所有任务扫描”下降到“只处理当前槽位任务”。

### 6.4 时间片和 idle 策略
- 默认时间片：`DEFAULT_TIME_SLICE_TICKS = 10`
- idle 任务：`pid=0`
- idle 不进入 ready queue
- idle 不参与时间片轮转

这样做的目的：
- 避免 idle 污染正常 ready queue
- 减少唤醒后无意义的 idle 进出队开销
- 降低 IRQ-to-task / queue wake 这类路径的尾延迟

### 6.5 诊断能力接入点
调度器内部还负责收集：
- `runtime_ticks`
- 栈水位（通过栈 sentinel 扫描）
- trace 计数器
- 心跳注册/更新时间
- `task_diagnostics(pid)` 快照

因此调度器不仅是“选任务”，也是任务健康状态的主要事实来源。

## 7. 异常与上下文 `src/task/context.rs`
### 7.1 SysTick
`SysTick` 处理顺序是：
1. `systick::on_tick()` 增加全局 tick
2. bench 下记录边沿时间
3. `soft_timer::on_tick(now)` 推入待执行回调
4. `scheduler::tick_at(now)` 处理时间片和超时唤醒

### 7.2 Fault 处理
实现了：
- `HardFault`
- `MemoryManagement`
- `BusFault`
- `UsageFault`

fault 处理会打印：
- `MSP/PSP`
- `CFSR/HFSR/MMFAR/BFAR`
- `R0-R3/R12/LR/PC/xPSR`

定位原则是：发生 fault 时优先保证串口可观测性，而不是尝试恢复系统。

## 8. 定时器系统
### 8.1 系统节拍 `src/timer/systick.rs`
这是最基础的时钟源：
- `TICK_HZ = 1000`
- `now()` 返回系统 tick
- `ms_to_ticks()` / `ticks_to_ms()` 提供单位换算
- `delay_ms()` 仅用于忙等场景

### 8.2 软定时器 `src/timer/soft_timer.rs`
软定时器使用固定数组：
- `MAX_TIMERS = 16`
- `MAX_PENDING = 16`

工作方式：
- `start_oneshot()` / `start_periodic()` 注册定时器
- `on_tick(now)` 检查到期项并把回调推入 `PENDING`
- `dispatch()` 在任务上下文中真正执行回调

注意：
- 回调执行不在中断中完成，而是在显式调用 `dispatch()` 时完成。
- 这是为了控制中断路径复杂度。

### 8.3 硬件定时器 `src/timer/hw_timer.rs`
支持 `TIM2` / `TIM3`：
- `init_tim2(...)`
- `init_tim3(...)`

它们通过 `device::timer::HalTimerHz` 包装 HAL 定时器，并在中断中只做：
- `wait()` 清除更新事件
- 调用注册回调

bench 中的 `TIM2` 延迟测试就是建立在这条路径上的。

## 9. 同步原语 `src/sync/`
### 9.1 `IrqMutex`
文件：`src/sync/mutex.rs`

特点：
- 基于 `cortex_m::interrupt::Mutex<RefCell<T>>`
- 进入临界区时全局关中断
- 适合短时间保护共享状态
- 不会导致任务阻塞切换

适用场景：
- 小型共享计数器
- 静态资源表
- 驱动内部寄存器影子状态

### 9.2 `BlockingMutex`
文件：`src/sync/mutex.rs`

特点：
- 固定长度等待队列
- 非递归
- 支持基础优先级继承
- owner 释放时唤醒优先级最高的 waiter

错误类型：
- `QueueFull`
- `Timeout`
- `NoCurrentTask`
- `NotOwner`
- `WouldDeadlock`

### 9.3 `Semaphore`
文件：`src/sync/semaphore.rs`

特点：
- 计数型信号量
- 固定等待队列
- `try_acquire()` 非阻塞
- `acquire(timeout_ms)` 支持超时
- `release()` 优先直接唤醒 waiter

### 9.4 `Event`
文件：`src/sync/event.rs`

特点：
- 手动复位事件
- `set()` 唤醒所有 waiter
- `clear()` 清除信号状态
- `wait(timeout_ms)` 支持超时

默认 APP 中 UART RX/TX 事件就是通过 `Event` 驱动的。

## 10. IPC 组件 `src/ipc/`
### 10.1 `RingBuf`
文件：`src/ipc/ringbuf.rs`

职责：
- 固定容量字节环形缓冲区
- 支持单字节 push/pop
- 支持 `push_slice()` / `pop_slice()` 批量操作
- `SyncRingBuf` 提供 IRQ-safe 包装
- `push_from_isr()` / `pop_from_isr()` 用于中断上下文

当前用途：
- UART RX ring
- UART TX ring

### 10.2 `MsgQueue`
文件：`src/ipc/mqueue.rs`

职责：
- 固定容量 `usize` 消息队列
- `send()` / `recv()`
- `SyncMsgQueue` 提供中断安全包装
- `send_from_isr()` 用于中断热路径快速投递

当前用途：
- 默认 APP 的命令槽索引队列
- bench 的队列延迟场景

## 11. 设备抽象 `src/device/`
### 11.1 UART 服务 `src/device/uart.rs`
这是默认 APP 的关键 I/O 基础设施。

核心资源：
- `RX_RING: SyncRingBuf<256>`
- `TX_RING: SyncRingBuf<1024>`
- `RX_EVENT`
- `TX_EVENT`
- 统计计数器：`RX_BYTES/TX_BYTES/RX_OVERFLOWS/TX_OVERFLOWS/RX_ERRORS`

主要接口：
- `init_usart2()`
- `wait_for_rx()` / `clear_rx_event()` / `read_byte()`
- `wait_for_tx()` / `clear_tx_event()` / `enqueue_tx_bytes()` / `drain_tx()`
- `stats()`
- `log_bytes()` / `raw_write_bytes()`

中断职责：
- `USART2` 中断只负责从 `DR` 读字节、写入 `RX_RING`、更新统计并触发 `RX_EVENT`

任务职责：
- 真正的串口发送在 `uart_tx_task` 中通过 `drain_tx()` 完成
- 普通日志通过 `log::with_logger()` 走 TX ring
- 启动早期和 fault 紧急日志走 `raw_write_bytes()` 直写

### 11.2 GPIO / PWM / ADC
- `src/device/gpio.rs`：统一封装输出/输入 GPIO
- `src/device/pwm.rs`：统一封装 PWM 通道占空比控制
- `src/device/adc.rs`：`ADC1` 初始化与非阻塞/阻塞读取

### 11.3 定时器抽象
文件：`src/device/timer.rs`

提供：
- `TimerDevice` trait
- `HardwareTimer` trait
- `SystemTimer`
- `HalTimerHz<TIM>`

它的作用是把 HAL 定时器包装成更适合 RTOS 使用的统一接口。

## 12. 内存模块 `src/mem/`
### 12.1 `layout.rs`
给出当前芯片的静态内存布局描述：
- Flash 起始/大小
- RAM 起始/大小
- `__STACK_START`

### 12.2 `static_pool.rs`
实现了一个固定块静态内存池：
- `StaticPool<BLOCK_SIZE, BLOCK_COUNT>`
- `alloc()` / `alloc_for<T>()` / `free_ptr()`

当前默认 APP 和核心调度尚未依赖该池；它更像后续扩展更复杂对象缓存时的预留能力。

## 13. 驱动模块 `src/driver/`
这些驱动是面向应用层的薄包装：
- `motor.rs`：PWM + 方向脚的电机驱动包装
- `encoder.rs`：原子计数型编码器累计器
- `sensor.rs`：数字输入型传感器包装

当前默认 APP 主要直接使用 GPIO/PWM 设备层，没有深度依赖这些高层驱动。

## 14. 默认 APP `src/app.rs`
默认固件的实际业务逻辑都在这里。

### 14.1 静态资源
- `LED`：板载 LED 封装
- `PWM`：PA8 PWM 通道封装
- `CMD_POOL`：命令槽池
- `CMD_QUEUE`：命令槽索引队列
- `CMD_COUNT`：命令计数信号量
- `LINE_DROPS` / `CMD_DROPS`：资源溢出统计

### 14.2 任务划分
#### `uart_rx_task`
职责：
- 等待 `USART2` RX 事件
- 从 RX ring 取字节
- 以 `\r\n` 组帧
- 超长行直接丢弃并增加 `line_drop`
- 成功组帧后把命令复制到命令池，并把槽位索引发到 `CMD_QUEUE`

#### `app_cmd_task`
职责：
- 从 `CMD_COUNT` 获取命令数量
- 从 `CMD_QUEUE` 拉取命令槽索引
- 执行命令解析与业务逻辑
- 输出 `PONG` / `OK` / `ERR` / `STAT`

#### `uart_tx_task`
职责：
- 等待 TX 事件
- 调用 `uart::drain_tx()` 真正把队列内容送到 USART2

#### `health_task`
职责：
- 周期性 `task_heartbeat()`
- 周期调用 `feed_watchdog_if_healthy()`
- 定时输出 `health:` 诊断摘要
- 一旦发现 stale task 或 stack warning，调用 `kernel::log_diagnostics()`

### 14.3 命令集
- `PING` -> `PONG`
- `ECHO <text>` -> 原样回显
- `LED ON|OFF|TOGGLE` -> 控制 `PA5`
- `PWM <0-100>` -> 调节 `PA8` 占空比
- `STAT` -> 输出系统健康状态和任务快照

### 14.4 输出路径
默认 APP 的输出有两类：
- 应答输出：命令解析后通过 `app_log_line()` 进入 UART TX ring
- 诊断输出：健康日志和 `STAT` 同样经由 logger 和 TX ring 输出

为了避免之前出现的“健康日志和命令应答串到同一行”的问题，当前 `src/log.rs` 已改为行级缓冲发送。

## 15. bench 固件 `src/bench.rs`
bench 固件不是示例代码，而是当前工程评估内核行为的主要工具。

### 15.1 bench 启动
`bench::init(...)` 负责：
- 使能 DWT cycle counter
- 初始化硬件计时依赖（例如 TIM2）
- 记录 CPU 频率

### 15.2 bench 主流程
`task_a()` 里顺序运行多个子场景：
- `run_context_bench()`
- `run_semaphore_bench()`
- `run_sleep_bench()`
- `run_irq_bench()`
- `run_queue_bench()`
- `run_mutex_bench()`
- `run_timer_callback_bench()`
- `run_timeout_validation_bench()`
- `run_scaling_bench()`

### 15.3 bench 覆盖的指标
当前 bench 覆盖：
- 上下文切换
- 信号量唤醒
- 睡眠额外延迟
- IRQ-to-task
- queue wake / queue end-to-end
- `IrqMutex` / `BlockingMutex` / PI 路径
- 软定时器回调
- timeout wheel 正确性
- scheduler scaling / O(1) 验证
- 多类 attribution / clean breakdown 归因输出

### 15.4 bench 相关脚本
- `scripts/collect_release_bench.ps1`
- `scripts/collect_release_bench.md`

它们用于：
- 多轮烧录+采集
- 生成 `summary.csv`、`baseline_summary.csv`
- 生成 timeout / scheduler / attribution / clean breakdown 汇总 CSV

## 16. 默认 APP soak 验证脚本
- `scripts/soak_default_app.ps1`
- `scripts/soak_default_app.md`

用途：
- 自动烧录默认 `release` 固件
- 自动串口轮询 `PING/ECHO/LED/PWM/STAT`
- 统计 `fault`、overflow、drop、watchdog、health 指标
- 生成 `session.log`、`summary.csv`、`summary.json`

当前脚本会实时写入 `session.log`，用于长时间 soak 中途观察。

## 17. 扩展开发建议
### 17.1 新增任务
推荐步骤：
1. 在 `src/app.rs` 或对应业务模块中定义 `fn(usize) -> !` 任务入口。
2. 在 `src/main.rs` 中准备静态栈。
3. 用 `kernel::create_task()` 创建任务。
4. 对长期运行任务调用 `register_task_heartbeat()`。
5. 如需阻塞等待，优先使用 `Event` / `Semaphore` / `MsgQueue`，不要在任务中忙等。

### 17.2 新增默认 APP 命令
推荐步骤：
1. 在 `handle_command()` 中增加命令分支。
2. 复杂命令拆成独立 `handle_xxx_command()`。
3. 若需要共享外设状态，放到 `IrqMutex` 或 `BlockingMutex` 保护下。
4. 若命令输出较大，优先按多行输出，不要生成超长单行。

### 17.3 新增驱动
推荐步骤：
1. 优先先写 `src/device/*` 的底层设备包装。
2. 若驱动具备业务语义，再在 `src/driver/*` 做面向应用的高层封装。
3. 中断只做搬运和唤醒，避免在中断里做复杂协议状态机。

### 17.4 新增基准项
推荐步骤：
1. 在 `src/bench.rs` 内新增 `run_xxx_bench()`。
2. 把指标接入 `task_a()` 的主流程。
3. 如需 CSV 聚合，补 `scripts/collect_release_bench.ps1` 的正则抽取。
4. 同步更新 `README.md` 与 `TESTING.md` 中的指标说明。

## 18. 关键配置项速查
常用配置点如下：
- `src/task/scheduler.rs`
- `MAX_TASKS`
- `DEFAULT_TIME_SLICE_TICKS`
- `TIMEOUT_WHEEL_SIZE`
- `src/timer/systick.rs`
- `TICK_HZ`
- `src/main.rs`
- `STACK_UART_RX_WORDS`
- `STACK_APP_CMD_WORDS`
- `STACK_UART_TX_WORDS`
- `STACK_HEALTH_WORDS`
- `kernel::enable_watchdog(..., 1500)`
- `src/app.rs`
- `MAX_LINE_LEN`
- `CMD_POOL_DEPTH`
- `HEARTBEAT_TIMEOUT_MS`
- `HEALTH_PERIOD_MS`
- `HEALTH_REPORT_MS`
- `src/device/uart.rs`
- `RX_BUF_SIZE`
- `TX_BUF_SIZE`
- `src/bench.rs`
- `BENCH_SAMPLES`
- `CONTEXT_SKIP_SAMPLES`

## 19. 当前限制
从开发者角度，需要明确当前仍然存在的边界：
- 默认 APP 的 `LED/PWM` 物理输出虽然已有软件应答，但还缺硬件目视/示波器复核。
- 长稳验证还没有完整跑满 `24h`。
- 当前仍然存在大量 `unused` 警告，说明模块预留能力多于当前默认固件实际启用能力。
- 没有 `MPU/MMU`、用户态隔离、文件系统和网络栈，这不是当前目标，也意味着系统定位是低端 MCU RTOS，而不是完整 OS 平台。

## 20. 阅读建议
若第一次接手本项目，建议按下面顺序读代码：
1. `README.md`
2. `src/main.rs`
3. `src/kernel.rs`
4. `src/task/scheduler.rs` + `src/task/tcb.rs`
5. `src/device/uart.rs`
6. `src/app.rs`
7. `src/task/context.rs` + `src/arch/cortex_m/pendsv.S`
8. `src/bench.rs`
9. `TESTING.md`

这个顺序可以先建立“系统如何启动和运行”的整体图，再进入调度、设备和测试细节。

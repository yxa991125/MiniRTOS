# CortexOS 用户与使用者说明文档

## 1. 文档目的
本文面向两类读者：
- 使用当前固件的开发板使用者
- 在 `CortexOS` 上编写应用逻辑的应用开发者

这里需要先说明一件事：
- 当前工程运行在低端 `Cortex-M` 平台上，没有 `MPU/MMU`
- 因此不存在严格意义上的“用户态/内核态隔离”
- 本文中的“用户态开发”，指的是“在应用层基于 `kernel` 门面编写任务、使用同步原语和设备接口开发业务功能”

如果你要维护 RTOS 内核本身，请看 `docs/release/DEVELOPER_GUIDE.md`。本文重点是“怎么用”。

## 2. 适用范围
- 目标板：`STM32F411RETx`（当前已围绕 Nucleo-F411RE 验证）
- 默认串口：`USART2 @ 115200 8N1`
- 默认板载 LED：`PA5`
- 默认 PWM 输出：`TIM1 CH1 / PA8`
- 默认固件形态：
- 普通应用固件：`cargo build` / `cargo build --release`
- bench 固件：`cargo build --features bench`

## 3. 快速开始
### 3.1 构建
默认固件：

```powershell
cargo build --release
```

bench 固件：

```powershell
cargo build --release --features bench
```

### 3.2 烧录
默认固件烧录：

```powershell
probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS
```

### 3.3 打开串口
- 串口参数：`115200 8N1`
- 复位后应看到：
- `boot ok (F411)`
- `reset=...`
- `cpu=84000000Hz`
- `app tasks created: rx=1 cmd=2 tx=3 health=4, start_first_task()`

## 4. 默认固件怎么用
当前默认固件不是空壳，而是一个基础控制 APP。它提供以下串口命令。

### 4.1 支持的命令
- `PING`
- `ECHO <text>`
- `LED ON`
- `LED OFF`
- `LED TOGGLE`
- `PWM <0-100>`
- `STAT`

### 4.2 命令示例
```text
PING
PONG

ECHO hello
hello

LED ON
OK

PWM 50
OK

STAT
STAT uptime=... wd=true ...
TASK pid=...
```

### 4.3 输入规范
- 行协议，ASCII 文本
- 以 `\r\n` 结尾
- 空行会被忽略
- 超长行会被丢弃并累计到 `line_drop`

## 5. 应用开发模型
### 5.1 “用户态开发”在本项目中的含义
当前平台没有用户态隔离，因此应用代码与 RTOS 同处一个地址空间。为了降低耦合，应用开发应遵循这条规则：
- 应用层优先只依赖 `src/kernel.rs` 暴露的门面 API
- 不要直接修改 `src/task/scheduler.rs`
- 不要直接操作任务表、ready queue、timeout wheel

换句话说：
- 业务代码写在 `src/app.rs` 或你自己的应用模块中
- 通过 `kernel::*`、`device::*`、`sync::*`、`ipc::*` 组合功能

### 5.2 一个应用由什么组成
典型应用由四部分组成：
- 任务入口函数
- 静态任务栈
- 共享资源
- 在 `main` 中的初始化和任务注册

## 6. 基本应用开发方法
### 6.1 编写任务函数
任务函数签名固定为：

```rust
fn my_task(arg: usize) -> !
```

基本约束：
- 返回类型必须是 `!`
- 主体通常是 `loop { ... }`
- 不要从任务函数返回

最小样例：

```rust
fn my_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(1000);
    loop {
        let _ = kernel::task_heartbeat();
        kernel::sleep_ms(100);
    }
}
```

### 6.2 分配静态栈
当前工程默认不引入堆，因此任务栈必须静态分配。

示例思路：

```rust
#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

static mut STACK_MY_TASK: AlignedStack<256> = AlignedStack([0; 256]);
```

在 `main` 中把它转换成切片：

```rust
let stack = unsafe {
    let ptr = core::ptr::addr_of_mut!(STACK_MY_TASK.0) as *mut u32;
    core::slice::from_raw_parts_mut(ptr, 256)
};
```

### 6.3 创建任务
通过 `kernel::create_task()` 注册：

```rust
let pid = kernel::create_task(my_task, 0, stack, 2).expect("create task failed");
```

参数含义：
- 第 1 个参数：任务入口函数
- 第 2 个参数：传给任务的 `arg`
- 第 3 个参数：静态栈
- 第 4 个参数：优先级，数字越小优先级越高

### 6.4 注册心跳
对长期运行任务，建议都注册心跳：

```rust
let _ = kernel::register_task_heartbeat(pid, 1000);
```

或者在任务自己启动后：

```rust
let _ = kernel::register_current_heartbeat(1000);
```

运行中定期上报：

```rust
let _ = kernel::task_heartbeat();
```

### 6.5 休眠与让出 CPU
- `kernel::sleep_ms(ms)`：当前任务睡眠
- `kernel::yield_now()`：主动触发一次调度

适用建议：
- 周期任务用 `sleep_ms`
- 仅在确实需要尽快切走时才用 `yield_now`

## 7. 同步与通信的基本用法
### 7.1 `Event`
适合“一次事件唤醒一个或多个任务”。

典型用途：
- UART RX 到达
- 中断通知任务处理

常用接口：
- `set()`
- `clear()`
- `wait(timeout_ms)`

### 7.2 `Semaphore`
适合计数型资源或生产者/消费者唤醒。

常用接口：
- `try_acquire()`
- `acquire(timeout_ms)`
- `release()`

### 7.3 `SyncMsgQueue`
适合在任务之间或中断到任务之间传递 `usize` 消息。

常用接口：
- `send(msg)`
- `send_from_isr(msg)`
- `recv()`

### 7.4 `IrqMutex`
适合很短的共享状态保护。

特点：
- 通过关中断进入临界区
- 不适合长时间持锁

### 7.5 `BlockingMutex`
适合需要阻塞等待的共享资源。

特点：
- 固定等待队列
- 支持基础优先级继承
- 非递归

## 8. 设备接口的基本用法
### 8.1 UART
当前默认 UART 服务在 `src/device/uart.rs`。

常用能力：
- `wait_for_rx()`：等待接收事件
- `read_byte()`：从 RX ring 读字节
- `enqueue_tx_bytes()`：排队发送数据
- `wait_for_tx()` + `drain_tx()`：在 TX 任务中真正送出数据
- `stats()`：读取 UART 计数器

适用原则：
- 中断只负责收字节和发事件
- 复杂协议解析放在任务上下文中做

### 8.2 GPIO
- `device::gpio::GpioOutput`
- `device::gpio::GpioInput`

常用能力：
- `set_high()`
- `set_low()`
- `toggle()`
- `is_high()` / `is_low()`

### 8.3 PWM
- `device::pwm::PwmChannel`

常用能力：
- `set_duty_percent(percent)`
- `set_duty(duty)`

## 9. 默认 APP 的应用开发参考
如果你要写自己的应用，当前默认 APP 是最直接的参考模板。

### 9.1 默认 APP 的任务链
- `uart_rx_task`
- `app_cmd_task`
- `uart_tx_task`
- `health_task`

对应职责：
- RX 任务负责收字节、组帧、投递命令
- CMD 任务负责解释命令并调用业务逻辑
- TX 任务负责统一串口输出
- Health 任务负责心跳、健康报告和看门狗

### 9.2 如何加一个新命令
建议直接按这个流程扩展：
1. 在 `src/app.rs` 的 `handle_command()` 里增加分支
2. 把复杂逻辑拆成 `handle_xxx_command()`
3. 如果要访问共享资源，用 `IrqMutex` 或 `BlockingMutex`
4. 输出统一走 `app_log_line(...)`

示意：

```rust
if cmd.eq_ignore_ascii_case("HELLO") {
    app_log_line(format_args!("WORLD"));
    return;
}
```

### 9.3 如何增加一个后台任务
适合场景：
- 周期采样
- 状态机驱动
- 串口转发
- 简单协议桥接

推荐做法：
1. 新建任务函数
2. 申请静态栈
3. 在 `main` 中创建任务
4. 如有必要，在 `STAT` 中增加状态输出

## 10. 用户态开发的基本模式
对于“信号接收与转发”这类应用，推荐使用下面的固定模式：

### 10.1 中断收集 + 任务处理
- 中断：只负责收数据、写 ring buffer、发事件
- 任务：读 ring buffer、解析协议、决定转发目标

不要把协议解析放进中断里。

### 10.2 任务分层
推荐至少拆成两层：
- 输入任务
- 业务处理任务

如果还需要对外输出，再拆一个：
- 输出任务

这样可以避免多个任务直接争抢同一个 UART/TX 通道。

### 10.3 固定容量设计
在本工程里，应用开发尽量坚持：
- 固定大小栈
- 固定大小 ring buffer
- 固定大小消息队列
- 固定大小命令槽

这样做的好处是：
- 内存使用可预测
- 不依赖堆
- 出问题时更容易定位 overflow 和 drop

## 11. 诊断与调试方法
### 11.1 系统健康快照
可直接调用：

```rust
let health = kernel::system_health();
```

能拿到：
- uptime
- live task 数
- heartbeat 注册数
- stale task 数
- stack warning 数
- watchdog 状态
- UART 统计

### 11.2 单任务诊断
```rust
if let Some(task) = kernel::task_diagnostics(pid) {
    // 查看 runtime、stack、heartbeat 状态
}
```

### 11.3 串口诊断快照
```rust
kernel::log_diagnostics();
```

适合在这些场景调用：
- 检测到任务 stale
- 检测到 stack warning
- 收到诊断命令

### 11.4 看门狗使用原则
默认 `release` 非 bench 固件启用 `IWDG`。
只有系统健康时才喂狗：
- 有已注册的关键任务
- 没有 stale task

因此如果关键任务卡死，系统会停止喂狗并复位。

## 12. 典型开发流程
推荐应用开发按下面的顺序进行：
1. 先在 `src/app.rs` 中实现最小命令或最小任务
2. 只依赖 `kernel` 门面，不直接改调度器
3. 先验证串口输出和状态机逻辑
4. 再接入 GPIO / PWM / 传感器等外设
5. 再加心跳和 `STAT` 诊断输出
6. 最后再做 soak 测试

## 13. 常见问题
### 13.1 为什么没有“真正用户态”
因为当前目标是低端 `Cortex-M`，默认没有 `MPU/MMU`，也没有进程隔离能力。当前工程通过接口分层而不是硬件隔离来组织应用。

### 13.2 为什么所有栈都要静态分配
因为当前工程不依赖堆，静态分配更可控，也更适合 RTOS 原型阶段的长稳分析。

### 13.3 为什么中断里不能写复杂业务逻辑
因为中断应尽可能短，否则会增加系统抖动、影响实时性，并放大排障难度。

### 13.4 为什么 `STAT` 和 `health` 很重要
因为在没有用户态隔离和复杂调试设施的低端开发板上，运行时自检和串口可观测性是最重要的排障工具。

## 14. 与其他文档的关系
- `README.md`：发布版总览、快速使用、测试入口
- `TESTING.md`：测试矩阵、通过标准、最新结果
- `docs/release/DEVELOPER_GUIDE.md`：内核与代码结构的详细技术文档
- `scripts/bench/collect_release_bench.md`：bench 采集脚本文档
- `scripts/test/soak_default_app.md`：默认 APP soak 脚本文档

如果你的目标是“我想在这个 RTOS 上开发一个控制应用”，优先看本文。
如果你的目标是“我要修改调度器、定时器或同步原语”，优先看 `docs/release/DEVELOPER_GUIDE.md`。


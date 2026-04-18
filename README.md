# CortexOS

面向无 `MPU/MMU` 的低端 `Cortex-M` 开发板的 `no_std` RTOS 原型工程。当前仓库重点是两件事：
- 将 RTOS 核心与板级资源解耦，支持多板扩展
- 提供一个可直接落板的默认 APP：`UART` 行协议收发、`LED/PWM` 控制、状态回传

## 当前支持

| Board Profile | Target | 状态 | 默认 UART | LED | PWM |
|---|---|---|---|---|---|
| `board-f411-nucleo` | `thumbv7em-none-eabihf` | 已完成 APP / bench / soak 验收 | `USART2 PA2/PA3` | `PA5` | `TIM1 CH1 / PA8` |
| `board-f103c8-bluepill` | `thumbv7m-none-eabi` | 已完成 APP / smoke / soak 验收 | `USART1 PA9/PA10` | `PC13` | `TIM1 CH1 / PA8` |
| `f103rct6-generic` | `thumbv7m-none-eabi` | `board-f103c8-bluepill` 的别名板配置 | `USART1 PA9/PA10` | `PC13` | `TIM1 CH1 / PA8` |

说明：
- `bench` 当前只承诺 `board-f411-nucleo`
- `board-f103c8-bluepill` 的链接脚本按保守值固定为 `FLASH=64K`、`RAM=20K`

## 仓库结构

### 根目录
- `src/`: RTOS 核心、平台门面、BSP、默认 APP、bench
- `memory/`: 各板链接脚本
- `host_tests/`: 宿主侧协议 / ring buffer 测试
- `scripts/`: 构建、烧录、测试、bench、板配置脚本
- `runs/`: smoke / soak / build / flash / bench / regression 历史产物
- `docs/release/`: 面向使用者与维护者的正式说明文档
- `README.md`: 发布入口文档
- `TESTING.md`: 测试规范与当前通过情况
- `DEV_LOG.md`: 开发结构调整日志
- `TEST_LOG.md`: 测试与验收日志

### 代码层次
- `src/arch/cortex_m/`: Cortex-M 启动与上下文切换汇编
- `src/kernel.rs`: 内核门面、系统健康、看门狗协调
- `src/platform/`: UART / watchdog / 诊断 / 控制能力门面
- `src/bsp/`: 各板 BSP 与 HAL/PAC 绑定
- `src/task/`: TCB、调度器、上下文、任务诊断
- `src/timer/`: SysTick、软定时器、硬件定时器
- `src/sync/`: `IrqMutex`、`BlockingMutex`、`Semaphore`、`Event`
- `src/ipc/`: ring buffer、消息队列
- `src/device/`: 板无关设备封装
- `src/app.rs`: 默认 UART 控制 APP
- `src/bench.rs`: RTOS 性能基准固件

## 系统能力
- 调度：`priority bitmap + ready queue`
- 超时：`timeout wheel`
- idle：不入 ready queue，不参与时间片
- 健康监控：任务心跳、栈水位、任务运行 tick、trace 计数器、reset reason
- 生存性：默认 `release` 非 bench 固件启用 `IWDG`
- APP 通信链路：RX IRQ -> ring buffer -> `uart_rx_task` -> 命令队列 -> `app_cmd_task` -> TX 队列 -> `uart_tx_task`

## Quickstart

### 1. 基本构建
```powershell
cargo build
cargo build --release
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_host_tests.ps1
```

### 2. 按板构建
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app
```

### 3. 按板烧录
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f411-nucleo -Image target/thumbv7em-none-eabihf/release/CortexOS -ResetAfter
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter
```

### 4. 默认 APP 烟雾测试
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_app_smoke.ps1 -Board f411-nucleo -Port COM6
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_app_smoke.ps1 -Board f103rct6-generic -Port COM18 -Probe 0483:3748 -Flash
```

### 5. 默认 APP soak
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f411-nucleo -Port COM6 -NoFlash -NoReset -DurationSec 3600
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f103rct6-generic -Port COM18 -Probe 0483:3748 -NoFlash -DurationSec 3600
```

### 6. bench 采集
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 10
```

### 7. 多板回归
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SmokeBoardPorts "f103rct6-generic:COM18" -SmokeBoardProbes "f103rct6-generic:0483:3748" -FlashOnSmoke true
```

## 默认 APP 使用方法

串口协议为 ASCII 行协议，以 `\r\n` 结束。

支持命令：
- `PING` -> `PONG`
- `ECHO <text>` -> 返回 `<text>`
- `LED ON|OFF|TOGGLE` -> 控制板级 LED
- `PWM <0-100>` -> 设置 PWM 占空比
- `STAT` -> 返回健康快照摘要

开发约束：
- 不使用堆，任务、队列、缓冲区均为静态分配
- 上层 APP 不直接操作 HAL 类型，统一经 `platform` / `device` / `bsp` 边界访问
- 新板接入优先按 `docs/release/BOARD_PORTING_GUIDE.md` 扩展 `board profile`

## bench 使用说明
- bench 固件入口在 `src/bench.rs`
- bench 当前只对 `board-f411-nucleo` 提供完整保证
- 主要指标包括：上下文切换、信号量唤醒、队列唤醒、IRQ 到任务、软定时器、timeout wheel、`scheduler_scale`、`scheduler_o1_check`
- 采集结果默认输出到 `runs/bench/<timestamp>/`

## 补充文档
- `docs/release/USER_GUIDE.md`: 面向使用者和应用开发者
- `docs/release/DEVELOPER_GUIDE.md`: 面向维护者与二次开发者
- `docs/release/BOARD_PORTING_GUIDE.md`: 新增开发板方法
- `TESTING.md`: 测试范围、基线、矩阵、当前通过状态

## 当前未关闭事项
- `F103 UART` 长时抗干扰继续观察：当前已做“检测到硬件接收错误后整行丢弃”的快速修复，需继续累积更长样本
- 新板扩展继续按 `board profile + BSP + script config` 路径推进

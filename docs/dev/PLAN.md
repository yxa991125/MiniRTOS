# CortexOS 多板扩展封装计划（正式执行版）

## 摘要
本轮目标是把当前 RTOS 从 `F411` 单板实现封装成可扩展到多块无 `MPU/MMU` 的低端 `Cortex-M` 开发板的通用框架，而不是为某个单一业务场景定制 RTOS。

首批支持范围固定为：
- `board-f411-nucleo`
- `board-f103c8-bluepill`

首批架构范围固定为：
- `Cortex-M3/M4`
- `M0/M0+` 后置

本轮不做：
- Rust `safe/unsafe` 安全改造
- 非 `Cortex-M` 支持
- `F103` 的完整 bench 体系
- 面向单一业务任务的专用 RTOS 改造

## Board Profile
### `board-f411-nucleo`
- target: `thumbv7em-none-eabihf`
- chip: `STM32F411RE`
- boot/runtime UART: `USART2 PA2/PA3`
- LED: `PA5`
- PWM: `TIM1 CH1 / PA8`
- watchdog: `IWDG`

### `board-f103c8-bluepill`
- target: `thumbv7m-none-eabi`
- chip: `STM32F103C8`
- boot/runtime UART: `USART1 PA9/PA10`
- LED: `PC13`
- PWM: `TIM1 CH1 / PA8`
- watchdog: `IWDG`
- memory 固定保守值：`FLASH=64K`、`RAM=20K`

规则：
- 新增板时必须先定义完整 `board profile`
- 每项资源要么明确映射，要么显式标记为 `None`

## 分层与边界
固定层次：
- `arch/cortex_m`: 纯架构层
- `kernel/task/timer/sync/ipc/log`: RTOS 核心层
- `platform`: 平台服务门面
- `device`: 面向应用的板无关设备 API
- `bsp/<board>`: 板级资源、HAL/PAC 绑定、IRQ 绑定、profile 能力声明

固定依赖方向：
- `kernel` / `device` / `log` -> `platform`
- `platform` -> `bsp` 注册实现
- `bsp` -> HAL/PAC
- `arch` 不持有板级 IRQ 枚举

## UART 与平台门面
### `device::uart`
不是单例 UART 服务，至少支持两类逻辑角色：
- `BootConsole`
- `AppUart`

### `kernel`
只依赖平台服务门面，不直接依赖：
- `stm32f4xx-hal::*`
- 某个固定 UART 实现
- 某个固定 watchdog 类型

## 构建系统
采用：`单 crate + board feature + PowerShell 脚本`

固定 feature：
- `board-f411-nucleo`
- `board-f103c8-bluepill`
- `bench`

默认 feature：
- `board-f411-nucleo`

固定脚本职责：
- `scripts/build/build_board.ps1`: compile-only 构建
- `scripts/build/flash_board.ps1`: 烧录与复位
- `scripts/test/run_app_smoke.ps1`: 烧录后串口烟雾验证

固定映射：
- `board-f411-nucleo` -> `thumbv7em-none-eabihf`
- `board-f103c8-bluepill` -> `thumbv7m-none-eabi`

## bench 范围
本轮只保证：
- `board-f411-nucleo` 的默认 APP 与 bench

本轮不要求：
- `board-f103c8-bluepill` 的完整 bench 对齐

## 验收要求
### 封装中快速回归
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app`
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode app`
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
- `scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app`
- `scripts/test/run_host_tests.ps1`
- `scripts/test/run_multiboard_regression.ps1 -SkipSmoke`

### 封装后正式验收
#### `board-f411-nucleo`
- 默认 APP 命令烟雾
- `1h soak`
- release bench 回归

#### `board-f103c8-bluepill`
- 构建成功
- 启动成功
- `PING`
- `ECHO hello`
- `STAT`
- `PWM 50` 或 `LED TOGGLE`
- 至少一次短时稳定样本

## 当前阶段标准
- 正式长稳标准：封装后 `1h soak + 多轮重复`
- 不再以连续 `24h soak` 作为当前阶段阻塞条件

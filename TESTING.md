# CortexOS 测试文档

## 1. 文档目的
- 固化测试范围、测试环境、通过标准与当前状态
- `TESTING.md` 不记录排障过程日志，仅保留规范化测试结果

## 2. 测试范围
- 构建可用性：默认 APP / bench / host tests
- 板级可用性：构建、烧录、串口 smoke、默认 APP 命令正确性
- 稳定性：默认 APP soak、watchdog、健康状态
- 性能：`board-f411-nucleo` 的 release bench 采集与基线对比
- 多板回归：按脚本统一执行 compile-only 与可选 smoke

## 3. 环境基线

| 项目 | 基线 |
|---|---|
| 日期 | `2026-04-15` |
| 主机 | Windows + PowerShell |
| Rust | stable toolchain |
| 下载工具 | `probe-rs` |
| F411 target | `thumbv7em-none-eabihf` |
| F103 target | `thumbv7m-none-eabi` |
| F411 板卡 | `STM32F411RETx` / Nucleo-F411RE |
| F103 板卡 | `STM32F103RCT6` 实板，映射到 `f103rct6-generic` |
| SWD 建议速度 | `100` |

## 4. 测试矩阵与通过标准

| 类别 | 测试项 | 入口 | 通过标准 |
|---|---|---|---|
| Build | 默认构建 | `cargo build` / `cargo build --release` | 构建成功 |
| Host | 宿主逻辑测试 | `scripts/test/run_host_tests.ps1` | 协议解析、超长行恢复、ring buffer 用例全部通过 |
| F411 APP | 板级构建 + smoke | `scripts/build/build_board.ps1` + `scripts/test/run_app_smoke.ps1` | `PING/ECHO/LED/PWM/STAT` 正常 |
| F103 APP | 板级构建 + smoke | `scripts/build/build_board.ps1` + `scripts/test/run_app_smoke.ps1` | `PING/ECHO/LED/PWM/STAT` 正常 |
| Soak | 默认 APP 长稳 | `scripts/test/soak_default_app.ps1` | `fault=0`、异常复位=0、`stale=0`、overflow / `cmd_drop` 不恶化 |
| Bench | release bench | `scripts/bench/collect_release_bench.ps1` | 采集完成并输出完整 CSV |
| Regression | 多板回归 | `scripts/test/run_multiboard_regression.ps1` | compile-only 或 smoke 步骤按配置通过 |

## 5. 当前通过情况

### 5.1 构建与脚本

| 项目 | 状态 | 说明 |
|---|---|---|
| 默认 APP `debug`/`release` 构建 | PASS | 常规 `cargo build` / `cargo build --release` 可用 |
| `host_tests` | PASS | 最近一次执行为 `5` 个用例全部通过 |
| `board-f411-nucleo` 显式构建 | PASS | APP / bench 构建脚本可用 |
| `board-f103c8-bluepill` 显式构建 | PASS | release compile-only 可用 |
| 多板回归脚本 | PASS | compile-only 路径已通过 |
| 新板模板脚本 | PASS | `new_board_scaffold.ps1` 已可按板配置生成模板 |

### 5.2 板级 APP

| Board | 状态 | 说明 |
|---|---|---|
| `f411-nucleo` smoke | PASS | `runs/smoke/20260415_141156_f411-nucleo/` |
| `f411-nucleo` `1h soak` | PASS | `runs/soak/20260415_2411_f411_1h/`，`8088/8088` 命令通过，`fault=0` |
| `f103rct6-generic` smoke | PASS | `runs/smoke/20260415_185129_f103rct6-generic/` |
| `f103rct6-generic` `1h soak` 轮次 1 | PASS | `runs/soak/20260415_131037/` |
| `f103rct6-generic` `1h soak` 轮次 2 | PASS | `runs/soak/20260415_141355/` |
| `f103rct6-generic` UART 抗干扰快速修复回归 | PASS | `runs/soak/20260415_f103_uart_fix_180s/`，`426/426` 命令通过 |

## 6. 当前基线目录

| 类型 | 基线目录 | 用途 |
|---|---|---|
| F411 release bench | `runs/bench/20260331_143830/` | 当前稳定性能基线 |
| F103 smoke | `runs/smoke/20260415_185129_f103rct6-generic/` | F103 默认 APP 最新烟雾基线 |
| F103 `1h soak` | `runs/soak/20260415_141355/` | F103 长稳基线 |
| F411 `1h soak` | `runs/soak/20260415_2411_f411_1h/` | F411 长稳基线 |
| 多板回归 | `runs/regression/20260414_161915/` | compile-only 回归基线 |

## 7. 最近一轮关键结论
- 多板封装后的主线验收已完成，`F411` 与 `F103` 默认 APP 均已跑通 smoke 和 soak
- `F411` 已完成 `1h soak`，结果稳定
- `F103` 已完成双轮 `1h soak`，系统级稳定性通过
- `F103 UART` 仍存在低概率物理链路误码风险，已上线“检测到 UART 硬件错误后整行丢弃”的快速修复，短时回归已恢复到 `426/426`
- `bench` 当前继续以 `board-f411-nucleo` 为唯一承诺板型

## 8. 常用测试入口
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_host_tests.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_app_smoke.ps1 -Board f103rct6-generic -Port COM18 -Probe 0483:3748 -Flash
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f411-nucleo -Port COM6 -NoFlash -NoReset -DurationSec 3600
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 10
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke
```

## 9. 未关闭测试项
- `F103 UART` 长时抗干扰继续复验
- 若硬件在线条件允许，可补更长时长 soak，但当前阶段不作为主线阻塞项

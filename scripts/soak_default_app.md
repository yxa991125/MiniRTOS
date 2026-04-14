# `soak_default_app.ps1` 使用说明

## 1. 目的
- 用于默认 `release` 固件的板级长稳验证。
- 自动完成：
- 烧录默认固件
- 复位目标板
- 打开串口并采集启动日志
- 周期发送 `PING / ECHO / LED / PWM / STAT`
- 汇总命令通过率、fault、overflow、drop、watchdog/health 指标

## 2. 适用对象
- 目标板：`STM32F411RETx`
- 固件：`target/thumbv7em-none-eabihf/release/CortexOS`
- 串口：默认 `USART2 @ 115200 8N1`

## 3. 基本命令
短时样本：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 60
```

完整 `24h` soak：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 86400
```

若固件已经手动烧录，可跳过脚本内烧录：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/soak_default_app.ps1 -Port COM6 -DurationSec 60 -NoFlash
```

## 4. 参数说明
- `-Port`：必填，串口号，例如 `COM6`
- `-DurationSec`：运行时长，默认 `60`
- `-Baud`：串口波特率，默认 `115200`
- `-Binary`：烧录镜像路径，默认 `target/thumbv7em-none-eabihf/release/CortexOS`
- `-Chip`：`probe-rs` 目标芯片名，默认 `STM32F411RETx`
- `-Speed`：SWD 速度，默认 `100`
- `-ResetDelayMs`：复位后等待启动日志的延时，默认 `200`
- `-ReadSliceMs`：每条命令后的串口读取窗口，默认 `1200`
- `-PauseMs`：两条命令之间的停顿，默认 `100`
- `-RunId`：可选，指定输出目录名，主要供后台启动脚本复用
- `-NoFlash`：跳过脚本内 `probe-rs download`

若希望避免误关控制台窗口中断 `24h` soak，优先使用：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/start_24h_soak.ps1 -Port COM6
```

## 5. 固定命令轮询
脚本按顺序循环发送：
- `PING`
- `ECHO soak`
- `LED TOGGLE`
- `PWM 50`
- `STAT`

判定规则：
- `PING` 期望 `PONG`
- `ECHO soak` 期望 `soak`
- `LED TOGGLE` 期望 `OK`
- `PWM 50` 期望 `OK`
- `STAT` 期望以 `STAT ` 开头的状态行

## 6. 输出文件
脚本会在 `app_soak_runs/<timestamp>/` 下生成：
- `session.log`：完整串口日志
- `summary.csv`：单次汇总
- `summary.json`：同一份汇总的 JSON 版本

说明：
- 当前脚本会实时写入 `session.log`，便于长时间 soak 过程中直接观察串口日志。

关键字段：
- `boot_seen`
- `task_banner_seen`
- `commands_sent`
- `commands_passed`
- `commands_failed`
- `health_lines`
- `max_feeds`
- `max_stale`
- `max_rx_overflow`
- `max_tx_overflow`
- `max_cmd_drop`
- `fault_lines`
- `error_lines`

## 7. 通过标准
短时样本建议至少满足：
- `boot_seen=true`
- `task_banner_seen=true`
- `commands_failed=0`
- `fault_lines=0`
- `max_stale=0`
- `max_rx_overflow=0`
- `max_tx_overflow=0`
- `max_cmd_drop=0`

`24h` soak 建议继续满足：
- 无 `fault:`
- 无非预期复位
- 健康日志持续输出
- `feeds` 持续增长
- `stale=0`
- overflow / drop 不持续增长

## 8. 当前样本
- `app_soak_runs/20260325_201735/`
- 时长：`60s`
- 结果：`commands_passed=45`，`commands_failed=0`，`fault_lines=0`
- `app_soak_runs/20260325_202530/`
- 时长：`600s`
- 结果：`commands_passed=440`，`commands_failed=0`，`fault_lines=0`
- `app_soak_runs/20260325_212839/`
- 时长：`10s`
- 结果：`commands_passed=8`，`commands_failed=0`，用于回归验证日志行级原子性
- `app_soak_runs/20260331_150313/`
- 时长：`5s`
- 结果：后台启动脚本烟雾验证通过，生成了 `job.json`、`session.log`、`summary.csv`、`summary.json`

## 9. 常见问题
- 没有启动日志：
- 检查 `COM` 口是否正确
- 检查串口助手是否占用端口
- 增大 `-ResetDelayMs`
- `probe-rs` 打不开探针：
- 关闭其他调试器/串口工具
- 重新插拔板卡
- 手动执行 `probe-rs list`
- 长时间运行中断：
- 优先保留 `session.log`
- 检查是否出现 `fault:`、overflow、stale 增长或非预期 reset

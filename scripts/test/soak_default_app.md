# soak_default_app.ps1

## 用途
- 默认 APP 的板级 soak 稳定性测试（可短时样本，也可长时运行）
- 自动执行：
- 可选烧录
- 可选复位
- 串口轮询命令并统计通过率
- 汇总健康/故障相关计数

## 支持与前提
- 板配置来自 `scripts/config/board_profiles.json`
- 支持通过 `-Board` 选择板型（如 `f411-nucleo`、`f103rct6-generic`）
- 多探针场景建议显式 `-Probe`（映射到 `probe-rs --probe`）

## 基本命令

### 1) F411 短时 soak（烧录+复位）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f411-nucleo -Port COM6 -DurationSec 60
```

### 2) F103 短时 soak（烧录+复位+指定 probe）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f103rct6-generic -Port COM17 -Probe 0483:3748 -DurationSec 60
```

### 3) 已预烧录，串口-only soak（不烧录不复位）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f103rct6-generic -Port COM17 -NoFlash -NoReset -DurationSec 60
```

## 参数
- `-Port`：必填，串口号
- `-Board`：板型，默认 `f411-nucleo`
- `-Probe`：可选，指定探针 `VID:PID[:SERIAL]`
- `-DurationSec`：测试时长，默认 `60`
- `-Baud`：串口波特率，默认 `115200`
- `-Binary`：可选，覆盖默认镜像路径
- `-Chip`：可选，覆盖默认 chip
- `-Speed`：SWD 速度，默认 `100`
- `-NoFlash`：跳过烧录
- `-NoReset`：跳过 probe reset
- `-RunId`：可选，指定输出目录名（供后台启动脚本复用）

## 固定命令轮询
- `PING`
- `ECHO soak`
- `LED TOGGLE`
- `PWM 50`
- `STAT`

判定：
- `PING` -> `PONG`
- `ECHO` -> `soak`
- `LED` -> `OK` 或 `ERR led_unavailable`
- `PWM` -> `OK` 或 `ERR pwm_unavailable`
- `STAT` -> 以 `STAT ` 开头

## 输出
- `runs/soak/<timestamp>/session.log`
- `runs/soak/<timestamp>/summary.csv`
- `runs/soak/<timestamp>/summary.json`

关键字段：
- `commands_sent / commands_passed / commands_failed`
- `fault_lines / error_lines`
- `max_feeds / max_stale / max_rx_overflow / max_tx_overflow / max_cmd_drop`


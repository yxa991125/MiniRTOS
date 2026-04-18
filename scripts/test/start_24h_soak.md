# start_24h_soak.ps1

## 用途
- 以后台进程方式启动长时 soak，避免误关交互式 PowerShell 窗口导致中断。
- 实际执行体是 `scripts/test/soak_default_app.ps1`。

## 常用命令

### 1) F411 后台 24h soak
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/start_24h_soak.ps1 -Board f411-nucleo -Port COM6
```

### 2) F103 后台 24h soak（多探针场景）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/start_24h_soak.ps1 -Board f103rct6-generic -Port COM17 -Probe 0483:3748
```

### 3) 不烧录不复位（仅串口轮询）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/start_24h_soak.ps1 -Board f103rct6-generic -Port COM17 -NoFlash -NoReset
```

## 参数
- `-Port`：必填，串口号
- `-Board`：板型，默认 `f411-nucleo`
- `-Probe`：可选，探针 `VID:PID[:SERIAL]`
- `-DurationSec`：默认 `86400`（24h）
- `-NoFlash`：跳过烧录
- `-NoReset`：跳过复位

其余参数（`-Baud`、`-Binary`、`-Chip`、`-Speed`、`-ReadSliceMs`、`-PauseMs`）会透传给 `soak_default_app.ps1`。

## 输出
- `runs/soak/<run_id>/job.json`
- `runs/soak/<run_id>/launcher.stdout.log`
- `runs/soak/<run_id>/launcher.stderr.log`
- `runs/soak/<run_id>/session.log`
- `runs/soak/<run_id>/summary.csv`
- `runs/soak/<run_id>/summary.json`


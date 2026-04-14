# run_multiboard_regression.ps1

## 用途
- 统一执行多板回归入口，覆盖：
- 按矩阵 `build_board.ps1` compile-only 构建
- 可选 `bench` 构建
- 可选多板 `run_app_smoke.ps1` 串口烟雾验证

## 核心泛化能力
- 板配置统一来自 `scripts/board_profiles.json`
- 支持通过 `-BuildMatrix` 扩展新板构建任务
- 支持通过 `-SmokeBoardPorts` / `-SmokeBoardProbes` 扩展新板 smoke 任务
- 兼容旧参数：`-F103Port` / `-F411Port`（未使用 `-SmokeBoardPorts` 时仍可用）

## 参数
- `-BuildMatrix`：构建矩阵，格式 `board:profile:mode[:required|optional]`
- `-IncludeBench`：追加 `f411-nucleo:release:bench:required`
- `-IncludeF103Debug`：追加 `f103c8-bluepill:debug:app:optional`
- `-SkipSmoke`：跳过所有 smoke 步骤
- `-SmokeBoardPorts`：`board:COMx` 或 `board=COMx`（支持逗号/分号拼接）
- `-SmokeBoardProbes`：`board:VID:PID[:SERIAL]` 或 `board=...`（用于多 probe 定向）
- `-F103Port` / `-F411Port`：旧版兼容端口参数
- `-F103Probe` / `-F411Probe`：旧版兼容 probe 参数
- `-FlashOnSmoke`：smoke 前是否烧录，支持 `true/false`、`1/0` 等
- `-AutoDisableFlashWhenProbeMissing`：启用后，若无 probe 自动降级为不烧录 smoke
- `-SmokeReadTimeoutMs`：smoke 命令读超时
- `-SmokeStartupWindowMs`：smoke 启动窗口

## 典型命令

### 1) 仅 compile-only 回归
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -SkipSmoke
```

### 2) 双板 smoke（旧参数兼容）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 -F103Port COM17 -F411Port COM6 -FlashOnSmoke false
```

### 3) 双板 smoke（推荐，通用参数）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 `
  -SmokeBoardPorts "f103rct6-generic:COM17,f411-nucleo:COM6" `
  -SmokeBoardProbes "f103rct6-generic:0483:3748,f411-nucleo:0483:374b:02290191523200213638414B" `
  -FlashOnSmoke true
```

### 4) 新板加入后的构建扩展（示例）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 `
  -BuildMatrix @(
    "f411-nucleo:debug:app:required",
    "f411-nucleo:release:app:required",
    "f103c8-bluepill:release:app:required",
    "new-board:release:app:required"
  ) `
  -SkipSmoke
```

## 输出
- `regression_runs/<timestamp>/meta.json`
- `regression_runs/<timestamp>/summary.csv`
- `regression_runs/<timestamp>/summary.json`
- 每一步 `stdout/stderr` 日志

## 失败判定
- `required=true` 的步骤失败则脚本退出码非 `0`
- `optional` 步骤失败只记录，不作为整体失败门禁

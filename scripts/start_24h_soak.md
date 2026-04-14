# 24h soak 后台启动脚本

## 目的
- 以隐藏窗口方式启动默认 APP 的长稳 soak，避免误关交互式 PowerShell 窗口导致测试中断。

## 命令
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/start_24h_soak.ps1 -Port COM6
```

## 行为
- 默认启动 `86400s`（`24h`） soak
- 后台启动 `scripts/soak_default_app.ps1`
- 自动生成唯一 `run_id`
- 在 `app_soak_runs/<run_id>/job.json` 中记录：
- 进程 `pid`
- 输出目录
- 启动时间
- 端口和持续时长

## 结果查看
- 日志：`app_soak_runs/<run_id>/session.log`
- 结果汇总：`app_soak_runs/<run_id>/summary.csv`
- JSON 摘要：`app_soak_runs/<run_id>/summary.json`

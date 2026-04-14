# host 侧测试脚本

## 目的
- 在主机环境运行纯逻辑测试，不依赖板卡、`probe-rs` 或裸机目标。
- 当前覆盖：
- `src/app_protocol.rs`：行协议组帧与命令解析
- `src/ipc/ringbuf_core.rs`：固定容量环形缓冲区核心逻辑

## 命令
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_host_tests.ps1
```

## 默认行为
- 使用 `host_tests/Cargo.toml`
- 默认目标：`x86_64-pc-windows-msvc`

## 当前测试点
- `PING / ECHO / LED / PWM / STAT` 命令解析
- 非法命令拒绝
- UART 行协议的完整包、半包、粘包、超长行恢复
- ring buffer 的满/空、回卷、顺序保持

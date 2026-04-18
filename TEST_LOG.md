# 测试日志

## 2026-04-15 当前有效结论

### F411
- smoke：`runs/smoke/20260415_141156_f411-nucleo/`
- `1h soak`：`runs/soak/20260415_2411_f411_1h/`
- 结果：`8088/8088` 命令通过，`fault=0`

### F103
- smoke：`runs/smoke/20260415_185129_f103rct6-generic/`
- `1h soak`：
  - `runs/soak/20260415_131037/`
  - `runs/soak/20260415_141355/`
- 系统层结论：无 `fault`、无异常复位、无 `stale`，主线稳定性通过

### F103 UART 抗干扰快速修复
- 修复内容：检测到 UART 硬件接收错误后，直接丢弃当前行，避免脏字节进入命令解析链路
- 短时回归：`runs/soak/20260415_f103_uart_fix_180s/`
- 结果：`426/426` 命令通过，`fault=0`，`error_lines=0`

## 2026-04-15 性能与基线
- F411 release bench 基线：`runs/bench/20260331_143830/`
- 当前 bench 仍只承诺 `board-f411-nucleo`

## 2026-04-15 多板回归
- compile-only 回归基线：`runs/regression/20260414_161915/`
- 结论：多板脚本入口可用，`board-f411-nucleo` 与 `board-f103c8-bluepill` 的构建链路已统一到脚本与 `board profile` 模型

## 仍需持续观察
- `F103 UART` 长时抗干扰样本继续积累
- 更长时长 soak 可继续做，但已不作为当前阶段主线阻塞项

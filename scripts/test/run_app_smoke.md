# run_app_smoke.ps1

## 用途
- 仅做默认 APP 的串口烟雾验证。
- 默认要求镜像已经构建完成。
- 正式回归路径中不负责烧录；仅人工便捷场景允许通过 `-Flash` 链式调用烧录。

## 当前支持
- `f411-nucleo`
- `f103c8-bluepill`
- `f103rct6-generic`（脚本别名，便于 `STM32F103RC*` 实板验收）

## 参数
- `-Board`：板型名称
- `-Port`：串口号
- `-BaudRate`：默认 `115200`
- `-ReadTimeoutMs`：单条命令等待超时
- `-StartupWindowMs`：启动阶段日志观察窗口
- `-ProbeSpeed`：执行 `probe-rs reset` 的 SWD 速度（默认 `100`）
- `-Probe`：可选，指定 `probe-rs --probe`（多探针场景建议显式指定）
- `-RequireBootBanner`：要求在启动窗口内看到 `boot ok`
- `-ResetBeforeCapture`：打开串口后先执行一次 `probe-rs reset`
- `-Flash`：本地便捷模式下先调用 `flash_board.ps1`
- `-Image`：配合 `-Flash` 指定镜像路径

说明：
- 当使用 `-Flash` 时，脚本会执行“烧录 -> 打开串口 -> reset -> 采集/命令验证”。
- 这样可以降低启动日志在串口尚未打开时丢失的问题。
- 串口读行会做 `\r\n` 归一化（去掉尾部 `\r`/`\n`），避免因行结束符差异造成命令匹配假失败。
- 板配置来自 `scripts/config/board_profiles.json`。

## 验证内容
- `PING -> PONG`
- `ECHO smoke -> smoke`
- `STAT -> STAT ...`
- `LED TOGGLE -> OK / ERR led_unavailable`
- `PWM 50 -> OK / ERR pwm_unavailable`

## 输出
- `runs/smoke/<timestamp>_<board>/session.log`
- `runs/smoke/<timestamp>_<board>/summary.csv`
- `runs/smoke/<timestamp>_<board>/summary.json`
- `runs/smoke/<timestamp>_<board>/smoke_meta.json`


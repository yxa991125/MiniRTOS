# flash_board.ps1

## 用途
- 仅做按板烧录与复位。
- 不做构建，不做串口烟雾验证。

## 当前支持
- `f411-nucleo`
- `f103c8-bluepill`
- `f103rct6-generic`（脚本别名，芯片按 `STM32F103RC` 处理）

## 参数
- `-Board`：板型名称
- `-Image`：待烧录镜像路径
- `-Speed`：SWD 速度，默认 `100`
- `-Probe`：可选，指定 `probe-rs --probe`（多探针场景建议显式指定）
- `-ResetAfter`：烧录后执行一次 `probe-rs reset`
- `-SkipVerify`：跳过 `--verify`

## 典型命令
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f411-nucleo -Image target/thumbv7em-none-eabihf/release/CortexOS -ResetAfter
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f103c8-bluepill -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -Probe 0483:3748 -ResetAfter
```

## 输出
- `runs/flash/<timestamp>_<board>/flash.log`
- `runs/flash/<timestamp>_<board>/flash_meta.json`

## 说明
- 板配置来自 `scripts/config/board_profiles.json`。


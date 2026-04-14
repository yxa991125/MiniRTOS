# build_board.ps1

## 用途
- 仅做按板 compile-only 构建。
- 不负责烧录，不负责串口验证。

## 当前支持
- `f411-nucleo`
- `f103c8-bluepill`
- `f103rct6-generic`（脚本别名，映射到 `board-f103c8-bluepill` feature）

## 参数
- `-Board`：板型名称，当前支持 `f411-nucleo` / `f103c8-bluepill` / `f103rct6-generic`
- `-Profile`：`debug` 或 `release`
- `-Mode`：`app` / `bench` / `uart-probe`

## 典型命令
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe
```

## 输出
- `board_builds/<timestamp>_<board>_<profile>_<mode>/build.log`
- `board_builds/<timestamp>_<board>_<profile>_<mode>/build_meta.json`

## 说明
- 脚本会显式使用 `--no-default-features --features board-*`。
- `f103rct6-generic` 仅是脚本别名，便于在 `STM32F103RC*` 实板上复用同一 feature 进行验证。
- 板配置来自 `scripts/board_profiles.json`，新增板型时优先更新该文件。

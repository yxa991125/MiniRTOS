# new_board_scaffold.ps1

## 用途
- 为新板快速生成模板文件：
- `memory/<board>.x`
- `src/bsp/<board_module>.rs`
- 可选更新 `scripts/config/board_profiles.json`

## 典型命令

### 1) 仅生成模板（不改 board_profiles）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/new_board_scaffold.ps1 `
  -Board stm32g0b1-devkit `
  -Chip STM32G0B1RETx `
  -Target thumbv6m-none-eabi `
  -FlashKB 128 `
  -RamKB 36
```

### 2) 模板 + 注册到 board_profiles
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/new_board_scaffold.ps1 `
  -Board stm32g0b1-devkit `
  -Chip STM32G0B1RETx `
  -Target thumbv6m-none-eabi `
  -Aliases board-stm32g0b1-devkit `
  -RegisterInBoardProfiles `
  -SupportsUartProbe
```

### 3) 预演（不写文件）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/new_board_scaffold.ps1 `
  -Board stm32g0b1-devkit `
  -Chip STM32G0B1RETx `
  -Target thumbv6m-none-eabi `
  -RegisterInBoardProfiles `
  -DryRun
```

## 参数
- `-Board`：板名（小写 kebab-case）
- `-Chip`：probe-rs 芯片名
- `-Target`：Rust target triple
- `-Feature`：Cargo feature（默认 `board-<Board>`）
- `-Aliases`：板别名列表
- `-FlashKB` / `-RamKB`：memory 模板容量
- `-ProbeProtocol`：默认 `swd`
- `-SupportsBench`：是否支持 bench（写入 board profile）
- `-SupportsUartProbe`：是否支持 uart-probe（写入 board profile）
- `-RegisterInBoardProfiles`：写入 `scripts/config/board_profiles.json`
- `-Force`：覆盖已存在模板文件
- `-DryRun`：仅打印动作，不写文件

## 说明
- 脚本只做模板生成，不会自动修改 `Cargo.toml` 和 `src/bsp/mod.rs`。
- 生成后请按 `docs/release/BOARD_PORTING_GUIDE.md` 完成剩余接入步骤。


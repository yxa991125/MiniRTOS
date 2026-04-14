# 新增开发板指南（Board Porting Guide）

本文档说明如何把一块新板接入当前 CortexOS 多板框架。目标是：**新增板只改“板级配置 + BSP 实现 + 少量注册点”**，不改 RTOS 核心调度/IPC/同步逻辑。

## 1. 适用范围
- 当前主线：`Cortex-M3/M4`，无 `MPU/MMU` 低端板
- 当前脚本体系：`single crate + board feature + PowerShell`
- 当前板型示例：`f411-nucleo`、`f103c8-bluepill`

## 2. 新增一块板的最小清单

1. 新增 Cargo feature（并绑定 HAL/PAC 依赖）
2. 新增链接脚本 `memory/<board>.x`
3. 新增 BSP 文件 `src/bsp/<board>.rs`
4. 在 `src/bsp/mod.rs` 注册模块并导出 `current`
5. 在 `scripts/board_profiles.json` 增加板配置
6. 跑构建与 smoke 回归

---

## 2.1 一键脚手架（推荐）

可以先用脚手架脚本生成模板，再补实现：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/new_board_scaffold.ps1 `
  -Board myboard `
  -Chip STM32XXXX `
  -Target thumbv7m-none-eabi `
  -RegisterInBoardProfiles `
  -DryRun
```

去掉 `-DryRun` 后会实际生成：
- `memory/myboard.x`
- `src/bsp/myboard.rs`
- （可选）更新 `scripts/board_profiles.json`

---

## 3. 具体步骤

### 步骤 A：注册 feature 与依赖
编辑 `Cargo.toml`：
- 新增可选依赖（对应目标芯片 HAL/PAC）
- 新增 feature，例如：
- `board-myboard = ["dep:xxx-hal-or-pac"]`

说明：
- `build.rs` 会自动从 `CARGO_FEATURE_BOARD_*` 识别当前 board feature 并选择 `memory/<board>.x`
- 规则：feature 名 `board-foo-bar` 对应链接脚本 `memory/foo-bar.x`

### 步骤 B：新增链接脚本
新增文件 `memory/<board>.x`，至少包含：
- `FLASH` 起始地址/容量
- `RAM` 起始地址/容量
- 基本段布局

建议：
- 先用保守容量（尤其是 Blue Pill 这类容量混板情况）

### 步骤 C：实现 BSP
新增 `src/bsp/<board>.rs`，按当前 BSP 约定实现：

1. `BoardContext`
- `take()`：接管外设 + 初始化板级硬件
- `reset_reason()` / `sysclk_hz()` / `emit_boot_banner()`
- `init_bench()`（若本轮不支持 bench，可显式 panic/返回不支持）

2. `controls` 模块
- `led_available()`
- `set_led(on: bool)`
- `toggle_led()`
- `pwm_available()`
- `set_pwm_percent(percent: u8)`

3. `uart` 模块
- `boot_write_bytes()`
- `init_hardware()`
- `init_app_uart()`
- `app_is_ready()`
- `app_wait_for_rx()/clear_rx_event()/app_read_byte()`
- `app_wait_for_tx()/clear_tx_event()/app_enqueue_tx_bytes()/app_drain_tx()`
- `app_stats()`
- 串口 IRQ handler

4. `watchdog` 模块
- `start(timeout_ms)`
- `feed()`

### 步骤 D：注册 BSP 模块
编辑 `src/bsp/mod.rs`：
- 新增 `pub mod <board>;`
- 新增 `pub use <board> as current;`（按 feature 选择）

注意：
- 这里是“板模块路由点”，新增板时必须同步更新。

### 步骤 E：注册脚本板配置
编辑 `scripts/board_profiles.json`，新增一项：
- `name`
- `aliases`
- `feature`
- `target`
- `chip`
- `probe_protocol`
- `supports.bench`
- `supports.uart_probe`

示例：
```json
{
  "name": "myboard",
  "aliases": ["board-myboard"],
  "feature": "board-myboard",
  "target": "thumbv7m-none-eabi",
  "chip": "STM32F103RC",
  "probe_protocol": "swd",
  "supports": { "bench": false, "uart_probe": true }
}
```

---

## 4. 验收流程（建议）

### 4.1 Compile-only
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board myboard -Profile release -Mode app
```

### 4.2 烧录（多探针时建议指定 `-Probe`）
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/flash_board.ps1 -Board myboard -Image target/<target>/release/CortexOS -Probe <VID:PID[:SERIAL]> -ResetAfter
```

### 4.3 APP 烟雾
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_app_smoke.ps1 -Board myboard -Port COMx -Probe <VID:PID[:SERIAL]> -Flash
```

### 4.4 纳入多板回归
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run_multiboard_regression.ps1 `
  -BuildMatrix @(
    "f411-nucleo:debug:app:required",
    "f411-nucleo:release:app:required",
    "myboard:release:app:required"
  ) `
  -SmokeBoardPorts "myboard:COMx" `
  -SmokeBoardProbes "myboard:<VID:PID[:SERIAL]>" `
  -FlashOnSmoke true
```

---

## 5. 常见问题

### Q1：两把 ST-Link 同时在线，脚本烧录失败并提示 probe 选择？
原因：`probe-rs` 进入交互式选择，脚本是非交互执行。  
处理：为 `flash/smoke/regression` 显式传 `-Probe`（底层映射到 `probe-rs --probe`）。

### Q2：新增板后 build.rs 报找不到 memory 脚本？
检查：
- feature 名是否为 `board-xxx`
- 是否存在 `memory/xxx.x`

### Q3：新增板后脚本提示 unsupported board？
检查 `scripts/board_profiles.json` 是否新增并拼写一致（`name` 与 `aliases`）。

---

## 6. 变更约束
- 不要在板移植时修改 RTOS 核心语义（调度、IPC、同步）
- 板差异尽量收敛在 `src/bsp/*` 与 `scripts/board_profiles.json`
- 多板相关脚本改动后，至少执行一次：
- `scripts/run_multiboard_regression.ps1 -SkipSmoke`

# AGENT_HANDOFF（完整交接版）

## 0. 文档定位
本文件用于“新 agent 接手当前项目时的一次性全景交接”。
目标是让新 agent 即使不追历史聊天，也能独立完成后续开发。

适用范围：
- 多板封装开发
- 板级联调
- 默认 APP 验证
- bench 验证与数据采集
- 文档维护与测试闭环

不替代：
- `README.md`（发布版说明）
- `TESTING.md`（测试规范与结果台账）
- `DEVELOPER_GUIDE.md`（开发细节）
- `CODEX_LOG.md`（开发日志）

---

## 1. 项目总览
### 1.1 项目名称
- `CortexOS`

### 1.2 一句话说明
- 一个面向低端无 MPU/MMU 开发板的 `Rust + no_std` RTOS 工程，正在从单板实现升级为可扩展多板框架。

### 1.3 当前主线
- 不是“补核心功能”。
- 是“多板封装 + 板级验收 + 稳定性收尾”。

### 1.4 当前优先级
1. 完成 `F103` 默认 APP 板级验收（命令、LED、PWM）。
2. 完成封装后稳定性验收（`1h soak + 多轮重复`）。
3. 固化多板回归脚本流程。
4. 后续再进入安全性研究（safe/unsafe 边界重构）。

---

## 2. 当前支持板卡与 Profile

### 2.1 board-f411-nucleo（主稳定板）
- target: `thumbv7em-none-eabihf`
- chip: `STM32F411RETx`
- 串口：`USART2 PA2/PA3`
- LED：`PA5`
- PWM：`TIM1 CH1 / PA8`
- watchdog：`IWDG`
- memory 脚本：`memory/f411-nucleo.x`
- 当前状态：默认 APP、bench、脚本链路均可用

### 2.2 board-f103c8-bluepill（第二板）
- target: `thumbv7m-none-eabi`
- chip: `STM32F103C8`
- 保守内存策略：`FLASH=64K`、`RAM=20K`
- memory 脚本：`memory/f103c8-bluepill.x`
- 当前状态：运行时 BSP 已接入，板级验收进行中

### 2.3 f103rct6-generic（脚本别名）
- 用途：适配你现在实际使用的 `STM32F103RCT6/RC` 类板
- 构建特性：映射到 `board-f103c8-bluepill`
- 烧录 chip：使用 `STM32F103RC`
- 目标：复用同一套多板封装路径，先确保开发效率

---

## 3. 功能实现现状（按模块）

### 3.1 内核与调度
已具备：
- 抢占式调度
- 时间片轮转
- 任务状态：运行/就绪/阻塞/睡眠/超时
- O(1) ready 选择（bitmap + ready queue）
- timeout wheel 超时唤醒路径
- idle 不入 ready queue、idle 不参与时间片

关键文件：
- `src/task/scheduler.rs`
- `src/task/tcb.rs`
- `src/task/context.rs`
- `src/arch/cortex_m/pendsv.S`

### 3.2 定时器
已具备：
- `SysTick`
- 软定时器
- bench 使用硬件计时路径

关键文件：
- `src/timer/systick.rs`
- `src/timer/soft_timer.rs`
- `src/timer/hw_timer.rs`

### 3.3 IPC 与同步
已具备：
- `Semaphore`
- `Event`
- `SyncMsgQueue`
- `IrqMutex`
- `BlockingMutex`
- 基础优先级继承链路

关键文件：
- `src/sync/semaphore.rs`
- `src/sync/event.rs`
- `src/sync/mutex.rs`
- `src/ipc/mqueue.rs`
- `src/ipc/ringbuf.rs`
- `src/ipc/ringbuf_core.rs`

### 3.4 诊断与生存性
已具备：
- Fault dump 与 reset reason 路径
- trace counters
- 任务运行快照与栈水位
- 任务心跳机制
- `system_health()` 快照
- 条件喂狗（仅关键任务健康时 feed）

关键文件：
- `src/kernel.rs`
- `src/task/diagnostics.rs`
- `src/log.rs`
- `src/platform/diagnostics.rs`
- `src/platform/watchdog.rs`

### 3.5 默认 APP（控制模板）
已具备：
- 串口行协议（CRLF）
- 命令：`PING`、`ECHO`、`LED`、`PWM`、`STAT`
- 任务拆分：RX / CMD / TX / HEALTH
- 静态内存、固定容量

关键文件：
- `src/app.rs`
- `src/app_protocol.rs`
- `src/device/uart.rs`

### 3.6 bench 基准体系
已具备：
- 上下文切换、睡眠唤醒、IRQ-to-task
- semaphore / queue / mutex / PI
- soft timer callback
- scaling 与 O(1) 诊断输出

关键文件：
- `src/bench.rs`
- `scripts/collect_release_bench.ps1`

---

## 4. 分层架构与依赖方向（非常关键）

目标依赖方向：
- `kernel` -> `platform facade` -> `bsp/device implementation`

当前分层：
- `src/arch/cortex_m`：架构层
- `src/kernel.rs` + `src/task/*` + `src/sync/*` + `src/ipc/*` + `src/timer/*`：内核层
- `src/platform/*`：平台门面层（统一内核访问板级服务）
- `src/device/*`：设备服务接口与包装
- `src/bsp/*`：板级 HAL/PAC 绑定与资源映射
- `src/app.rs` / `src/bench.rs`：应用与测试固件层

禁止回退：
- 不要让 `kernel` 直接依赖某板 HAL 类型
- 不要让 `main.rs` 重新塞入板级 GPIO/UART/PWM 初始化细节
- 不要恢复根目录旧 `memory.x`

---

## 5. 关键模式与构建系统说明

### 5.1 固件模式
- `Mode=app`：默认应用固件
- `Mode=bench`：性能基准固件
- `Mode=uart-probe`：F103 串口/运行排障固件（不进 RTOS 调度）

模式约束：
- `bench` 与 `uart-probe` 互斥（`main.rs` 有编译期限制）

### 5.2 板级构建脚本
- `scripts/build_board.ps1`
- 入参：`-Board`、`-Profile debug|release`、`-Mode app|bench|uart-probe`
- 负责：按板选择 `target + features` 并构建

### 5.3 板级烧录脚本
- `scripts/flash_board.ps1`
- 入参：`-Board`、`-Image`、`-ResetAfter`、`-Speed`
- 负责：probe-rs 下载/校验/可选 reset

### 5.4 APP 烟雾脚本
- `scripts/run_app_smoke.ps1`
- 入参：`-Board`、`-Port`、`-BaudRate`、`-Flash`
- 负责：串口命令闭环验证

### 5.5 其他脚本
- `scripts/run_host_tests.ps1`：host 逻辑测试
- `scripts/collect_release_bench.ps1`：bench 批量采集
- `scripts/soak_default_app.ps1`：默认 APP soak
- `scripts/start_24h_soak.ps1`：后台长稳启动
- `serial_io_test.ps1`：手工串口输入输出验证

---

## 6. F103 近期关键排障结论（必须知道）

### 6.1 串口链路曾“无输出”的关键原因
- 曾存在错误链接脚本/内存配置影响启动稳定性的问题。
- 已修复：
  - 根目录 `memory.x` 已移除
  - 使用 `memory/f103c8-bluepill.x`
  - `__STACK_START` 已在板级 memory 脚本中显式导出

### 6.2 uart-probe 现状
- 已确认可在 `COM14` 看到：
  - `boot ok (F103)`
  - `uart probe heartbeat`
  - 手工输入可回显 `rx: <text>`
- 当前 probe 并发覆盖：`USART1(PA9/PA10)`、`USART2(PA2/PA3)`、`USART3(PB10/PB11)`
- 目的：快速定位板卡实际 USB-UART 路由

### 6.3 实操时序建议
- 先打开串口工具，再 reset，最容易看到首条 boot banner。
- 若没看到 boot banner，但持续有 heartbeat，说明固件仍在运行。

---

## 7. 当前测试状态（摘要）

### 7.1 已通过
- F411：APP + bench 构建与主链路可用
- bench 稳定批次：`bench_runs/20260331_143830/`（30/30 完整）
- host 逻辑测试：通过
- F103：`uart-probe` 已实机验证（有心跳、有回显）

### 7.2 未关闭项（当前任务）
- F103 默认 APP 的 `PING/ECHO/STAT` 烟雾通过
- F103 `LED TOGGLE` 物理验收
- F103 `PWM 50` 物理验收
- 封装后 `1h soak + 多轮` 稳定性收尾

---

## 8. 新 agent 首次执行清单（建议照抄）

### 8.1 阅读顺序
1. `README.md`
2. `TESTING.md`
3. `AGENT_HANDOFF.md`（本文件）
4. `DEVELOPER_GUIDE.md`（需要做代码改动时）

### 8.2 第一步：构建回归
1. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode app`
2. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
3. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app`
4. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe`

### 8.3 第二步：先确认 F103 串口路由
1. 烧录 `uart-probe`
2. 打开串口 `COMx 115200 8N1`
3. reset 后观察 heartbeat
4. 发送任意文本，看是否回显 `rx:`

### 8.4 第三步：切换默认 APP 验收
1. 烧录 `app` 固件
2. 运行 `run_app_smoke.ps1`
3. 必要时手工发：`PING`、`ECHO hello`、`STAT`
4. 验证：`LED TOGGLE`、`PWM 50`

### 8.5 第四步：稳定性收尾
1. 执行 `soak_default_app.ps1`（先短时）
2. 执行 `1h` 样本与多轮重复
3. 更新 `TESTING.md` + `README.md` 待办状态

---

## 9. 维护规则（协作约束）

通用：
- 新增或修改功能后，至少补一条可复现测试路径。
- 未实际执行的测试，不写 PASS。

文档边界：
- 发布版内容：`README.md`、`TESTING.md`
- 开发版内容：`CODEX_LOG.md`、`Prompt.md`、`SERIAL_FEEDBACK_ANALYSIS.md`、`DEVELOPMENT_PLAN.md`、本文件等

代码边界：
- 优先通过 `kernel` 与 `platform` 门面编程
- 中断保持短路径
- 静态分配优先

高风险区（修改前先给验证方案）：
- `src/task/scheduler.rs`
- `src/task/tcb.rs`
- `src/arch/cortex_m/pendsv.S`
- `src/arch/cortex_m/boot.S`

---

## 10. 后续路线图（修订版）

### Phase A（当前，P0）
- 关闭 F103 默认 APP 板级验收项
- 固化 F103 串口与复位时序经验到脚本/文档

### Phase B（P1）
- 完成封装后 `1h soak + 多轮` 稳定性验收
- 将 F103 纳入常规回归矩阵（至少 compile-only + smoke）

### Phase C（P1/P2）
- 继续扩展更多 M3/M4 板 profile
- bench 继续以 F411 为主验收路径，其他板按需接入

### Phase D（后置）
- 安全性重构（safe/unsafe）
- 推荐顺序：
1. 继续封装提高边界清晰度
2. 定义 TCB 与共享状态边界
3. 做系统性 unsafe 收敛与审计

---

## 11. 当前仓库重要文件索引

工程根：
- `Cargo.toml`
- `build.rs`
- `memory/f411-nucleo.x`
- `memory/f103c8-bluepill.x`

源码核心：
- `src/main.rs`
- `src/kernel.rs`
- `src/platform/*`
- `src/bsp/*`
- `src/task/*`
- `src/app.rs`
- `src/app_protocol.rs`
- `src/bench.rs`
- `src/uart_probe.rs`

脚本：
- `scripts/build_board.ps1`
- `scripts/flash_board.ps1`
- `scripts/run_app_smoke.ps1`
- `scripts/run_host_tests.ps1`
- `scripts/collect_release_bench.ps1`
- `scripts/soak_default_app.ps1`
- `serial_io_test.ps1`

文档：
- `README.md`
- `TESTING.md`
- `DEVELOPER_GUIDE.md`
- `USER_GUIDE.md`
- `CODEX_LOG.md`
- `SERIAL_FEEDBACK_ANALYSIS.md`
- `DEVELOPMENT_PLAN.md`

---

## 12. 可直接给新 agent 的启动 Prompt（完整版）

```text
你现在接手 CortexOS 项目。当前主任务不是新增 RTOS 大功能，而是完成多板封装后的板级验收与稳定性收尾。

已知状态：
- F411 路径稳定（app + bench）。
- F103 已有运行时 BSP，uart-probe 已在实板串口回显通过。
- F103 默认 app 验收尚未完全关闭（PING/ECHO/STAT + LED/PWM 物理验证）。

工作原则：
- no_std、静态分配、ISR 短路径。
- kernel 通过 platform facade 访问板级能力。
- 不恢复根目录旧 memory.x。
- 未执行测试不写 PASS。

你的执行顺序：
1) 先读 README.md、TESTING.md、AGENT_HANDOFF.md。
2) 跑 build_board.ps1 的 F411/F103 构建回归。
3) 在 F103 上先跑 uart-probe，确认 heartbeat + rx 回显。
4) 切回 app 固件跑 run_app_smoke.ps1，完成 PING/ECHO/STAT。
5) 完成 LED/PWM 物理验收。
6) 执行 1h soak + 多轮重复。
7) 更新 TESTING.md 与 README.md 待办状态。

如果要改核心路径（scheduler/tcb/pendsv），先给验证方案再改。
```

---

## 13. 最后提示
本文件已经按“慢速理解也能完整接手”的标准写成。
若后续阶段目标发生变化，优先更新本文件第 3、7、10 章，再同步 `README.md` 与 `TESTING.md`。

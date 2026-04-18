# Codex Change Log

## 2026-03-02
- Implemented SysTick tick counter and software timer manager.
- Integrated timer tick into SysTick handler.
- Added timer module declaration and initialization.
- Updated README to reflect timer implementation.
- Files: src/timer/mod.rs, src/timer/systick.rs, src/timer/soft_timer.rs, src/task/context.rs, src/main.rs, README.md
- Tests: not run (not requested).

## 2026-03-02 (scheduler)
- Implemented time-slice round-robin selection for same-priority tasks.
- Added task states for blocked/sleeping with timeout wakeups.
- Added blocking/sleep APIs and integrated wake logic into SysTick.
- Updated README to reflect scheduler changes.
- Files: src/task/scheduler.rs, src/task/tcb.rs, src/task/context.rs, README.md
- Tests: not run (not requested).

## 2026-03-02 (scheduler gaps)
- Added idle task with `WFI` for low-power idle handling.
- Implemented task delete/exit and dynamic priority APIs.
- Enabled ready bitmap tracking with per-priority ready counts.
- Updated README to reflect scheduler gap closures.
- Files: src/task/scheduler.rs, README.md
- Tests: not run (not requested).

## 2026-03-02 (timer gaps)
- Deferred software timer callbacks to task context via pending queue.
- Added unified timer API (`device::timer::SystemTimer`) backed by SysTick.
- Updated README to reflect timer gap closures.
- Files: src/timer/soft_timer.rs, src/task/scheduler.rs, src/device/timer.rs, src/device/mod.rs, src/main.rs, README.md
- Tests: not run (not requested).

## 2026-03-02 (testing/debug)
- Fixed build errors in scheduler (missing `ready_counts` scope, duplicate bindings).
- Ran `cargo build` (success with unused-code warnings).
- Added testing document.
- Files: src/task/scheduler.rs, TESTING.md
- Tests: `cargo build`.

## 2026-03-02 (timer hw)
- Added HAL-based hardware timer driver wrapper (`HalTimerHz`) and timer error mapping.
- Updated README to reflect hardware timer driver completion and remaining gap.
- Files: src/device/timer.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (timer irq)
- Added TIM2 hardware timer interrupt integration and example initialization.
- Exposed `HalTimerHz` listen helpers for update events.
- Updated README to reflect TIM2 IRQ integration and remaining gaps.
- Files: src/timer/hw_timer.rs, src/timer/mod.rs, src/device/timer.rs, src/main.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (timer irq tim3)
- Added TIM3 hardware timer interrupt integration and example initialization.
- Updated README to include TIM3 example.
- Files: src/timer/hw_timer.rs, src/main.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (ipc)
- Implemented ring buffer and message queue IPC primitives with IRQ-safe wrappers.
- Added IPC module declarations and updated README.
- Files: src/ipc/ringbuf.rs, src/ipc/mqueue.rs, src/ipc/mod.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (mem)
- Implemented memory layout helper and fixed-block static pool allocator.
- Added mem module declarations and updated README.
- Files: src/mem/layout.rs, src/mem/static_pool.rs, src/mem/mod.rs, src/main.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (arch)
- Implemented Cortex-M interrupt/NVIC helpers and CPU instruction/register utilities.
- Updated README and ran build.
- Files: src/arch/cortex_m/interrupts.rs, src/arch/cortex_m/cpu.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (device)
- Implemented UART/GPIO/PWM/ADC device wrappers and updated module exports.
- Updated README and ran build.
- Files: src/device/uart.rs, src/device/gpio.rs, src/device/pwm.rs, src/device/adc.rs, src/device/mod.rs, README.md
- Tests: `cargo build`.

## 2026-03-02 (driver)
- Implemented basic motor/encoder/sensor driver wrappers and module exports.
- Updated README and ran build.
- Files: src/driver/motor.rs, src/driver/encoder.rs, src/driver/sensor.rs, src/driver/mod.rs, src/main.rs, README.md
- Tests: `cargo build`.

## 2026-03-03 (sync)
- Implemented IRQ-safe sync primitives (mutex/event/semaphore) with wait queues and scheduler integration for blocking/timeout.
- Added sync module declaration and current pid accessor.
- Updated README and ran build.
- Files: src/sync/mutex.rs, src/sync/event.rs, src/sync/semaphore.rs, src/sync/mod.rs, src/task/scheduler.rs, src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (kernel)
- Implemented kernel facade APIs for scheduler and timers.
- Added kernel module declaration and updated README.
- Files: src/kernel.rs, src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (kernel integration)
- Wired kernel facade into `main` initialization flow.
- Updated README and test log.
- Files: src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (rules)
- Added `Prompt` file to capture collaboration rules and documented the update in README/TESTING.
- Files: Prompt, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (docs/rules update only).

## 2026-03-03 (coding rules)
- Expanded `Prompt` with project coding rules and updated README.
- Files: Prompt, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (docs update only).

## 2026-03-03 (rules rename)
- Renamed `Prompt` to `rules.md` and added `docs/agent/Prompt.md` summary.
- Updated README and test log.
- Files: rules.md, docs/agent/Prompt.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (docs update only).

## 2026-03-03 (app features)
- Added UART print and LED blink tasks in `app.rs` and wired them into `main`.
- Initialized logger in `main` and configured PA5 LED.
- Updated README and test log.
- Files: src/app.rs, src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-04 (flash attempt)
- Attempted `cargo run` (probe-rs flash) for STM32F411RETx; failed with `JtagNoDeviceConnected` at 1000 kHz.
- Updated README/TESTING with the attempt record.
- Files: README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo run` (failed to open probe).

## 2026-03-04 (flash fix)
- Diagnosed the probe link and confirmed the target is reachable at `100 kHz` SWD.
- Updated `.cargo/config.toml` runner speed from `1000` to `100` and documented the workaround.
- Files: .cargo/config.toml, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `probe-rs info --chip STM32F411RETx --protocol swd --speed 100`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`.

## 2026-03-04 (flash verification)
- Verified low-speed SWD flashing with `probe-rs reset` after download.
- Confirmed `cargo run` leaves a long-lived `probe-rs` session that must be terminated before reopening the probe.
- Files: README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `probe-rs reset --chip STM32F411RETx --protocol swd --speed 100 --non-interactive`, `cargo run` (timed out in host session, left probe occupied until process cleanup).

## 2026-03-04 (rtos bench)
- Added a feature-gated benchmark firmware (`bench`) to test context switch, sleep wakeup, and TIM2 IRQ-to-task latency.
- Kept the default application path unchanged; benchmark logic is isolated behind `--features bench`.
- Added formatted logger access for benchmark metric output.
- Files: Cargo.toml, src/log.rs, src/bench.rs, src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`.

## 2026-03-04 (bench fix)
- Hardened the benchmark state machine after the first metric could complete while later stages stalled.
- Lowered the helper task priority after the context-switch test, added edge-wait timeout logs, and removed the IRQ block/unblock race with a critical section.
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`.

## 2026-03-04 (bench recheck)
- Reworked the post-context-switch transition again: helper task no longer gets deleted/reprioritized, and is instead parked into a blocked state before the sleep/IRQ benchmarks start.
- Added explicit expected UART progress markers (`bench: helper parked`, then `bench: sleep-wakeup start`) and documented that manual `probe-rs download` must be preceded by `cargo build --features bench` to avoid flashing a non-bench image from the shared debug path.
- Flashed the updated bench image with `probe-rs download --verify`; a follow-up `probe-rs reset` hit `os error 5` because the probe was re-occupied immediately after download.
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`

## 2026-03-04 (bench stack)
- Rechecked the unchanged UART output and moved the likely fault source to bench task stack pressure rather than the stage handoff itself.
- Increased bench-only task stacks in `src/main.rs` from `256/256` words to `1024/512` words, keeping the default application path unchanged.
- Rebuilt both targets and reflashed the bench image with `probe-rs download --verify`.
- Files: src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`

## 2026-03-10 (release bench path)
- Added explicit release benchmark path aliases in `.cargo/config.toml`: `bench-dev`, `bench-release`, `bench-release-build`.
- Bench UART init line now includes build profile (`debug` or `release`) to avoid flashing/measurement confusion.
- Documented stable release flashing flow using `target/thumbv7em-none-eabihf/release/CortexOS`.
- Files: .cargo/config.toml, src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo bench-release-build`.

## 2026-03-10 (bench anomaly fix)
- Fixed context benchmark over-count by preventing updates when `CTX_LEFT == 0` and clearing late `CTX_PENDING` after the context benchmark loop.
- Added bench-only idle behavior switch: while `STAGE_SLEEP` is active, idle no longer executes `WFI`, avoiding `DWT::CYCCNT` freeze and removing the false `sleep_1tick_extra=0` result.
- Kept default firmware behavior unchanged; the `WFI` bypass is compiled only with feature `bench`.
- Files: src/bench.rs, src/task/scheduler.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (sleep metric rework)
- Reworked `sleep_1tick_extra` measurement to use wakeup latency from the recorded SysTick edge (`now - last_systick_edge_cycle`) instead of `elapsed - 1ms` saturating subtraction.
- Added bench SysTick edge timestamp hook in `src/task/context.rs` and `bench::on_systick_edge()`.
- This addresses release runs that incorrectly reported `sleep_1tick_extra = 0` across all samples.
- Files: src/bench.rs, src/task/context.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (sleep metric rework v2)
- Fixed `sleep_1tick_extra` false +1 tick offset (~1000us) by changing the benchmark model:
- Snapshot SysTick edge cycle *before* `sleep_ms(1)`, then measure `resume - start_edge - one_tick`.
- Hardened sleep/timeout arming against SysTick races by moving `now/wake_tick` computation into scheduler critical sections.
- Files: src/bench.rs, src/task/scheduler.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (bench metrics expansion)
- Extended bench suite with new latency metrics:
- semaphore latency (`taskA give -> taskB wake`)
- queue latency (`ISR send -> task receive`)
- mutex lock/unlock latency
- soft timer callback latency
- scheduler scaling test at 2/8/32 tasks with per-switch normalization and O(1) verdict line.
- Added bench-only worker-task scaling infrastructure in `src/bench.rs`.
- Added bench-only scheduler capacity override (`MAX_TASKS=40`) to support 32-task scaling cases, while keeping default path unchanged (`MAX_TASKS=8`).
- Added `mod ipc;` in `src/main.rs` so bench can use `SyncMsgQueue`.
- Priority-inheritance status is explicitly logged as unsupported for current `IrqMutex`.
- Files: src/bench.rs, src/task/scheduler.rs, src/main.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (bench semaphore park race fix)
- Fixed race between helper park flag and actual block transition in benchmark helper task.
- In `STAGE_PARK_HELPER`, helper now sets `TASK_B_PARKED` and calls `block_current(None)` inside one critical section (`interrupt::free`), preventing false park-ack observations.
- This targets UART failures:
- `bench:semaphore_waiter timeout sample=0`
- `bench: helper re-park timeout`
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (bench sleep zero fix)
- Fixed `sleep_1tick_extra` reporting all-zero in release runs.
- Changed sleep metric in `src/bench.rs` to measure direct wake latency from latest SysTick edge after wakeup (`now - last_systick_edge_cycle`) under a critical-section snapshot.
- Removed one-tick saturating subtraction model for this metric.
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (README refactor)
- Removed fix/anomaly change-log content from `README.md`.
- Reorganized README around usage documentation only:
- runtime environment
- RTOS architecture
- directory map
- TODO list
- testing guide
- usage guide (parameter tuning, app规范, call path, runtime logic)
- Kept change history in `docs/agent/CODEX_LOG.md` and test records in `TESTING.md`.
- Files: README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-10 (README parameter split)
- Updated `README.md` parameter section to split tuning into:
- kernel parameter tuning
- benchmark/test parameter tuning
- Added concrete knobs for each category (tick rate, time slice, MAX_TASKS, bench samples, bench timer Hz, scaling cases, profile path).
- Files: README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-10 (queue metrics split)
- Split queue benchmark into two explicit metrics in `src/bench.rs`:
- `queue_wake_latency`: wake path latency from ISR wake stamp to task resume.
- `queue_end_to_end_latency`: ISR enqueue stamp to task `recv` completion.
- Replaced previous single output `queue_isr_send_to_task_recv` with the two new lines.
- Added queue metric description to `README.md` test section.
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (bench samples 1000)
- Increased benchmark sample count from `64` to `1000` in `src/bench.rs` (`BENCH_SAMPLES`).
- Updated README parameter section to document current default sample count.
- Files: src/bench.rs, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench -v`.

## 2026-03-13 (TESTING.md normalization)
- Reworked `TESTING.md` into a standardized test document and removed accumulated chronological dev logs.
- New structure focuses on:
- scope
- environment baseline
- test matrix + pass criteria
- bench output acceptance points
- latest execution snapshot
- known limits
- maintenance rules
- Updated README document convention entry to reflect the new TESTING role.
- Commands run for this update:
- `cargo build`
- `cargo build --features bench`
- `cargo build --release --features bench`
- Files: TESTING.md, README.md, docs/agent/CODEX_LOG.md
- Result: all three builds passed (warnings unchanged).

## 2026-03-13 (docs/agent/Prompt.md full rewrite)
- Replaced `docs/agent/Prompt.md` with a complete project collaboration prompt based on current repository context.
- New prompt includes: environment, hard rules, current bench status, test acceptance points, workflow, output style, and document responsibilities.
- Updated `README.md` document convention section to include `docs/agent/Prompt.md` role.
- Files: docs/agent/Prompt.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (development plan doc)
- Added `docs/dev/DEVELOPMENT_PLAN.md` as the dedicated discussion document for future roadmap, priorities, milestones, and open questions.
- Updated `README.md` document convention section to include the new plan document.
- Files: docs/dev/DEVELOPMENT_PLAN.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (scheduler O(1) migration)
- Replaced scheduler ready-task selection with per-priority ready queues plus ready bitmap lookup.
- Extended `Tcb` with ready-queue link fields and explicit queue-membership tracking.
- Removed task-table scanning from scheduler selection paths and updated:
- task creation
- first-task start
- context switch
- unblock
- delete
- priority change
- Added `docs/agent/SCHEDULER_O1_MIGRATION.md` to capture the full migration record.
- Updated README TODO to reflect the remaining O(n) timeout-scan limitation.
- Files: src/task/scheduler.rs, src/task/tcb.rs, docs/agent/SCHEDULER_O1_MIGRATION.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`

## 2026-03-17 (scheduler O(1) plan detail)
- Expanded `docs/dev/DEVELOPMENT_PLAN.md` with a dedicated scheduler section covering:
- current scheduler strategy
- current design issues
- concrete O(1) migration tasks
- suggested implementation order
- acceptance criteria
- Updated `README.md` TODO entry to point readers to the detailed plan document.
- Files: docs/dev/DEVELOPMENT_PLAN.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (scheduler migration doc localization)
- Translated `docs/agent/SCHEDULER_O1_MIGRATION.md` from English to Chinese.
- Updated `README.md` document convention to clarify that the migration record is maintained in Chinese.
- Files: docs/agent/SCHEDULER_O1_MIGRATION.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (timeout wakeup path optimization)
- Replaced the O(n) timeout scan in `tick_at` with a timeout wheel.
- Extended `Tcb` with timeout-link fields and per-task wheel rounds.
- Updated scheduler timeout-related paths:
- `init`
- `tick_at`
- `sleep_ms`
- `block_current`
- `unblock`
- `delete_task`
- Refreshed `docs/agent/SCHEDULER_O1_MIGRATION.md` to cover the timeout-wheel phase and updated README TODO / parameter docs.
- Files: src/task/scheduler.rs, src/task/tcb.rs, docs/agent/SCHEDULER_O1_MIGRATION.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`

## 2026-03-17 (serial bench analysis doc)
- Read `docs/data/串口反馈数据.txt` and generated a dedicated analysis document from the captured release bench output.
- The new document summarizes:
- boot/log integrity observations
- metric-by-metric interpretation
- scheduler benchmark caveats
- prioritized follow-up work
- Updated `README.md` document convention section to include the new analysis document.
- Files: docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, README.md, docs/agent/CODEX_LOG.md, TESTING.md
- Tests: not run (analysis/documentation-only update based on existing captured data)


## 2026-03-17 (serial analysis P0 follow-up)
- Added timeout-wheel validation coverage to bench:
- `timeout_wheel_sleep_1tick`
- `timeout_wheel_cross_bucket`
- `timeout_wheel_long_delay`
- `timeout_wheel_early_unblock`
- `timeout_wheel_wraparound`
- Added `bench_validate_timeout_wraparound()` in the scheduler for `u32` tick wrap-around self-check under `bench`.
- Adjusted `scheduler_o1_check` to compare steady-state `8/32` task cases while keeping `2-task` output only as a bias baseline.
- Added `scripts/bench/collect_release_bench.ps1` to automate repeated `release bench` flashing, serial capture, raw log storage, and `summary.csv` export.
- Updated release docs to reflect the new bench acceptance items and current P0 status.
- Updated development docs to record the code-side completion of P0 and the remaining hardware rerun work.
- Files: src/bench.rs, src/task/scheduler.rs, scripts/bench/collect_release_bench.ps1, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, README.md, TESTING.md, docs/agent/agent_work.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (wraparound self-check live-state fix)
- Root-caused the incomplete serial capture: bench stopped after `timeout_wheel_wraparound` because the self-check called `scheduler::init()` while the RTOS was already running.
- Reworked `bench_validate_timeout_wraparound()` to run on local `tasks/ready_queues/timeout_wheel` state inside a critical section.
- The new implementation preserves the live scheduler state and restores `READY_MASK` after the local simulation.
- Updated release/test docs to note that a fresh hardware rerun is required to verify `scheduler_scale`, `scheduler_o1_check`, and `bench complete`.
- Updated the serial analysis document to mark the old `docs/data/串口反馈数据.txt` capture as incomplete after the timeout-wheel stage.
- Files: src/task/scheduler.rs, README.md, TESTING.md, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (bench startup hang instrumentation)
- Investigated a new earlier boot-time stall where serial output stopped right after `bench tasks created, start_first_task()`.
- Changed bench startup to record task IDs at creation time instead of waiting for `task_b` to publish its PID through the first context switch.
- Added bench boot diagnostics:
- `bench tasks created: task_a=<pid> task_b=<pid>, start_first_task()`
- `bench: task_a entered pid=<pid>`
- `bench: task_b entered pid=<pid>`
- This narrows the failure boundary between `start_first_task` entry and the first `yield_now -> PendSV` switch.
- Files: src/main.rs, src/bench.rs, README.md, TESTING.md, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (bench early-hang fault instrumentation)
- Latest serial capture showed the stall moved further forward: `task_a` started executing, but output still stopped almost immediately.
- Reduced early bench diagnostics to literal strings:
- `bench: task_a entered`
- `bench: task_b entered`
- Added exception-side UART breadcrumbs in `src/task/context.rs`:
- `fault: HardFault`
- `fault: MemManage`
- `fault: BusFault`
- `fault: UsageFault`
- This separates three cases on the next run: first task never entered, first context switch failed, or execution faulted into an exception handler.
- Files: src/bench.rs, src/task/context.rs, README.md, TESTING.md, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (raw UART breadcrumbs)
- Added `log::emergency_write_str()` / `log::emergency_log_line()` to write directly to USART2 registers.
- Switched early bench breadcrumbs to raw UART output:
- `bench: task_a entered`
- `bench: task_b entered`
- `bench: context-switch start`
- Switched fault breadcrumbs to raw UART output to avoid `LOGGER + RefCell + interrupt::free` interference during early failures.
- Updated release/test docs and the serial analysis note to reflect that these lines now bypass the normal logger path.
- Files: src/log.rs, src/bench.rs, src/task/context.rs, README.md, TESTING.md, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (first PendSV hardfault narrowing)
- Read the updated `docs/data/串口反馈数据.txt`; the latest capture now stops at `bench: context-switch start` and then prints `fault: HardFault`.
- Hardened `src/arch/cortex_m/pendsv.S`:
- added `isb`
- switched LR preservation from a general register to an MSP stack save/restore sequence
- added fallback restore of the just-saved context when `__cortexos_switch_context` returns a null SP
- Added first-switch trace in `src/task/scheduler.rs`:
- `bench: ctxsw first from=... to=... save_sp=... next_sp=... pc=... xpsr=...`
- Added stack-pointer sanity filtering in `scheduler::context_switch()`; invalid selected stacks now fall back to the idle task instead of directly dereferencing a bad PSP.
- Enabled configurable fault handlers in `src/main.rs` and expanded `src/task/context.rs` fault dumps to print `MSP/PSP/CFSR/HFSR/MMFAR/BFAR` plus stacked `PC/LR/xPSR`.
- Files: src/arch/cortex_m/pendsv.S, src/task/scheduler.rs, src/task/context.rs, src/main.rs, src/log.rs, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`


## 2026-03-17 (bench task_a stack overflow)
- Read the updated `docs/data/串口反馈数据.txt`; the first-switch trace changed from a hard fault to:
- `bench: ctxsw first from=1 to=0 ... pc=0x080031ed`
- `0x080031ed` resolves to `idle_task`, which means the ready queue for bench tasks was already corrupted before the first switch completed.
- Cross-checked the release image with `arm-none-eabi-objdump`: `CortexOS::bench::task_a` has a release-frame allocation of `7872B` (`sub.w sp, sp, #7872`), while the previous bench stack budget was only `1024 words = 4096B`.
- Increased bench default stacks in `src/main.rs` to:
- `task_a = 4096 words`
- `task_b = 1024 words`
- This is a direct fix for the early stack smash that was pushing `task_a` below its stack base and corrupting scheduler state.
- Files: src/main.rs, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`, `arm-none-eabi-objdump -d -C ...`, `arm-none-eabi-nm -S ...`


## 2026-03-17 (hardware bench fully completed)
- Read the newest `docs/data/串口反馈数据.txt`; the hardware run now completes end-to-end.
- Key confirmations from the serial output:
- `bench: ctxsw first from=1 to=2 ...`
- `bench: task_b entered`
- all `timeout_wheel_*` checks passed
- `scheduler_o1_check ... verdict=likely_o1`
- `bench complete`
- Rewrote `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md` into a clean UTF-8 Chinese analysis document and updated it with:
- latest metric interpretation
- context-switch outlier explanation (first-sample diagnostics contaminating the statistic)
- timeout wheel and scheduler O(1) conclusions
- next-step recommendations
- Updated `TESTING.md` latest execution results to mark T05/T06/T07/T08 as PASS based on the completed hardware serial run.
- Files: docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, TESTING.md, docs/agent/CODEX_LOG.md

## 2026-03-23 (context-switch benchmark cleanup + multi-run baseline tooling)
- Completed the code-side part of `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md` section `9.1 P0`.
- Updated `src/bench.rs` so `context_switch_a_to_b` no longer uses the raw first sample in its final statistic:
- added `CONTEXT_SKIP_SAMPLES = 1`
- stored raw context-switch samples
- changed the output to `count/skipped/min/avg/p50/p95/max`
- Added percentile calculation with nearest-rank `p50/p95` for the context-switch metric.
- Reworked `scripts/bench/collect_release_bench.ps1`:
- default `-Runs` is now `10`
- parses optional `skipped/p50/p95`
- still emits per-run `summary.csv`
- now also emits grouped `baseline_summary.csv`
- Attempted to execute the automated release collection locally, but `probe-rs` returned `No connected probes were found`, so the new `5~10` round hardware baseline has not yet been regenerated.
- Updated release-side docs to reflect the new metric format and collection flow.
- Files: src/bench.rs, scripts/bench/collect_release_bench.ps1, README.md, TESTING.md, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`, `cargo build --features bench`, `cargo build --release --features bench`, `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 1` (blocked: no probe)

## 2026-03-23 (collect_release_bench script documentation)
- Added `scripts/bench/collect_release_bench.md` as a dedicated document for the repeated release-bench collection script.
- Documented:
- script purpose
- prerequisites
- parameters
- execution flow
- parsed metric format
- output files (`run_*.log`, `summary.csv`, `baseline_summary.csv`)
- common failure modes
- recommended workflow
- Updated release-side docs to reference the new script document.
- Files: scripts/bench/collect_release_bench.md, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: docs-only

## 2026-03-24 (release bench collection script reset fix)
- Investigated why `run_*.log`, `summary.csv`, and `baseline_summary.csv` were empty during scripted collection.
- Root cause: the script used `probe-rs download --verify` and then immediately waited on the UART, but it did not explicitly reset/start the target after flashing.
- Updated `scripts/bench/collect_release_bench.ps1` to:
- run `probe-rs reset` after each successful download
- wait `ResetDelayMs` before starting serial reads
- warn when a run produces an empty serial log
- warn when `bench complete` is not observed before timeout
- Added/updated documentation for the revised script behavior.
- Files: scripts/bench/collect_release_bench.ps1, scripts/bench/collect_release_bench.md, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 0`

## 2026-03-24 (release bench timeout + baseline aggregation fix)
- Reviewed the user's `runs/bench/20260324_145839/` output.
- Finding 1: serial capture is working; `run_09.log` and `run_10.log` contain valid boot and partial bench output through `queue-latency start`.
- Finding 2: the previous `ReadTimeoutMs = 30000` was too short for a full `release + bench` run with `BENCH_SAMPLES = 1000`, so the script timed out before `bench complete`.
- Finding 3: `baseline_summary.csv` aggregation was broken because `$runs` collided with the script parameter `$Runs` (PowerShell variables are case-insensitive).
- Updated `scripts/bench/collect_release_bench.ps1` to:
- raise default `ReadTimeoutMs` to `120000`
- keep the explicit `probe-rs reset`
- tell the user to increase `-ReadTimeoutMs` when only partial logs are collected
- rename the aggregation-local variable to avoid the `$Runs` collision
- Updated release/test docs and the dedicated script document.
- Files: scripts/bench/collect_release_bench.ps1, scripts/bench/collect_release_bench.md, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: inspected `runs/bench/20260324_145839/run_09.log`, `runs/bench/20260324_145839/run_10.log`, `runs/bench/20260324_145839/summary.csv`, `runs/bench/20260324_145839/baseline_summary.csv`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 0`

## 2026-03-24 (10-run release bench baseline captured)
- Reviewed the latest hardware capture folder: `runs/bench/20260324_155928/`.
- Confirmed the scripted collection is now working end-to-end:
- `10/10` logs contain `bench complete`
- `10/10` logs contain `bench: ctxsw first from=1 to=2`
- `10/10` logs contain `scheduler_o1_check ... verdict=likely_o1`
- `10/10` logs contain all five `timeout_wheel_* pass/fail` lines with `fail=0`
- Updated `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md` to replace the older single-run narrative with the new 10-run hardware baseline analysis.
- Updated `TESTING.md` latest execution results to mark `T04` and `T09` as PASS based on the completed automated collection.
- Noted two current tail-latency observations from the baseline:
- `queue_wake_latency` / `queue_end_to_end_latency` have two visibly higher runs (`run_08`, `run_10`)
- `mutex_lock_unlock` average is stable at `31cy`, but max has isolated spikes near `500cy`
- Files: runs/bench/20260324_155928/summary.csv, runs/bench/20260324_155928/baseline_summary.csv, runs/bench/20260324_155928/run_01.log, runs/bench/20260324_155928/run_10.log, docs/agent/SERIAL_FEEDBACK_ANALYSIS.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: data inspection only (no new build/flash performed by me in this turn)

## 2026-03-24 (P1 mutex/PI + P2 collection extraction completed)
- Added a blocking `BlockingMutex` implementation with basic priority inheritance hooks in the scheduler.
- Extended the bench path to emit three new metrics:
- `mutex_waiter_wake_latency`
- `priority_inheritance_enter_latency`
- `priority_inheritance_exit_latency`
- The capability line now reports `bench:mutex_priority_inheritance supported=1 mode=blocking_mutex`.
- Extended `scripts/bench/collect_release_bench.ps1` to extract and export:
- `timeout_validation.csv` / `timeout_validation_summary.csv`
- `scheduler_scale.csv` / `scheduler_scale_summary.csv`
- `scheduler_o1.csv` / `scheduler_o1_summary.csv`
- Updated release-facing docs (`README.md`, `TESTING.md`, `scripts/bench/collect_release_bench.md`) and the analysis document (`docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`).
- Tests: `cargo build`; `cargo build --features bench`; `cargo build --release --features bench`; PowerShell syntax check for `scripts/bench/collect_release_bench.ps1`; regex extraction check against `runs/bench/20260324_155928/run_01.log` (`metrics=8`, `validation=5`, `scale=3`, `o1=1`).

## 2026-03-24 (mutex PI bench first-run fix + script Int64 parsing)
- Reviewed `runs/bench/20260324_185313/run_01.log` after the first scripted run on the new mutex/PI bench build.
- Found two issues:
- `bench:mutex_pi_boost timeout sample=0` on the first mutex sample
- `scripts/bench/collect_release_bench.ps1` crashed because `mutex_waiter_wake_latency` emitted a `u32`-scale value (`3541348415cy`) that overflowed PowerShell `Int32` casts
- Root cause 1: `BlockingMutex::acquire()` was boosting the owner after the waiter had already transitioned to `Blocked`, so the owner sometimes resumed before inheriting the waiter priority.
- Root cause 2: the collection script parsed cycle values as `Int32` instead of `Int64`.
- Fixes applied:
- `src/sync/mutex.rs`: reorder contested acquire path to add the owner boost before `block_current()`
- `src/bench.rs`: wait explicitly for the boosted priority and ignore waiter-wake samples when `MUTEX_UNLOCK_STAMP == 0`
- `scripts/bench/collect_release_bench.ps1`: switch cycle/aggregate parsing from `Int32` arrays/casts to `Int64`
- Updated `TESTING.md`, `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`, and `scripts/bench/collect_release_bench.md` with the new finding and fix status.
- Tests: `cargo build --features bench`; `cargo build --release --features bench`; PowerShell syntax check for `scripts/bench/collect_release_bench.ps1`; regex parse check against `runs/bench/20260324_185313/run_01.log`.

## 2026-03-24 (20260324_190953 hardware rerun verified)
- Reviewed the latest 10-run hardware capture folder: `runs/bench/20260324_190953/`.
- Confirmed log completeness:
- `10/10` logs contain `bench complete`
- `10/10` logs contain `bench:mutex_priority_inheritance supported=1 mode=blocking_mutex`
- `10/10` logs contain `scheduler_o1_check ... verdict=likely_o1`
- `10/10` logs contain no `fault:` and no `timeout sample=`
- Confirmed new script outputs are present:
- `timeout_validation.csv` / `timeout_validation_summary.csv`
- `scheduler_scale.csv` / `scheduler_scale_summary.csv`
- `scheduler_o1.csv` / `scheduler_o1_summary.csv`
- Confirmed key aggregated baseline values from `baseline_summary.csv`:
- `context_switch_a_to_b avg_p50 = 417cy`
- `mutex_waiter_wake_latency avg_p50 = 1353cy`
- `priority_inheritance_enter_latency avg_p50 = 1521cy`
- `priority_inheritance_exit_latency avg_p50 = 2223cy`
- Updated release-side verification status in `TESTING.md`, updated latest baseline notes in `README.md`, and closed the `8.2 P1` / `8.3 P2` analysis items in `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`.
- Tests: data inspection only for `runs/bench/20260324_190953/` (no new build/flash executed by me in this turn)

## 2026-03-24 (queue / IrqMutex tail revalidation completed)
- Compared `runs/bench/20260324_155928/summary.csv` with `runs/bench/20260324_190953/summary.csv` for:
- `queue_wake_latency`
- `queue_end_to_end_latency`
- `mutex_lock_unlock`
- Findings:
- queue high-tail runs existed only in the older batch (`run_08`, `run_10`) and disappeared completely in `20260324_190953`
- `mutex_lock_unlock` still shows rare single-sample max spikes (`~497-498cy`), but `avg`, `avg_p50`, and `avg_p95` remain fixed at `31cy`
- Conclusion:
- no stable, reproducible abnormal tail was confirmed for queue
- `IrqMutex` currently shows isolated max spikes rather than a distribution-wide tail regression
- Updated `README.md` to replace the old queue/IrqMutex tail TODO with a narrower `IrqMutex` spike root-cause item.
- Updated `TESTING.md` with `T12 = PASS` for the tail revalidation result.
- Updated `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md` to record the comparison and resulting conclusion.
- Tests: data inspection only for `runs/bench/20260324_155928/summary.csv` and `runs/bench/20260324_190953/summary.csv`

## 2026-03-24 (IrqMutex spike attribution instrumentation added)
- Added `mutex_lock_unlock_attribution` to the bench mutex stage.
- The new output captures:
- `overlap_samples`
- `spikes`
- `irq_spikes`
- `clean_spikes`
- `systick_spikes`
- `tim2_spikes`
- `max_irq_spike`
- `max_clean_spike`
- The attribution logic uses `SysTick` and bench `TIM2` interrupt counters to distinguish interrupt-overlapped spikes from clean spikes.
- Extended `scripts/bench/collect_release_bench.ps1` to export:
- `mutex_lock_attribution.csv`
- `mutex_lock_attribution_summary.csv`
- Updated release/test/script docs to describe the new bench line and the new CSV outputs.
- Tests: `cargo build --features bench`; `cargo build --release --features bench`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 0`

## 2026-03-25 (20260324_222048 attribution rerun reviewed)
- Reviewed `runs/bench/20260324_222048/`.
- Confirmed `10/10` logs contain `bench complete`, no `fault:`, and include `bench:mutex_lock_unlock_attribution ...`.
- `mutex_lock_attribution_summary.csv` shows:
- `spikes_total = 5`
- `irq_spikes_total = 5`
- `clean_spikes_total = 0`
- `systick_spikes_total = 5`
- `tim2_spikes_total = 0`
- Conclusion: the current `IrqMutex` single-sample spikes are strongly correlated with `SysTick`, not with the lock fast path itself.
- Also noted that `queue`, `tim2_irq_to_task`, and `semaphore` tails reappeared in a few runs of the diagnostic batch, so the next attribution target should broaden beyond `IrqMutex`.
- Updated `README.md`, `TESTING.md`, and `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md` to reflect the new result.
- Tests: data inspection only for `runs/bench/20260324_222048/`

## 2026-03-25 (generic latency attribution instrumentation prepared)
- Extended bench attribution coverage beyond `IrqMutex`.
- New bench outputs now include:
- `semaphore_give_to_taskb_wake_attribution`
- `tim2_irq_to_task_attribution`
- `queue_wake_latency_attribution`
- `queue_end_to_end_latency_attribution`
- Generalized `scripts/bench/collect_release_bench.ps1` to export:
- `latency_attribution.csv`
- `latency_attribution_summary.csv`
- Rewrote `README.md`, `TESTING.md`, `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`, and `scripts/bench/collect_release_bench.md` to remove mojibake and document the new attribution flow.
- Current status: code-side attribution chain is ready; hardware rerun is still required before closing the `queue / IRQ / semaphore` attribution item.
- Tests: `cargo build --features bench`; `cargo build --release --features bench`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 0`

## 2026-03-25 (automated hardware rerun blocked by probe open failure)
- Ran `cargo build --release --features bench` successfully.
- Tried to execute `scripts/bench/collect_release_bench.ps1` against `COM6`.
- Initial finding: the old script opened the serial port before flashing; I changed it to open the serial port per-run after `probe-rs reset`.
- Retried the automated run, but `probe-rs download` still failed on run 1 with: `Failed to open probe -> USB error -> reset not supported by WinUSB`.
- Confirmed `probe-rs list` can enumerate the ST-Link, so the current blocker is probe open / driver state on this host, not missing hardware enumeration and not RTOS firmware behavior.
- Updated `scripts/bench/collect_release_bench.md` and `TESTING.md` to capture the blocker and the revised script flow.

## 2026-03-25 (20260325_131119 latency attribution rerun completed)
- Fixed `scripts/bench/collect_release_bench.ps1` capture order again: per run it now opens the serial port before `probe-rs reset`, so early boot and semaphore logs are no longer dropped.
- Ran a 1-run verification batch (`runs/bench/20260325_130933/`) to confirm full log capture from `boot ok` through `bench complete`.
- Ran the full 10-run diagnostic batch: `runs/bench/20260325_131119/`.
- Confirmed `10/10` logs contain all four new attribution lines and end with `bench complete`, with no `fault:` lines.
- Attribution result:
- `semaphore_give_to_taskb_wake`: spikes are fully explained by `SysTick` overlap (`clean_spikes = 0`)
- `tim2_irq_to_task`: mixed result, still has `clean_spikes = 28`
- `queue_wake_latency`: mixed result, still has `clean_spikes = 55`
- `queue_end_to_end_latency`: mixed result, still has `clean_spikes = 55`
- Updated `README.md`, `TESTING.md`, `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`, and `scripts/bench/collect_release_bench.md` with the new result and the narrowed follow-up item (`queue / IRQ` clean spike analysis).

## 2026-03-25 (20260325_140028 clean breakdown rerun completed)
- Extended `src/bench.rs` with phase-level clean spike diagnostics for:
- `tim2_irq_to_task_clean_breakdown`
- `queue_wake_latency_clean_breakdown`
- `queue_end_to_end_latency_clean_breakdown`
- Extended `scripts/bench/collect_release_bench.ps1` to export:
- `clean_breakdown.csv`
- `clean_breakdown_summary.csv`
- Ran a 1-run verification batch (`runs/bench/20260325_135843/`) to confirm the new lines are emitted and parsed.
- Ran the full 10-run diagnostic batch: `runs/bench/20260325_140028/`.
- `clean_breakdown_summary.csv` shows:
- `tim2_irq_to_task`: `clean_spikes_total = 28`, `resume_dominant = 28`, `unblock_dominant = 0`
- `queue_wake_latency`: `clean_spikes_total = 80`, `resume_dominant = 80`, `unblock_dominant = 0`
- `queue_end_to_end_latency`: `clean_spikes_total = 80`, `resume_dominant = 80`, `send / unblock / recv dominant = 0`
- Conclusion: the remaining `queue / IRQ` clean spikes are not caused by `send`, `recv`, or `kernel::unblock()`, but by the shared post-unblock `resume` gap (`PendSV / context_switch / exception return`).
- Updated `README.md`, `TESTING.md`, `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`, and `scripts/bench/collect_release_bench.md` to close the old queue/IRQ clean-spike item and replace it with the narrower common-resume-path follow-up.
- Tests: `cargo build --release --features bench`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM10 -Runs 0`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 1 -ReadTimeoutMs 180000`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 10 -ReadTimeoutMs 180000`

## 2026-03-25 (README cleanup after closing queue/IRQ clean-spike item)
- Removed the `最近完成事项` section from `README.md`.
- Kept only the active follow-up item in `README.md`: common wakeup resume-path tail-latency optimization.
- Tests: docs-only change, no build or hardware rerun.

## 2026-03-25 (20260325_164200 wakeup resume-path optimization completed)
- Optimized the scheduler wakeup path in `src/task/scheduler.rs`:
- idle task no longer enters the ready queue
- idle task no longer participates in time-slice rotation
- This removes unnecessary idle requeue/pop work from `PendSV -> context_switch()` when a blocked task is woken by an interrupt.
- Ran a 1-run verification batch: `runs/bench/20260325_164021/`.
- Ran the full 10-run optimized batch: `runs/bench/20260325_164200/`.
- Compared with `runs/bench/20260325_140028/`:
- `tim2_irq_to_task avg_p50`: `728cy -> 674cy`
- `queue_wake_latency avg_p50`: `725cy -> 663cy`
- `queue_end_to_end_latency avg_p50`: `950cy -> 881cy`
- `tim2_irq_to_task clean_spikes_total`: `28 -> 0`
- `queue_wake_latency clean_spikes_total`: `80 -> 56`
- `queue_end_to_end_latency clean_spikes_total`: `80 -> 61`
- Updated `README.md`, `TESTING.md`, and `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`:
- closed the old common wakeup-path optimization item
- replaced it with the narrower remaining follow-up on queue-path resume-dominant tail spikes
- marked `runs/bench/20260325_164200/` as the current stable baseline for the optimized code
- Tests: `cargo build --release --features bench`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 1 -ReadTimeoutMs 180000`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 10 -ReadTimeoutMs 180000`

## 2026-03-25 (20260325_172035 queue tail convergence completed)
- Added `SyncMsgQueue::send_from_isr()` in `src/ipc/mqueue.rs` as a fast ISR send path.
- Updated the bench queue stage to use `send_from_isr()` inside `TIM2`, avoiding a nested `interrupt::free` on the hot ISR queue-send path.
- Ran a 1-run verification batch: `runs/bench/20260325_171852/`.
- Ran the full 10-run optimized batch: `runs/bench/20260325_172035/`.
- Compared with `runs/bench/20260325_164200/`:
- `queue_end_to_end_latency avg_p50`: `881cy -> 864cy`
- `queue_wake_latency avg_p50`: `663cy -> 663cy`
- `queue_wake_latency clean_spikes_total`: `56 -> 21`
- `queue_end_to_end_latency clean_spikes_total`: `61 -> 21`
- Queue clean spikes now appear only in `run_08`, so the queue tail issue is no longer stably reproducible.
- Also observed low-frequency `tim2_irq_to_task` spikes in `run_03` and `run_06`; this shifts the follow-up from queue-specific tuning to longer-cycle verification of rare tails.
- Updated `README.md`, `TESTING.md`, and `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`:
- closed the queue-tail convergence item
- replaced it with a narrower long-run verification follow-up
- marked `runs/bench/20260325_172035/` as the current stable baseline for the latest code
- Tests: `cargo build --release --features bench`; `cargo build --features bench`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 1 -ReadTimeoutMs 180000`; `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bench/collect_release_bench.ps1 -Port COM6 -Runs 10 -ReadTimeoutMs 180000`

## 2026-03-25 (README todo wording refined)
- Removed the completed `queue / IRQ clean spike` attribution wording from `README.md`'s todo list.
- Kept the completed item only under `最近完成事项`.
- Left a narrower new todo item for shared wakeup-resume-path optimization (`PendSV / context_switch / exception return`).
- Tests: docs-only change

## 2026-03-25 (README clarified completed queue/IRQ item)
- Updated `README.md` to explicitly mark `queue / IRQ clean spike` attribution as completed under `最近完成事项`.
- Kept the narrower unfinished follow-up in `待办事项`: common wakeup resume-path tail-latency tuning.
- Tests: docs-only change; no new build or hardware run.

## 2026-03-25 (20260325_131119 latency attribution rerun completed)
- Fixed `scripts/bench/collect_release_bench.ps1` capture order again: per run it now opens the serial port before `probe-rs reset`, so early boot and semaphore logs are no longer dropped.
- Ran a 1-run verification batch (`runs/bench/20260325_130933/`) to confirm full log capture from `boot ok` through `bench complete`.
- Ran the full 10-run diagnostic batch: `runs/bench/20260325_131119/`.
- Confirmed `10/10` logs contain all four new attribution lines and end with `bench complete`, with no `fault:` lines.
- Attribution result:
- `semaphore_give_to_taskb_wake`: spikes are fully explained by `SysTick` overlap (`clean_spikes = 0`)
- `tim2_irq_to_task`: mixed result, still has `clean_spikes = 28`
- `queue_wake_latency`: mixed result, still has `clean_spikes = 55`
- `queue_end_to_end_latency`: mixed result, still has `clean_spikes = 55`
- Updated `README.md`, `TESTING.md`, `docs/agent/SERIAL_FEEDBACK_ANALYSIS.md`, and `scripts/bench/collect_release_bench.md` with the new result and the narrowed follow-up item (`queue / IRQ` clean spike analysis).

## 2026-03-25 (diagnostics capability completed)
- Added `src/task/diagnostics.rs` and wired task diagnostics into the scheduler/kernel facade.
- Implemented stack watermark support by filling task stacks with a sentinel and reporting `stack_free_low_water_words` / `stack_used_high_water_words`.
- Implemented per-task runtime accounting via `runtime_ticks` updated from `SysTick`.
- Added trace diagnostics: `trace_counters`, `clear_trace_counters`, optional `set_trace_hook`, and `log_diagnostics`.
- Updated `README.md` to document the diagnostic APIs and removed the completed diagnostics todo item.
- Updated `TESTING.md` with `T18` for diagnostic-interface build validation.
- Tests: `cargo build`; `cargo build --features bench`; `cargo build --release --features bench`.

## 2026-03-25 (default UART control app + health/watchdog integration)
- Replaced the old demo `task1/task2` default firmware with a static, interrupt-driven UART control app in `src/app.rs`.
- New default app pipeline:
- `USART2 IRQ -> RX ring buffer -> uart_rx_task -> command queue -> app_cmd_task -> TX queue -> uart_tx_task`
- Added fixed command set:
- `PING`
- `ECHO <text>`
- `LED ON|OFF|TOGGLE`
- `PWM <0-100>`
- `STAT`
- Extended kernel diagnostics/survivability support:
- task heartbeat registration and updates
- `SystemHealth` snapshot
- reset-reason tracking
- conditional watchdog feeding (`feed_watchdog_if_healthy`)
- diagnostics log now prints health + heartbeat state
- Reworked `src/device/uart.rs` into a USART2 service with:
- RX IRQ path
- static RX/TX buffers
- RX/TX events
- UART counters and overflow/error stats
- Updated `src/main.rs` to:
- detect and print reset reason
- configure USART2 + PWM + LED resources
- initialize UART service
- enable `IWDG` in non-bench `release` builds
- create the four default app tasks instead of the old print/blink pair
- Updated release-side docs to reflect the new default APP and new pending hardware validations.
- Files: Cargo.toml, src/app.rs, src/main.rs, src/kernel.rs, src/log.rs, src/device/uart.rs, src/device/pwm.rs, src/ipc/ringbuf.rs, src/task/diagnostics.rs, src/task/tcb.rs, src/task/scheduler.rs, README.md, TESTING.md, docs/agent/CODEX_LOG.md
- Tests: `cargo build`; `cargo build --release`; `cargo build --features bench`; `cargo build --release --features bench`.

## 2026-03-25 (default app board smoke test)
- Flashed the default `release` firmware and validated the new UART control APP on hardware over `COM6`.
- Boot path observed after reset:
- `boot ok (F411)`
- `reset=software`
- `app tasks created: rx=1 cmd=2 tx=3 health=4`
- Command smoke-test results:
- `PING -> PONG`
- `ECHO hello -> hello`
- `LED ON -> OK`
- `LED TOGGLE -> OK`
- `PWM 50 -> OK`
- `STAT` returned uptime, watchdog counters, UART counters, drop counters, and task snapshots
- Health/watchdog observations during the board run:
- `wd=true`
- `feeds` increased during runtime
- `stale=0`
- no UART overflow or command-drop counters observed
- Updated release docs:
- narrowed the README todo from full default-app validation to physical `LED/PWM` output recheck
- marked `T20 = PARTIAL`, `T21 = PASS` in `TESTING.md`
- Tests: `cargo build --release`; `probe-rs list`; `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS`; board-side serial command smoke test on `COM6`.

## 2026-03-25 (default app short soak sample + soak script)
- Added `scripts/test/soak_default_app.ps1` documentation in `scripts/test/soak_default_app.md`.
- Ran a board-level `60s` default-app soak sample on `COM6`:
- command:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Port COM6 -DurationSec 60`
- output:
- `runs/soak/20260325_201735/session.log`
- `runs/soak/20260325_201735/summary.csv`
- `runs/soak/20260325_201735/summary.json`
- Result summary:
- `boot_seen=true`
- `task_banner_seen=true`
- `commands_sent=45`
- `commands_passed=45`
- `commands_failed=0`
- `fault_lines=0`
- `max_stale=0`
- `max_rx_overflow=0`
- `max_tx_overflow=0`
- `max_cmd_drop=0`
- `max_feeds=260`
- Updated release docs:
- `README.md` now documents the soak script and records the `60s` sample directory/result.
- `TESTING.md` adds `T23` for the short soak sample and keeps `T22` as `NOT RUN` until a full `24h` run is completed.

## 2026-03-25 (default app 10min soak sample)
- Ran a longer board-level default-app soak sample on `COM6`:
- command:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Port COM6 -DurationSec 600`
- output:
- `runs/soak/20260325_202530/session.log`
- `runs/soak/20260325_202530/summary.csv`
- `runs/soak/20260325_202530/summary.json`
- Result summary:
- `boot_seen=true`
- `task_banner_seen=true`
- `commands_sent=440`
- `commands_passed=440`
- `commands_failed=0`
- `fault_lines=0`
- `max_stale=0`
- `max_rx_overflow=0`
- `max_tx_overflow=0`
- `max_cmd_drop=0`
- `max_feeds=2440`
- Updated release docs:
- `README.md` now records both the `60s` and `600s` soak samples.
- `TESTING.md` adds `T24` for the `600s` soak sample; `T22` remains `NOT RUN` until a full `24h` soak completes.

## 2026-03-25 (default app log atomicity fix)
- Found a release-side issue during a short soak sample:
- `runs/soak/20260325_212619/` showed `health: ... wd=truePONG ...`, meaning health output and command replies were interleaving on the UART line and could cause false command failures.
- Root cause:
- `src/log.rs` previously forwarded `fmt::Write` chunks directly to the UART queue, so one logical line could be split into multiple enqueues and interleave with other producers.
- Fix:
- `src/log.rs` now uses a line-buffered logger and flushes on newline, restoring line-level atomicity for normal log output.
- `scripts/test/soak_default_app.ps1` already writes `session.log` incrementally, so the regression can be observed directly during runtime.
- Validation:
- `cargo build --release`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Port COM6 -DurationSec 10`
- Regression result:
- `runs/soak/20260325_212839/`
- `commands_sent=8`
- `commands_passed=8`
- `commands_failed=0`
- `fault_lines=0`
- Updated release docs:
- `README.md` documents line-buffered logging as part of the current system strategy.
- `TESTING.md` adds `T25` for the UART line-atomicity regression check.

## 2026-03-27 (developer technical document)
- Added `docs/release/DEVELOPER_GUIDE.md` as a developer-facing technical document for maintainers.
- The new document covers:
- repository/module structure
- startup and execution flow
- kernel facade responsibilities
- scheduler / timeout wheel / diagnostics internals
- timer / sync / IPC / device / driver modules
- default APP task chain and command path
- bench architecture and supporting scripts
- extension guidance and key tuning knobs
- Updated release docs:
- `README.md` now references `docs/release/DEVELOPER_GUIDE.md`
- `TESTING.md` records this as a docs-only update
- Tests: docs-only change; no new build or hardware run.

## 2026-03-27 (user/application guide)
- Added `docs/release/USER_GUIDE.md` as a user-facing and application-facing usage document.
- The new document covers:
- default firmware quick start
- flashing and serial usage
- built-in command set
- application-side development model based on `kernel` facade APIs
- task creation, static stacks, heartbeat registration, sync/IPC/device basics
- how to add commands and background tasks
- diagnostics and soak-oriented usage guidance
- Clarified that “用户态开发” in this project means application-layer development on top of the RTOS APIs, not hardware-enforced user/kernel isolation.
- Updated release docs:
- `README.md` now references `docs/release/USER_GUIDE.md`
- `TESTING.md` records this as a docs-only update
- Tests: docs-only change; no new build or hardware run.

## 2026-03-31 (phase0 stage0 implementation)
- Added `src/app_protocol.rs` to isolate UART line assembly and command parsing from board-specific task code.
- Added `src/ipc/ringbuf_core.rs` and kept `src/ipc/ringbuf.rs` as the IRQ-safe wrapper, so ring buffer core logic can be host-tested.
- Added `host_tests/` plus `scripts/test/run_host_tests.ps1` / `scripts/test/run_host_tests.md`.
- Host tests now cover protocol parsing, sticky/partial/overflow line handling, and ring buffer wrap/full behavior; current result: `5/5 PASS`.
- Extended `scripts/bench/collect_release_bench.ps1` with `-NoFlash` for reset-only long-cycle sampling after one manual bench flash.
- Completed long-cycle bench revalidation at `runs/bench/20260331_143830/`: `30/30` runs reached `bench complete`, `scheduler_o1_check=likely_o1` for all runs.
- Added `scripts/test/start_24h_soak.ps1` / `scripts/test/start_24h_soak.md` to launch default-app soak in a detached hidden PowerShell process with `job.json` metadata and stdout/stderr capture.
- Verified the detached soak launcher with `runs/soak/20260331_150313/` (`5s` smoke sample, summaries generated, `commands_failed=0`).
- Updated release docs: `README.md`, `TESTING.md`, `scripts/bench/collect_release_bench.md`, `scripts/test/soak_default_app.md`.
- Stage0 status after this round: software-side closeout items are implemented; remaining release-side blockers are still the manual `LED/PWM` physical verification and the full `24h` soak acceptance run.

## 2026-03-31 (project context handoff document)
- Added docs/dev/PROJECT_CONTEXT.md as a development-side handoff/onboarding document.
- Purpose: provide a single file that can serve both as a secondary-development reference and as startup context for a new agent.
- Contents include current project stage, firmware modes, code structure, implemented capabilities, current validation state, recommended entry points, engineering rules, and a copy-paste prompt template for new agents.
- Tests: docs-only change; no new build or hardware run.


## 2026-03-31 (stage1 phase1 encapsulation start)
- Started stage1 of the multi-board encapsulation plan with the `board-f411-nucleo` path.
- Added `src/platform/` as the platform facade layer:
- `src/platform/uart.rs`
- `src/platform/watchdog.rs`
- `src/platform/diagnostics.rs`
- `src/platform/controls.rs`
- Added `src/bsp/mod.rs` and `src/bsp/f411_nucleo.rs`.
- F411-specific UART / LED / PWM / watchdog setup was moved out of `src/main.rs` into the F411 BSP.
- `src/main.rs` now only performs boot orchestration, SysTick setup, kernel init, and task creation.
- `src/kernel.rs` no longer stores `stm32f4xx_hal::watchdog::IndependentWatchdog` directly and now depends on the platform facade for watchdog and UART health data.
- `src/app.rs` no longer stores F4 HAL LED/PWM types directly and now uses `platform::controls` for LED/PWM actions.
- `src/device/uart.rs` was reduced to a board-agnostic wrapper over `platform::uart`; early boot output now goes through `platform::uart::boot_write_bytes()`.
- `Cargo.toml` now introduces the explicit `board-f411-nucleo` feature and makes `stm32f4xx-hal` optional.
- `build.rs` now selects `memory/f411-nucleo.x` instead of copying the legacy root `memory.x` directly.
- Added first-stage board scripts:
- `scripts/build/build_board.ps1`
- `scripts/build/flash_board.ps1`
- `scripts/test/run_app_smoke.ps1`
- Added script docs:
- `scripts/build/build_board.md`
- `scripts/build/flash_board.md`
- `scripts/test/run_app_smoke.md`
- Validation completed:
- `cargo build --no-default-features --features board-f411-nucleo --target thumbv7em-none-eabihf`
- `cargo build --no-default-features --features board-f411-nucleo,bench --target thumbv7em-none-eabihf`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
- Script parsing validated for:
- `scripts/build/flash_board.ps1`
- `scripts/test/run_app_smoke.ps1`
- Current scope remains F411-only for execution; `board-f103c8-bluepill` has not been landed yet.

## 2026-03-31 (stage1 follow-up completion)
- Continued stage1 implementation and removed a remaining F4 PAC leak in the architecture helper:
- `src/arch/cortex_m/interrupts.rs` now keeps only Cortex-M generic interrupt helpers (`enable/disable/with_critical_section`) and no longer depends on `stm32f4xx_hal::pac::Interrupt`.
- Re-ran board-script build validation:
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app`
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode app`
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
- Re-ran host tests:
- `scripts/test/run_host_tests.ps1` (`5/5` pass)
- Updated `docs/dev/PROJECT_CONTEXT.md` to reflect the current roadmap shift:
- priority is now multi-board encapsulation first,
- safety refactor is explicitly postponed until after board-extension stabilization.

## 2026-03-31 (stage1 continuation: build-system decoupling + f103 compile-only scaffold)
- Removed board hardcoding from `.cargo/config.toml`:
- dropped `[build].target`
- dropped F411-specific runner
- kept target rustflags and changed bench aliases to explicit board/target builds.
- Added `board-f103c8-bluepill` mechanical onboarding pieces:
- `Cargo.toml` feature: `board-f103c8-bluepill`
- `build.rs` board-memory selector now supports `board-f411-nucleo` and `board-f103c8-bluepill`, with explicit panic on zero/multiple board features.
- added `memory/f103c8-bluepill.x` with conservative memory (`FLASH=64K`, `RAM=20K`).
- Added BSP skeleton for F103 compile-only path:
- `src/bsp/f103c8_bluepill.rs` implementing `BoardContext` and platform hooks as stage-1 stubs.
- `src/bsp/mod.rs` now supports both board features and enforces exactly one board feature.
- Reduced board coupling blockers for F103 compile-only:
- `src/device/mod.rs` now gates `adc` and `timer` to `board-f411-nucleo`.
- `src/timer/mod.rs` now gates `hw_timer` to `board-f411-nucleo`.
- `src/device/gpio.rs`, `src/device/pwm.rs`, `src/driver/motor.rs`, `src/driver/sensor.rs` switched to direct `embedded-hal` trait imports.
- Extended board scripts to recognize F103 profile:
- `scripts/build/build_board.ps1`
- `scripts/build/flash_board.ps1`
- `scripts/test/run_app_smoke.ps1`
- Updated script docs:
- `scripts/build/build_board.md`
- `scripts/build/flash_board.md`
- `scripts/test/run_app_smoke.md`
- Validation completed:
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app`
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode bench`
- `scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile debug -Mode app`
- `scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app`
- `scripts/test/run_host_tests.ps1` (`5/5` pass)
- Updated docs:
- `README.md`
- `TESTING.md`
- `docs/dev/PROJECT_CONTEXT.md`

## 2026-04-02 (stage1 continuation: F103 runtime BSP + RC board path)
- Upgraded `src/bsp/f103c8_bluepill.rs` from compile-only stubs to runtime implementation:
- board init now configures `USART1/USART2` pinmux and peripheral clocks
- boot banner now prints `MSP/PSP/VTOR/PendSV/SysTick`
- UART runtime path now includes RX interrupt ingest (`USART1` + `USART2`), RX/TX ring buffers, events, and stats
- LED control (`PC13`, active-low) and PWM output (`TIM1 CH1 / PA8`) were wired into `platform::controls`
- watchdog start/feed path was added for F103 via `IWDG` register path
- Added `stm32f1` optional dependency and connected `board-f103c8-bluepill` feature to it.
- Enabled `cortex-m` feature `critical-section-single-core` to satisfy `critical-section` linkage on `thumbv7m`.
- Extended board scripts for current hardware:
- `scripts/build/build_board.ps1` / `scripts/build/flash_board.ps1` / `scripts/test/run_app_smoke.ps1` now accept `f103rct6-generic` alias (maps to `board-f103c8-bluepill` feature, chip `STM32F103RC` for flash/smoke).
- Fixed `run_app_smoke.ps1` serial assembly load path (`System.IO.Ports`) for this host PowerShell environment.
- Validation executed:
- `scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile debug -Mode app` (PASS)
- `scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app` (PASS)
- `scripts/build/build_board.ps1 -Board f103rct6-generic -Profile release -Mode app` (PASS)
- `scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter` (PASS)
- `scripts/test/run_host_tests.ps1` (`5/5` PASS)
- Current hardware status:
- `runs/smoke/20260402_181040_f103rct6-generic/`, `runs/smoke/20260402_182236_f103rct6-generic/`, and `runs/smoke/20260402_182608_f103rct6-generic/` (with `-Flash`) all showed no boot banner and no command response on `COM14`; marked as ongoing UART/link mapping issue pending board-side confirmation.
- Updated release docs:
- `README.md` (TODO + F103RCT6 commands + current acceptance focus)
- `TESTING.md` (added T33/T34/T35 and latest results)

## 2026-04-02 (uart link debug mode and manual serial tool)
- Added `uart-probe` feature mode for quick board-level UART link debugging without entering RTOS/app scheduler path.
- Added `src/uart_probe.rs`:
- minimal startup banner
- continuous heartbeat print (`uart probe heartbeat`)
- line echo path (`rx: ...`) via interrupt-fed RX ring
- Updated `src/main.rs`:
- new feature-gated `uart-probe` entry
- compile-time guard to forbid `bench + uart-probe`
- `app` modules/stack sections are excluded in `uart-probe` mode
- Updated `scripts/build/build_board.ps1` to support `-Mode uart-probe`.
- Added root helper script `serial_io_test.ps1` for manual COM Tx/Rx verification.
- Validation executed:
- `scripts/build/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe` (PASS)
- `scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter` (PASS)
- Serial verification step deferred to user run because local check hit `COM14` occupied (`Access denied`).
- Updated docs:
- `README.md` (added UART probe and `serial_io_test.ps1` usage)
- `TESTING.md` (added T36 with current status)

## 2026-04-02 (uart-probe LED blink support)
- Updated `src/uart_probe.rs`:
- added periodic LED toggle in probe loop (`current::controls::toggle_led()`).
- retained UART probe heartbeat and echo behavior.
- Rebuilt and flashed UART probe image for `f103rct6-generic`:
- build: `runs/build/20260402_190535_f103rct6-generic_release_uart-probe/`
- flash: `runs/flash/20260402_190547_f103rct6-generic/`
- Updated docs:
- `README.md` (uart-probe expected LED behavior)
- `TESTING.md` (added T37 LED validation status)

## 2026-04-02 (F103 board-photo alignment: UART multi-route probe)
- Referencing the provided STM32F103RCT6 board pin map, updated F103 BSP UART probe path to cover multiple common serial routes on this board family.
- `src/bsp/f103c8_bluepill.rs` updates:
- enabled and configured `USART3` (`PB10/PB11`) in addition to existing `USART1` (`PA9/PA10`) and `USART2` (`PA2/PA3`)
- RX IRQ path now listens to `USART1/USART2/USART3`
- boot/probe TX now broadcasts to all three UARTs to detect actual board USB-UART wiring
- Rebuilt and reflashed UART probe:
- build: `runs/build/20260402_190943_f103rct6-generic_release_uart-probe/`
- flash: `runs/flash/20260402_190954_f103rct6-generic/`
- Updated docs:
- `README.md` (uart-probe now documents `USART1/2/3` parallel probing)
- `TESTING.md` (T36 updated to latest artifacts and criteria)

## 2026-04-02 (F103 LED behavior follow-up)
- Addressed user observation (`D2` always on, `D1` always off) by extending F103 LED control path:
- `src/bsp/f103c8_bluepill.rs` now toggles two candidate onboard LED lines in probe/control path:
  - primary: `PC13` (active-low)
  - alternate: `PA1` (active-high on some RCT6 boards)
- Rebuilt and reflashed UART probe:
- build: `runs/build/20260402_191506_f103rct6-generic_release_uart-probe/`
- flash: `runs/flash/20260402_191516_f103rct6-generic/`
- Updated docs:
- `README.md` (clarified LED does not blink during flashing itself)
- `TESTING.md` (T37 updated to latest partial status)

## 2026-04-02 (uart-probe startup-path simplification)
- Reworked `uart-probe` into a standalone minimal path:
- `src/uart_probe.rs` now uses direct registers only and avoids RTOS/kernel/static scheduler data during probe mode.
- probe responsibilities: boot banner + UART echo + heartbeat + dual-candidate LED blink (`PC13`/`PA1`) + optional watchdog feed.
- Updated `src/main.rs` module gating:
- when `uart-probe` is enabled, RTOS/app modules are not compiled into this path.
- added link stub `__cortexos_switch_context` for shared asm object compatibility in probe mode.
- Validation:
- build pass: `runs/build/20260402_195231_f103rct6-generic_release_uart-probe/`
- `f103` app and `f411` bench regression builds remain pass.
- Flash status in this environment:
- automatic `flash_board.ps1` failed to open probe with `USB error: reset not supported by WinUSB` (tool/driver state issue), pending manual reconnect/retry.
- Updated docs:
- `README.md` (uart-probe purpose clarified as startup-path diagnostic mode)
- `TESTING.md` (T36 updated to latest artifact/status)

## 2026-04-02 (F103 uart-probe recovery: linker memory selection + live serial recheck)
- Root cause fixed: linker was still picking legacy root `memory.x` (`RAM=128K`) for `thumbv7m` image, causing invalid F103 stack top (`0x20020000`) and early startup instability.
- Changes:
- removed legacy root `memory.x` from repo root to avoid board-script shadowing.
- normalized board linker scripts to UTF-8 no BOM and ensured both export `__STACK_START`:
  - `memory/f411-nucleo.x`
  - `memory/f103c8-bluepill.x`
- tuned `src/uart_probe.rs` liveness cadence:
  - heartbeat threshold `6_000_000 -> 200_000`
  - blink threshold `3_000_000 -> 100_000`
  so serial tools opened after boot can still observe output quickly.
- Validation:
- `scripts/build/build_board.ps1 -Board f103rct6-generic -Profile release -Mode uart-probe` (PASS, latest: `runs/build/20260402_202644_f103rct6-generic_release_uart-probe/`)
- `scripts/build/flash_board.ps1 -Board f103rct6-generic -Image target/thumbv7m-none-eabi/release/CortexOS -ResetAfter` (PASS, latest: `runs/flash/20260402_202302_f103rct6-generic/`)
- serial capture on `COM14 @ 115200` now confirms:
  - boot banner on reset: `boot ok (F103)` ...
  - periodic `uart probe heartbeat`
  - Tx/Rx echo: `hello -> rx: hello`
- Cross-board regression:
- `scripts/build/build_board.ps1 -Board f411-nucleo -Profile debug -Mode app` (PASS)
- Docs updated:
- `README.md` (uart-probe heartbeat expectation clarified)
- `TESTING.md` (T36 marked PASS with latest evidence)

## 2026-04-04 (agent handoff package)
- Added `docs/agent/AGENT_HANDOFF.md` as a full project handoff document for new-agent onboarding.
- Contents include:
- current project stage and board profiles
- architecture/module map
- build/flash/test entry commands
- verification snapshot and open gaps
- phased multi-board plan (P0/P1 + deferred safety phase)
- direct prompt block for new agent startup
- This document is development-context oriented and does not change release-facing behavior.

## 2026-04-04 (AGENT_HANDOFF expanded, full-detail version)
- Rewrote `docs/agent/AGENT_HANDOFF.md` into a comprehensive handoff package for new-agent onboarding.
- Added full sections for:
- project stage and priorities
- board profiles (F411/F103 + alias)
- module-by-module implementation status
- layered architecture and dependency rules
- script contracts and command matrix
- F103 troubleshooting conclusions
- current verification snapshot and open gaps
- step-by-step execution checklist
- phased roadmap and startup prompt template
- Re-saved file as UTF-8 to avoid prior Chinese mojibake display issues.

## 2026-04-10 (next step: multiboard regression entry + smoke reset sequencing)
- Continued to next phase after user confirmed F103 tests passed.
- Updated `scripts/test/run_app_smoke.ps1` sequencing:
  - in `-Flash` mode now uses `flash -> open serial -> probe-rs reset -> capture`, instead of resetting before serial opens.
  - added params: `-ProbeSpeed`, `-ResetBeforeCapture`.
  - purpose: reduce startup banner loss during smoke capture.
- Added multiboard regression script:
  - `scripts/test/run_multiboard_regression.ps1`
  - `scripts/test/run_multiboard_regression.md`
  - scope: F411 debug/release app + F103 release app + optional F411 bench + optional smoke (F103/F411 ports)
  - output: `runs/regression/<timestamp>/meta.json + summary.csv/json + per-step logs`.
- Adjusted F103 debug policy in regression:
  - `f103 debug` moved to optional (`-IncludeF103Debug`), default skipped due conservative `FLASH=64K` frequently overflowing.
- Validation:
  - `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench`
  - result: `runs/regression/20260410_165750/` with `pass=4 fail=0 skip=3`.
- Docs updated:
  - `README.md` (P0 F103 acceptance marked completed, added multiboard regression usage, clarified F103 debug optional)
  - `TESTING.md` (added T38; T35 marked PASS by user 2026-04-10 feedback)
  - `scripts/test/run_app_smoke.md` (new reset sequencing and params)

## 2026-04-10 (PowerShell bool arg compatibility fix for multiboard regression)
- User reported `run_multiboard_regression.ps1 -FlashOnSmoke $true` failing with parameter conversion error (`System.String -> System.Boolean`).
- Root cause: in this host PowerShell invocation pattern (`powershell -File ...`), bool script parameter binding is inconsistent.
- Fix:
- changed `run_multiboard_regression.ps1` `-FlashOnSmoke` from `[bool]` to `[string]` and added `Parse-BoolArg` helper.
- now accepts `true/false`, `$true/$false`, `1/0`, `yes/no`, `on/off`.
- Updated docs/examples:
- `README.md` switched example to `-FlashOnSmoke true`.
- `scripts/test/run_multiboard_regression.md` examples and parameter description updated.
- Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke -F103Port COM14 -FlashOnSmoke true`
- PASS, output `runs/regression/20260410_170900/`.
- `TESTING.md` T38 updated with this compatibility recheck sample.

## 2026-04-10 (multiboard regression robustness: probe-aware flash fallback)
- Added `-AutoDisableFlashWhenProbeMissing` to `scripts/test/run_multiboard_regression.ps1`.
- Behavior:
- when `-FlashOnSmoke true` and probe is missing, script auto-forces `FlashOnSmoke=false` for this run and records a skip note instead of hard-failing at flash stage.
- Added `Probe-Available` precheck using `probe-rs list` parsing.
- Docs updated:
- `scripts/test/run_multiboard_regression.md` (new flag and usage example)
- `README.md` (fallback command example)
- Validation run:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -F103Port COM14 -FlashOnSmoke true -AutoDisableFlashWhenProbeMissing`
- precheck recorded `probe detected` in `runs/regression/20260410_182054/summary.csv`.
- this run failed at `smoke_f103_app` due host serial contention: `Access to the port 'COM14' is denied` (not build/flash failure).

## 2026-04-14 (F103 smoke stability fix + multiboard regression pass)
- Root-cause fixes for F103 smoke instability:
- `src/bsp/f103c8_bluepill.rs` and `src/bsp/f411_nucleo.rs`: fixed USART IRQ RX/error handling to avoid double `DR` reads when error bits and `RXNE` coexist.
- `src/app.rs`: adjusted default APP priorities to `cmd(1) > rx(2) > tx(3) > health(4)` to prevent command responses from lagging behind service tasks.
- Smoke script robustness:
- `scripts/test/run_app_smoke.ps1`: normalize serial input lines with `TrimEnd(\"\\r\", \"\\n\")`, removing CRLF-related false negative matches.
- Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_app_smoke.ps1 -Board f103rct6-generic -Port COM16 -ReadTimeoutMs 4000 -StartupWindowMs 3000 -Flash`
- PASS: `runs/smoke/20260414_145748_f103rct6-generic/` (`5/5` commands passed).
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -F103Port COM16 -FlashOnSmoke true`
- PASS: `runs/regression/20260414_145802/` (`pass=4 fail=0 skip=2`).

## 2026-04-14 (script generalization for future multi-board growth)
- Added central board profile registry:
- `scripts/config/board_profiles.json`
- Added shared parser library:
- `scripts/lib/board_profiles.ps1`
- Migrated scripts to board-profile-driven resolution:
- `scripts/build/build_board.ps1`
- `scripts/build/flash_board.ps1`
- `scripts/test/run_app_smoke.ps1`
- `scripts/test/run_multiboard_regression.ps1`
- Added explicit multi-probe routing:
- `flash_board.ps1` new `-Probe` (maps to `probe-rs --probe`)
- `run_app_smoke.ps1` new `-Probe` (flash/reset both honor it)
- `run_multiboard_regression.ps1` new `-SmokeBoardProbes` plus backward-compatible `-F103Probe/-F411Probe`
- Added generalized smoke target mapping:
- `-SmokeBoardPorts` accepts `board:COMx` / `board=COMx`, with comma/semicolon split support
- Added generalized build matrix:
- `-BuildMatrix` accepts `board:profile:mode[:required|optional]`
- Docs updated:
- `scripts/test/run_multiboard_regression.md` rewritten in UTF-8 and aligned to new parameters
- `scripts/build/flash_board.md`, `scripts/test/run_app_smoke.md`, `scripts/build/build_board.md`, `README.md`
- Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench`
- PASS: `runs/regression/20260414_155345/`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SmokeBoardPorts "f103rct6-generic:COM17" -SmokeBoardProbes "f103rct6-generic:0483:3748" -FlashOnSmoke true`
- PASS: `runs/regression/20260414_155252/`

## 2026-04-14 (board onboarding docs + build script generalization)
- `build.rs` generalized:
- no longer hardcoded to F411/F103 pair.
- auto-detects active `CARGO_FEATURE_BOARD_*`, enforces exactly one board feature, and maps to `memory/<board>.x`.
- added explicit panic when memory script is missing.
- Added board onboarding document:
- `docs/release/BOARD_PORTING_GUIDE.md` (step-by-step board addition + validation workflow).
- README updated with `docs/release/BOARD_PORTING_GUIDE.md` entry.
- Regression recheck:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f411-nucleo -Profile release -Mode app` PASS
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/build_board.ps1 -Board f103c8-bluepill -Profile release -Mode app` PASS
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke -IncludeBench` PASS (`runs/regression/20260414_160139/`)

## 2026-04-14 (continue: board onboarding scaffold)
- Added board scaffold script:
- `scripts/build/new_board_scaffold.ps1`
- Added script doc:
- `scripts/build/new_board_scaffold.md`
- Added integration notes into:
- `docs/release/BOARD_PORTING_GUIDE.md`
- `README.md`
- Purpose:
- provide a consistent bootstrap path for future board additions (memory script + BSP skeleton + optional board_profiles registration)
- Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build/new_board_scaffold.ps1 -Board demo-m3-board -Chip STM32F103RC -Target thumbv7m-none-eabi -RegisterInBoardProfiles -DryRun` (PASS)
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_multiboard_regression.ps1 -SkipSmoke` (PASS, `runs/regression/20260414_161915/`)

## 2026-04-14 (continue: resume original plan with soak-path generalization)
- Generalized soak scripts for multi-board execution:
- `scripts/test/soak_default_app.ps1` now supports `-Board`, `-Probe`, `-NoReset` and resolves default `chip/binary` from `board_profiles`.
- command expectations for `LED/PWM` accept both `OK` and `ERR *_unavailable` for portable board capability checks.
- boot detection changed to generic pattern `^boot ok \(`.
- `scripts/test/start_24h_soak.ps1` now supports `-Board`, `-Probe`, `-NoReset` and forwards parameters to soak runner.
- Docs updated:
- `scripts/test/soak_default_app.md`
- `scripts/test/start_24h_soak.md`
- `README.md`
- Validation:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f103rct6-generic -Port COM17 -Probe 0483:3748 -DurationSec 20 -ReadSliceMs 1800` -> PASS (`runs/soak/20260414_163043/`)
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f411-nucleo -Port COM6 -NoFlash -NoReset -DurationSec 20` -> PASS (`runs/soak/20260414_163002/`)
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/start_24h_soak.ps1 -Board f103rct6-generic -Port COM17 -DurationSec 5 -NoFlash -NoReset` -> PASS (`runs/soak/20260414_163411/`)

## 2026-04-14 (continue: targeted F103 soak stabilization)
- Context:
- user requested targeted fixes first; defer next 1h revalidation.
- Implemented code-side stabilization:
- `src/app.rs`:
- added F103 cooperative burst limits (`RX_BURST_MAX`, `CMD_BURST_MAX`, `TX_BURST_DRAIN_THRESHOLD`).
- adjusted F103 task priorities to `health(1) > cmd(2) > rx(3) > tx(4)` to prevent health-task starvation.
- `src/bsp/f103c8_bluepill.rs` / `src/bsp/f411_nucleo.rs`:
- TX drain changed to chunked mode (max 64 bytes per drain call).
- USART IRQ error handling updated to count error but keep current `RXNE` byte (avoid extra command truncation).
- `src/bsp/f103c8_bluepill.rs`:
- RX pins switched to pull-up input (PA10, and PA3/PB11 in `uart-probe`) to reduce idle-line noise sensitivity.
- Validation:
- F103 smoke with probe:
- `runs/smoke/20260414_175656_f103rct6-generic` PASS
- `runs/smoke/20260414_175710_f103rct6-generic` PASS
- `runs/smoke/20260414_175728_f103rct6-generic` PASS
- F103 180s soak:
- `runs/soak/20260414_175751` -> `max_stale=0`, `fault_lines=0`, `commands_failed=4`.
- Cross-board compile regression:
- `runs/regression/20260414_180140/` PASS (`pass=3 fail=0 skip=2`).

## 2026-04-15 (final validation closure)
- Added round progress tracker:
- `docs/agent/CURRENT_ROUND_PROGRESS.md`
- Completed `F103` formal `1h soak`:
- `runs/soak/20260415_131037/`
- result: `commands_sent=8458`, `commands_failed=6`, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`
- Completed repeated `F103` `1h soak`:
- `runs/soak/20260415_141355/`
- result: `commands_sent=8490`, `commands_failed=3`, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`
- failure signature narrowed to rare UART corruption (`ERR unknown`, truncated command text), not scheduler/kernel instability
- Verified `F411` serial smoke without probe:
- `runs/smoke/20260415_141156_f411-nucleo/` PASS
- Verified `F411` short soak:
- `runs/soak/20260415_141207/` PASS (`136/136`, `fault_lines=0`)
- Completed `F411` formal `1h soak`:
- `runs/soak/20260415_2411_f411_1h/`
- result: `commands_sent=8088`, `commands_failed=0`, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`
- Updated release/test documents:
- `README.md`
- `TESTING.md`
- `docs/agent/CURRENT_ROUND_PROGRESS.md`

## 2026-04-15 (F103 UART quick hardening)
- Problem targeted:
- rare `F103` UART RX corruption could still surface as `ERR unknown`, truncated commands, or single-command match failures during long runs
- Fast fix implemented:
- `src/app_protocol.rs`
- added `LineAssembler::drop_current_line()`
- `src/app.rs`
- on `board-f103c8-bluepill`, `uart_rx_task` now monitors `uart::stats().rx_errors`
- if the hardware RX error counter advances while assembling a line, the current line is dropped instead of being parsed
- Verification:
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/run_app_smoke.ps1 -Board f103rct6-generic -Port COM18 -Probe 0483:3748 -Flash`
- PASS: `runs/smoke/20260415_185129_f103rct6-generic/`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test/soak_default_app.ps1 -Board f103rct6-generic -Port COM18 -Probe 0483:3748 -NoFlash -DurationSec 180 -RunId 20260415_f103_uart_fix_180s`
- PASS: `runs/soak/20260415_f103_uart_fix_180s/` (`426/426`, `fault_lines=0`, `error_lines=0`)



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
- Files: src/sync/mutex.rs, src/sync/event.rs, src/sync/semaphore.rs, src/sync/mod.rs, src/task/scheduler.rs, src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (kernel)
- Implemented kernel facade APIs for scheduler and timers.
- Added kernel module declaration and updated README.
- Files: src/kernel.rs, src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (kernel integration)
- Wired kernel facade into `main` initialization flow.
- Updated README and test log.
- Files: src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-03 (rules)
- Added `Prompt` file to capture collaboration rules and documented the update in README/TESTING.
- Files: Prompt, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (docs/rules update only).

## 2026-03-03 (coding rules)
- Expanded `Prompt` with project coding rules and updated README.
- Files: Prompt, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (docs update only).

## 2026-03-03 (rules rename)
- Renamed `Prompt` to `rules.md` and added `Prompt.md` summary.
- Updated README and test log.
- Files: rules.md, Prompt.md, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (docs update only).

## 2026-03-03 (app features)
- Added UART print and LED blink tasks in `app.rs` and wired them into `main`.
- Initialized logger in `main` and configured PA5 LED.
- Updated README and test log.
- Files: src/app.rs, src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`.

## 2026-03-04 (flash attempt)
- Attempted `cargo run` (probe-rs flash) for STM32F411RETx; failed with `JtagNoDeviceConnected` at 1000 kHz.
- Updated README/TESTING with the attempt record.
- Files: README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo run` (failed to open probe).

## 2026-03-04 (flash fix)
- Diagnosed the probe link and confirmed the target is reachable at `100 kHz` SWD.
- Updated `.cargo/config.toml` runner speed from `1000` to `100` and documented the workaround.
- Files: .cargo/config.toml, README.md, CODEX_LOG.md, TESTING.md
- Tests: `probe-rs info --chip STM32F411RETx --protocol swd --speed 100`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`.

## 2026-03-04 (flash verification)
- Verified low-speed SWD flashing with `probe-rs reset` after download.
- Confirmed `cargo run` leaves a long-lived `probe-rs` session that must be terminated before reopening the probe.
- Files: README.md, CODEX_LOG.md, TESTING.md
- Tests: `probe-rs reset --chip STM32F411RETx --protocol swd --speed 100 --non-interactive`, `cargo run` (timed out in host session, left probe occupied until process cleanup).

## 2026-03-04 (rtos bench)
- Added a feature-gated benchmark firmware (`bench`) to test context switch, sleep wakeup, and TIM2 IRQ-to-task latency.
- Kept the default application path unchanged; benchmark logic is isolated behind `--features bench`.
- Added formatted logger access for benchmark metric output.
- Files: Cargo.toml, src/log.rs, src/bench.rs, src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`.

## 2026-03-04 (bench fix)
- Hardened the benchmark state machine after the first metric could complete while later stages stalled.
- Lowered the helper task priority after the context-switch test, added edge-wait timeout logs, and removed the IRQ block/unblock race with a critical section.
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`.

## 2026-03-04 (bench recheck)
- Reworked the post-context-switch transition again: helper task no longer gets deleted/reprioritized, and is instead parked into a blocked state before the sleep/IRQ benchmarks start.
- Added explicit expected UART progress markers (`bench: helper parked`, then `bench: sleep-wakeup start`) and documented that manual `probe-rs download` must be preceded by `cargo build --features bench` to avoid flashing a non-bench image from the shared debug path.
- Flashed the updated bench image with `probe-rs download --verify`; a follow-up `probe-rs reset` hit `os error 5` because the probe was re-occupied immediately after download.
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`

## 2026-03-04 (bench stack)
- Rechecked the unchanged UART output and moved the likely fault source to bench task stack pressure rather than the stage handoff itself.
- Increased bench-only task stacks in `src/main.rs` from `256/256` words to `1024/512` words, keeping the default application path unchanged.
- Rebuilt both targets and reflashed the bench image with `probe-rs download --verify`.
- Files: src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build`, `cargo build --features bench`, `probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target\\thumbv7em-none-eabihf\\debug\\CortexOS`

## 2026-03-10 (release bench path)
- Added explicit release benchmark path aliases in `.cargo/config.toml`: `bench-dev`, `bench-release`, `bench-release-build`.
- Bench UART init line now includes build profile (`debug` or `release`) to avoid flashing/measurement confusion.
- Documented stable release flashing flow using `target/thumbv7em-none-eabihf/release/CortexOS`.
- Files: .cargo/config.toml, src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo bench-release-build`.

## 2026-03-10 (bench anomaly fix)
- Fixed context benchmark over-count by preventing updates when `CTX_LEFT == 0` and clearing late `CTX_PENDING` after the context benchmark loop.
- Added bench-only idle behavior switch: while `STAGE_SLEEP` is active, idle no longer executes `WFI`, avoiding `DWT::CYCCNT` freeze and removing the false `sleep_1tick_extra=0` result.
- Kept default firmware behavior unchanged; the `WFI` bypass is compiled only with feature `bench`.
- Files: src/bench.rs, src/task/scheduler.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (sleep metric rework)
- Reworked `sleep_1tick_extra` measurement to use wakeup latency from the recorded SysTick edge (`now - last_systick_edge_cycle`) instead of `elapsed - 1ms` saturating subtraction.
- Added bench SysTick edge timestamp hook in `src/task/context.rs` and `bench::on_systick_edge()`.
- This addresses release runs that incorrectly reported `sleep_1tick_extra = 0` across all samples.
- Files: src/bench.rs, src/task/context.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (sleep metric rework v2)
- Fixed `sleep_1tick_extra` false +1 tick offset (~1000us) by changing the benchmark model:
- Snapshot SysTick edge cycle *before* `sleep_ms(1)`, then measure `resume - start_edge - one_tick`.
- Hardened sleep/timeout arming against SysTick races by moving `now/wake_tick` computation into scheduler critical sections.
- Files: src/bench.rs, src/task/scheduler.rs, README.md, CODEX_LOG.md, TESTING.md
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
- Files: src/bench.rs, src/task/scheduler.rs, src/main.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`, `cargo build`.

## 2026-03-10 (bench semaphore park race fix)
- Fixed race between helper park flag and actual block transition in benchmark helper task.
- In `STAGE_PARK_HELPER`, helper now sets `TASK_B_PARKED` and calls `block_current(None)` inside one critical section (`interrupt::free`), preventing false park-ack observations.
- This targets UART failures:
- `bench:semaphore_waiter timeout sample=0`
- `bench: helper re-park timeout`
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (bench sleep zero fix)
- Fixed `sleep_1tick_extra` reporting all-zero in release runs.
- Changed sleep metric in `src/bench.rs` to measure direct wake latency from latest SysTick edge after wakeup (`now - last_systick_edge_cycle`) under a critical-section snapshot.
- Removed one-tick saturating subtraction model for this metric.
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
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
- Kept change history in `CODEX_LOG.md` and test records in `TESTING.md`.
- Files: README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-10 (README parameter split)
- Updated `README.md` parameter section to split tuning into:
- kernel parameter tuning
- benchmark/test parameter tuning
- Added concrete knobs for each category (tick rate, time slice, MAX_TASKS, bench samples, bench timer Hz, scaling cases, profile path).
- Files: README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-10 (queue metrics split)
- Split queue benchmark into two explicit metrics in `src/bench.rs`:
- `queue_wake_latency`: wake path latency from ISR wake stamp to task resume.
- `queue_end_to_end_latency`: ISR enqueue stamp to task `recv` completion.
- Replaced previous single output `queue_isr_send_to_task_recv` with the two new lines.
- Added queue metric description to `README.md` test section.
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
- Tests: `cargo build --features bench`, `cargo build --release --features bench`.

## 2026-03-10 (bench samples 1000)
- Increased benchmark sample count from `64` to `1000` in `src/bench.rs` (`BENCH_SAMPLES`).
- Updated README parameter section to document current default sample count.
- Files: src/bench.rs, README.md, CODEX_LOG.md, TESTING.md
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
- Files: TESTING.md, README.md, CODEX_LOG.md
- Result: all three builds passed (warnings unchanged).

## 2026-03-13 (Prompt.md full rewrite)
- Replaced `Prompt.md` with a complete project collaboration prompt based on current repository context.
- New prompt includes: environment, hard rules, current bench status, test acceptance points, workflow, output style, and document responsibilities.
- Updated `README.md` document convention section to include `Prompt.md` role.
- Files: Prompt.md, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (development plan doc)
- Added `DEVELOPMENT_PLAN.md` as the dedicated discussion document for future roadmap, priorities, milestones, and open questions.
- Updated `README.md` document convention section to include the new plan document.
- Files: DEVELOPMENT_PLAN.md, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

## 2026-03-17 (scheduler O(1) plan detail)
- Expanded `DEVELOPMENT_PLAN.md` with a dedicated scheduler section covering:
- current scheduler strategy
- current design issues
- concrete O(1) migration tasks
- suggested implementation order
- acceptance criteria
- Updated `README.md` TODO entry to point readers to the detailed plan document.
- Files: DEVELOPMENT_PLAN.md, README.md, CODEX_LOG.md, TESTING.md
- Tests: not run (documentation-only update).

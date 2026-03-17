Rules
1. After every update, record changes in README.md, CODEX_LOG.md, and TESTING.md.
2. The project is `no_std`/`no_main`; avoid `std` and OS APIs, use `core`/HAL instead.
3. Avoid heap allocation; prefer fixed-size arrays and `const fn` initializers.
4. Shared data across interrupts must use `cortex_m::interrupt::Mutex<RefCell<...>>` and `interrupt::free`.
5. Task stacks must be 8-byte aligned; maintain alignment when constructing stack frames.
6. SysTick base tick is 1ms (`TICK_HZ = 1000`); update scheduler/timer code if changed.
7. Soft-timer callbacks run in task context via pending queue, not directly in ISRs.
8. Keep ISRs short; defer heavy work to task context or PendSV.
9. Use atomics with `Ordering::Relaxed` unless stronger ordering is required and documented.
10. Prefer the `kernel` facade for scheduling/timer entry points in application code.

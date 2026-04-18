# Current Round Progress

## Goal
- Close the remaining validation work for the multi-board packaging stage.
- Produce a complete board-level validation snapshot for `F103 + F411`.
- Sync the final conclusions back into the release documents.

## Completed In This Round
- `F103` formal `1h soak` completed:
- `runs/soak/20260415_131037/`
- `8458/8452` commands passed, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`
- `F103` repeated `1h soak` completed:
- `runs/soak/20260415_141355/`
- `8490/8487` commands passed, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`
- `F411` smoke re-validation completed:
- `runs/smoke/20260415_141156_f411-nucleo/`
- `F411` short soak completed:
- `runs/soak/20260415_141207/`
- `136/136` commands passed, `fault_lines=0`
- `F411` formal `1h soak` completed:
- `runs/soak/20260415_2411_f411_1h/`
- `8088/8088` commands passed, `fault_lines=0`, `max_stale=0`, `max_cmd_drop=0`

## Current Assessment
- Multi-board packaging phase is functionally closed.
- `F411` path is stable on the current image and serial link.
- `F103` path is stable at the RTOS/system level:
- no fault
- no abnormal reset
- no stale task
- no overflow or queue growth
- Remaining issue is limited to very low-frequency UART corruption on `F103`, not scheduler/kernel instability.

## Remaining Follow-up
- Optional, non-blocking:
- Improve `F103` UART anti-noise robustness to reduce rare `ERR unknown` / truncated command cases during very long runs.
- Optional, later stage:
- Full `24h soak` if hardware availability allows.

## Quick UART Hardening
- `2026-04-15`: added a fast mitigation for `F103` UART noise handling.
- Strategy:
- when `rx_errors` increases during line assembly, the current command line is discarded until the next newline instead of being forwarded into parsing.
- Validation:
- `runs/smoke/20260415_185129_f103rct6-generic/` PASS
- `runs/soak/20260415_f103_uart_fix_180s/` PASS
- `commands_sent=426`
- `commands_failed=0`
- `fault_lines=0`
- `error_lines=0`

## Execution Log
- `2026-04-15`: created this file and started final validation.
- `2026-04-15`: completed two `F103 1h soak` samples and one `F411 1h soak` sample.
- `2026-04-15`: updated `README.md`, `TESTING.md`, and `docs/agent/CODEX_LOG.md`.
- `2026-04-15`: completed repository document/layout refactor:
- moved release/dev/agent/data documents under `docs/`
- moved smoke/soak/bench/build/flash/regression artifacts under `runs/`
- regrouped scripts into `scripts/build` / `scripts/test` / `scripts/bench` / `scripts/config` / `scripts/lib`
- added top-level `DEV_LOG.md` and `TEST_LOG.md`
- rewrote release-facing `README.md` and `TESTING.md`
- validated new entrypoints with `build_board.ps1`, `run_host_tests.ps1`, and `run_multiboard_regression.ps1 -SkipSmoke`


use core::fmt::Write;
use core::str;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::app_protocol::{
    self, LedCommand, LineAssembler, LineAssemblerEvent, ParseError, ParsedCommand,
};
use crate::device::uart;
use crate::ipc::mqueue::{MsgQueueError, SyncMsgQueue};
use crate::kernel;
use crate::log;
use crate::platform;
use crate::sync::{
    mutex::IrqMutex,
};

const MAX_LINE_LEN: usize = app_protocol::MAX_LINE_LEN;
const CMD_POOL_DEPTH: usize = 8;
#[cfg(feature = "board-f103c8-bluepill")]
const HEARTBEAT_TIMEOUT_MS: u32 = 60_000;
#[cfg(not(feature = "board-f103c8-bluepill"))]
const HEARTBEAT_TIMEOUT_MS: u32 = 1_000;
#[cfg(feature = "board-f103c8-bluepill")]
const WAIT_TIMEOUT: Option<u32> = None;
#[cfg(not(feature = "board-f103c8-bluepill"))]
const WAIT_TIMEOUT: Option<u32> = Some(200);
const HEALTH_PERIOD_MS: u32 = 250;
const HEALTH_REPORT_MS: u32 = 5_000;
const STACK_WARNING_WORDS: usize = 64;

#[derive(Clone, Copy)]
struct CommandSlot {
    in_use: bool,
    len: usize,
    bytes: [u8; MAX_LINE_LEN],
}

impl CommandSlot {
    const fn new() -> Self {
        Self {
            in_use: false,
            len: 0,
            bytes: [0; MAX_LINE_LEN],
        }
    }
}

static CMD_POOL: IrqMutex<[CommandSlot; CMD_POOL_DEPTH]> =
    IrqMutex::new([const { CommandSlot::new() }; CMD_POOL_DEPTH]);
static CMD_QUEUE: SyncMsgQueue<CMD_POOL_DEPTH> = SyncMsgQueue::new();

static LINE_DROPS: AtomicU32 = AtomicU32::new(0);
static CMD_DROPS: AtomicU32 = AtomicU32::new(0);

pub fn create_default_tasks(
    rx_stack: &'static mut [u32],
    cmd_stack: &'static mut [u32],
    tx_stack: &'static mut [u32],
    health_stack: &'static mut [u32],
) -> Option<[usize; 4]> {
    // Keep command handling ahead of service tasks so line replies are not
    // delayed behind UART TX draining or periodic health reporting.
    let cmd_pid = kernel::create_task(app_cmd_task, 0, cmd_stack, 1)?;
    let rx_pid = kernel::create_task(uart_rx_task, 0, rx_stack, 2)?;
    let tx_pid = kernel::create_task(uart_tx_task, 0, tx_stack, 3)?;
    let health_pid = kernel::create_task(health_task, 0, health_stack, 4)?;

    let _ = kernel::register_task_heartbeat(rx_pid, HEARTBEAT_TIMEOUT_MS);
    let _ = kernel::register_task_heartbeat(cmd_pid, HEARTBEAT_TIMEOUT_MS);
    let _ = kernel::register_task_heartbeat(tx_pid, HEARTBEAT_TIMEOUT_MS);
    let _ = kernel::register_task_heartbeat(health_pid, HEARTBEAT_TIMEOUT_MS);

    Some([rx_pid, cmd_pid, tx_pid, health_pid])
}

fn uart_rx_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(HEARTBEAT_TIMEOUT_MS);
    let mut assembler = LineAssembler::<MAX_LINE_LEN>::new();

    loop {
        #[cfg(feature = "board-f103c8-bluepill")]
        {
            let mut had_rx = false;
            while let Some(byte) = uart::read_byte() {
                had_rx = true;
                match assembler.push_byte(byte) {
                    LineAssemblerEvent::None => {}
                    LineAssemblerEvent::Dropped => {
                        LINE_DROPS.fetch_add(1, Ordering::Relaxed);
                    }
                    LineAssemblerEvent::Line(line) => {
                        if !enqueue_command(line.as_bytes()) {
                            CMD_DROPS.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
            if !had_rx {
                kernel::sleep_ms(1);
            }
        }

        #[cfg(not(feature = "board-f103c8-bluepill"))]
        {
            let _ = uart::wait_for_rx(WAIT_TIMEOUT);
            uart::clear_rx_event();

            while let Some(byte) = uart::read_byte() {
                match assembler.push_byte(byte) {
                    LineAssemblerEvent::None => {}
                    LineAssemblerEvent::Dropped => {
                        LINE_DROPS.fetch_add(1, Ordering::Relaxed);
                    }
                    LineAssemblerEvent::Line(line) => {
                        if !enqueue_command(line.as_bytes()) {
                            CMD_DROPS.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        }

        let _ = kernel::task_heartbeat();
    }
}

fn app_cmd_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(HEARTBEAT_TIMEOUT_MS);
    let mut local = [0u8; MAX_LINE_LEN];

    loop {
        let mut did_work = false;
        while let Some(slot_idx) = CMD_QUEUE.recv() {
            did_work = true;
            if let Some(len) = take_command(slot_idx, &mut local) {
                handle_command(&local[..len]);
            }
        }

        if !did_work {
            kernel::sleep_ms(1);
        }

        let _ = kernel::task_heartbeat();
    }
}

fn uart_tx_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(HEARTBEAT_TIMEOUT_MS);

    loop {
        #[cfg(feature = "board-f103c8-bluepill")]
        {
            if uart::drain_tx() == 0 {
                kernel::sleep_ms(1);
            }
        }

        #[cfg(not(feature = "board-f103c8-bluepill"))]
        {
            let _ = uart::wait_for_tx(WAIT_TIMEOUT);
            uart::clear_tx_event();
            let _ = uart::drain_tx();
        }

        let _ = kernel::task_heartbeat();
    }
}

fn health_task(_arg: usize) -> ! {
    let _ = kernel::register_current_heartbeat(HEARTBEAT_TIMEOUT_MS);
    let mut last_report = kernel::now_ticks();
    let mut last_diag = kernel::now_ticks();

    loop {
        kernel::sleep_ms(HEALTH_PERIOD_MS);
        let _ = kernel::task_heartbeat();
        let _ = kernel::feed_watchdog_if_healthy();

        let now = kernel::now_ticks();
        let health = kernel::system_health();

        if now.wrapping_sub(last_report) >= HEALTH_REPORT_MS {
            last_report = now;
            let queue_len = CMD_QUEUE.len();
            app_log_line(format_args!(
                "health: uptime={}ms reset={} wd={} feeds={} live={} hb={} stale={} stack_warn={} rx={} tx={} rxov={} txov={} rxerr={} line_drop={} cmd_drop={} queue={}",
                kernel::now_ms(),
                health.reset_reason.as_str(),
                health.watchdog_enabled,
                health.watchdog_feeds,
                health.live_tasks,
                health.registered_heartbeats,
                health.stale_tasks,
                health.stack_warning_tasks,
                health.uart_rx_bytes,
                health.uart_tx_bytes,
                health.uart_rx_overflows,
                health.uart_tx_overflows,
                health.uart_rx_errors,
                LINE_DROPS.load(Ordering::Relaxed),
                CMD_DROPS.load(Ordering::Relaxed),
                queue_len,
            ));

            if queue_len > 0 {
                let mut task_ids = [usize::MAX; kernel::MAX_TASKS];
                let count = kernel::list_tasks(&mut task_ids);
                for &pid in task_ids.iter().take(count) {
                    if let Some(task) = kernel::task_diagnostics(pid) {
                        app_log_line(format_args!(
                            "health_task pid={} state={:?} prio={} wake={} timeout={} hb_age={}",
                            task.pid,
                            task.state,
                            task.priority,
                            task.wake_tick,
                            task.has_timeout,
                            task.heartbeat_age_ticks,
                        ));
                    }
                }
            }
        }

        if (health.stale_tasks != 0 || health.stack_warning_tasks != 0)
            && now.wrapping_sub(last_diag) >= HEALTH_REPORT_MS
        {
            last_diag = now;
            kernel::log_diagnostics();
        }
    }
}

fn enqueue_command(bytes: &[u8]) -> bool {
    let Some(slot_idx) = alloc_command_slot(bytes) else {
        return false;
    };

    match CMD_QUEUE.send(slot_idx) {
        Ok(()) => {
            true
        }
        Err(MsgQueueError::Full) => {
            free_command_slot(slot_idx);
            false
        }
    }
}

fn alloc_command_slot(bytes: &[u8]) -> Option<usize> {
    CMD_POOL.lock(|pool| {
        for (idx, slot) in pool.iter_mut().enumerate() {
            if !slot.in_use {
                slot.in_use = true;
                slot.len = bytes.len();
                slot.bytes[..bytes.len()].copy_from_slice(bytes);
                return Some(idx);
            }
        }
        None
    })
}

fn take_command(slot_idx: usize, out: &mut [u8; MAX_LINE_LEN]) -> Option<usize> {
    CMD_POOL.lock(|pool| {
        let slot = pool.get_mut(slot_idx)?;
        if !slot.in_use {
            return None;
        }

        let len = slot.len.min(MAX_LINE_LEN);
        out[..len].copy_from_slice(&slot.bytes[..len]);
        slot.in_use = false;
        slot.len = 0;
        Some(len)
    })
}

fn free_command_slot(slot_idx: usize) {
    CMD_POOL.lock(|pool| {
        if let Some(slot) = pool.get_mut(slot_idx) {
            slot.in_use = false;
            slot.len = 0;
        }
    });
}

fn handle_command(line: &[u8]) {
    let Ok(text) = str::from_utf8(line) else {
        app_log_line(format_args!("ERR utf8"));
        return;
    };

    match app_protocol::parse_command(text) {
        Ok(ParsedCommand::Ping) => app_log_line(format_args!("PONG")),
        Ok(ParsedCommand::Echo(arg)) => app_log_line(format_args!("{}", arg)),
        Ok(ParsedCommand::Led(cmd)) => handle_led_command(cmd),
        Ok(ParsedCommand::Pwm(percent)) => handle_pwm_command(percent),
        Ok(ParsedCommand::Stat) => emit_stat_report(),
        Err(ParseError::Empty) => {}
        Err(ParseError::Unknown) => app_log_line(format_args!("ERR unknown")),
        Err(ParseError::InvalidLed) => app_log_line(format_args!("ERR led")),
        Err(ParseError::InvalidPwm) => app_log_line(format_args!("ERR pwm")),
    }
}

fn handle_led_command(command: LedCommand) {
    let ok = match command {
        LedCommand::On => platform::controls::set_led(true),
        LedCommand::Off => platform::controls::set_led(false),
        LedCommand::Toggle => platform::controls::toggle_led(),
    };

    if !ok {
        app_log_line(format_args!("ERR led_unavailable"));
    } else {
        app_log_line(format_args!("OK"));
    }
}

fn handle_pwm_command(percent: u8) {
    if !platform::controls::set_pwm_percent(percent) {
        app_log_line(format_args!("ERR pwm_unavailable"));
    } else {
        app_log_line(format_args!("OK"));
    }
}

fn emit_stat_report() {
    let health = kernel::system_health();
    let uart_stats = uart::stats();

    app_log_line(format_args!(
        "STAT uptime={}ms reset={} wd={} feeds={} live={} hb={} stale={} stack_warn={} rx={} tx={} rxov={} txov={} rxerr={} rx_pending={} tx_pending={} line_drop={} cmd_drop={} queue={}",
        kernel::now_ms(),
        health.reset_reason.as_str(),
        health.watchdog_enabled,
        health.watchdog_feeds,
        health.live_tasks,
        health.registered_heartbeats,
        health.stale_tasks,
        health.stack_warning_tasks,
        health.uart_rx_bytes,
        health.uart_tx_bytes,
        health.uart_rx_overflows,
        health.uart_tx_overflows,
        health.uart_rx_errors,
        uart_stats.rx_pending,
        uart_stats.tx_pending,
        LINE_DROPS.load(Ordering::Relaxed),
        CMD_DROPS.load(Ordering::Relaxed),
        CMD_QUEUE.len(),
    ));

    let mut task_ids = [usize::MAX; kernel::MAX_TASKS];
    let count = kernel::list_tasks(&mut task_ids);
    for &pid in task_ids.iter().take(count) {
        if let Some(task) = kernel::task_diagnostics(pid) {
            let stack_warn = task.stack_free_low_water_words <= STACK_WARNING_WORDS;
            app_log_line(format_args!(
                "TASK pid={} state={:?} prio={}/{} runtime={} stack_used={}/{}w stack_free_low={}w hb={} age={} timeout={} stale={} warn={}",
                task.pid,
                task.state,
                task.priority,
                task.base_priority,
                task.runtime_ticks,
                task.stack_used_high_water_words,
                task.stack_size_words,
                task.stack_free_low_water_words,
                task.heartbeat_registered,
                task.heartbeat_age_ticks,
                task.heartbeat_timeout_ticks,
                task.heartbeat_stale,
                stack_warn,
            ));
        }
    }
}

fn app_log_line(args: core::fmt::Arguments<'_>) {
    let _ = log::with_logger(|tx| {
        let _ = tx.write_fmt(args);
        let _ = tx.write_str("\r\n");
    });
}

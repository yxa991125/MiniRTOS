use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use cortex_m::interrupt::free;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use crate::task::tcb::{Tcb, TaskState};
use crate::timer::{soft_timer, systick};

#[cfg(feature = "bench")]
pub const MAX_TASKS: usize = 40;
#[cfg(not(feature = "bench"))]
pub const MAX_TASKS: usize = 8;
pub const MAX_PRIORITY: usize = 8;
pub const DEFAULT_TIME_SLICE_TICKS: u32 = 10;
const INVALID_PID: usize = usize::MAX;
const IDLE_PID: usize = 0;
const IDLE_STACK_WORDS: usize = 128;
const IDLE_PRIORITY: u8 = (MAX_PRIORITY - 1) as u8;

#[repr(align(8))]
struct AlignedStack<const N: usize>([u32; N]);

static mut IDLE_STACK: AlignedStack<IDLE_STACK_WORDS> = AlignedStack([0; IDLE_STACK_WORDS]);

static TASKS: Mutex<RefCell<[Option<Tcb>; MAX_TASKS]>> =
    Mutex::new(RefCell::new([const{None}; MAX_TASKS]));

/* // Ready set represented as a bitmap of priorities.
static READY_SET: Mutex<RefCell<[bool; MAX_PRIORITY]>> =
    Mutex::new(RefCell::new([false; MAX_PRIORITY])); */
static READY_COUNTS: Mutex<RefCell<[u8; MAX_PRIORITY]>> =
    Mutex::new(RefCell::new([0; MAX_PRIORITY]));
static READY_MASK: AtomicU32 = AtomicU32::new(0);

/* static CURRENT_PID: Mutex<RefCell<usize>> =
    Mutex::new(RefCell::new(0)); */

static CURRENT_PID: AtomicUsize = AtomicUsize::new(INVALID_PID);

static SCHED_STARTED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    fn __start_first_task(sp: *mut u32) -> !;
}

/// Initialize task storage, ready set, and reset current pid.
pub fn init() {
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        tasks.iter_mut().for_each(|slot| *slot = None);

/*         READY_SET
            .borrow(cs)
            .borrow_mut()
            .iter_mut()
            .for_each(|flag| *flag = false); */
        ready_counts.iter_mut().for_each(|count| *count = 0);
        READY_MASK.store(0, Ordering::Relaxed);

        // *CURRENT_PID.borrow(cs).borrow_mut() = 0;
        CURRENT_PID.store(INVALID_PID, Ordering::Relaxed);

        init_idle_task(&mut tasks, &mut ready_counts);
    });
}

pub fn current_pid() -> Option<usize> {
    let pid = CURRENT_PID.load(Ordering::Relaxed);
    if pid == INVALID_PID {
        None
    } else {
        Some(pid)
    }
}

/// Create a task, place it into the task table, and mark it ready.
pub fn create_task(
    entry: fn(usize) -> !,
    arg: usize,
    stack: &'static mut [u32],
    priority: u8,
) -> Option<usize> {
    if priority as usize >= MAX_PRIORITY {
        return None;
    }
    
    let mut created_pid = None;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        if let Some((pid, slot)) = tasks
            .iter_mut()
            .enumerate()
            .find(|(_, task)| task.is_none())
        {
            // Stack grows downwards: end points to the current top (empty stack).
            let stack_start = stack.as_mut_ptr();
            let stack_end = unsafe { stack_start.add(stack.len()) };
            let sp = init_stack_frame(stack_start, stack_end, entry, arg);

            *slot = Some(Tcb::init(
                pid,
                sp,
                priority,
                DEFAULT_TIME_SLICE_TICKS,
                stack_start,
                stack_end,
                entry,
                arg,
            ));
            ready_inc(&mut ready_counts, priority);

/*             if let Some(flag) = READY_SET
                .borrow(cs)
                .borrow_mut()
                .get_mut(priority as usize)
            {
                *flag = true;
            } */
            created_pid = Some(pid);
        }
    });

    created_pid
}

pub fn tick() {
    let now = systick::now();
    tick_at(now);
}

pub fn tick_at(now: u32) {
    if !SCHED_STARTED.load(Ordering::Relaxed) {
        return;
    }

    let mut pend_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        // Wake sleeping tasks or blocked tasks with timeout.
        for slot in tasks.iter_mut() {
            let Some(task) = slot.as_mut() else {
                continue;
            };

            match task.state {
                TaskState::Sleeping => {
                    if time_after_eq(now, task.wake_tick) {
                        task.state = TaskState::Ready;
                        task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                        ready_inc(&mut ready_counts, task.priority);
                        pend_switch = true;
                    }
                }
                TaskState::Blocked => {
                    if task.has_timeout && time_after_eq(now, task.wake_tick) {
                        task.state = TaskState::Ready;
                        task.has_timeout = false;
                        task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                        ready_inc(&mut ready_counts, task.priority);
                        pend_switch = true;
                    }
                }
                _ => {}
            }
        }

        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            if current.state == TaskState::Running {
                if current.remaining_slice > 0 {
                    current.remaining_slice -= 1;
                }

                if current.remaining_slice == 0 {
                    current.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                    current.state = TaskState::Ready;
                    ready_inc(&mut ready_counts, current.priority);
                    pend_switch = true;
                }

                if !pend_switch {
                    if let Some(best_prio) = highest_ready_priority(READY_MASK.load(Ordering::Relaxed)) {
                        if best_prio < current.priority {
                            current.state = TaskState::Ready;
                            ready_inc(&mut ready_counts, current.priority);
                            pend_switch = true;
                        }
                    }
                }
            } else {
                pend_switch = true;
            }
        } else {
            pend_switch = true;
        }
    });

    if pend_switch {
        cortex_m::peripheral::SCB::set_pendsv();
    }
}

/// 启动第一个任务：选择一个 Ready 任务，切到 PSP，并进入 Thread mode 执行该任务。
pub fn start_first_task() -> ! {
    // 从任务表里选出的首任务 sp（指向软件帧起点 R4..R11）
    let mut first_sp: *mut u32 = core::ptr::null_mut();

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        // 选择优先级最高（priority 数值最小）的 Ready 任务
        let mut best: Option<(usize, u8, *mut u32)> = None;

        for (pid, slot) in tasks.iter_mut().enumerate() {
            if let Some(t) = slot.as_mut() {
                if t.state == TaskState::Ready {
                    match best {
                        None => best = Some((pid, t.priority, t.sp)),
                        Some((_, best_prio, _)) if t.priority < best_prio => {
                            best = Some((pid, t.priority, t.sp))
                        }
                        _ => {}
                    }
                }
            }
        }

        let (pid, _prio, sp) = best.expect("no ready task to start");

        // 标记为 Running
        if let Some(t) = tasks[pid].as_mut() {
            if t.state == TaskState::Ready {
                ready_dec(&mut ready_counts, t.priority);
            }
            t.state = TaskState::Running;
            if t.remaining_slice == 0 {
                t.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
            }
        }

        // 设置当前 pid（此处在临界区里，Relaxed 足够）
        CURRENT_PID.store(pid, Ordering::Relaxed);

        first_sp = sp;
    });

    // ---- 临界区外：做少量 sanity check，避免直接 HardFault ----
    if first_sp.is_null() {
        loop { cortex_m::asm::bkpt(); }
    }

    // 8-byte 对齐检查：AAPCS + Cortex-M 异常入栈要求
    if (first_sp as usize & 0x7) != 0 {
        loop { cortex_m::asm::bkpt(); }
    }

    unsafe {
        // first_sp 指向软件帧起点（R4..R11 共 8 words）
        // 硬件帧起点在 sw 之上 8 words
        let hw = first_sp.add(8);

        let pc = hw.add(6).read();    // 异常返回后将跳转的 PC
        let xpsr = hw.add(7).read();  // xPSR，要求 T-bit = 1

        // 关键不变量：Thumb 位 + xPSR 的 T-bit
        let pc_thumb = (pc & 1) == 1;
        let xpsr_tbit = (xpsr & 0x0100_0000) != 0;

        if !(pc_thumb && xpsr_tbit) {
            // 在这里停住，用调试器看 pc/xpsr/栈内容
            loop { cortex_m::asm::bkpt(); }
        }

        // 标记调度器已启动：允许 SysTick 触发 PendSV
        SCHED_STARTED.store(true, Ordering::Relaxed);

        // 跳入首任务：该函数应通过 SVC/异常返回方式切 PSP 并进入 Thread mode
        __start_first_task(first_sp)
    }
}

/// Save current task context, pick the next ready task, and return its stack pointer.
pub fn context_switch(save_sp: *mut u32) -> *mut u32 {
    let mut next_sp: *mut u32 = ptr::null_mut();

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();
        // let current_pid = *CURRENT_PID.borrow(cs).borrow();
        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        let mut fallback: Option<(usize, u8, *mut u32)> = None;

        // Save current task context and mark it ready.
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            fallback = Some((current_pid, current.priority, save_sp));
            current.sp = save_sp;
            if current.state == TaskState::Running {
                current.state = TaskState::Ready;
                ready_inc(&mut ready_counts, current.priority);
            }
/*             if let Some(flag) = READY_SET
                .borrow(cs)
                .borrow_mut()
                .get_mut(current.priority as usize)
            {
                *flag = true;
            } */
        }

        let (pid, _prio, sp) = pick_next_ready(&mut tasks, current_pid)
            .or(fallback)
            .unwrap_or((current_pid, 0, ptr::null_mut()));
        
        if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
            if task.state == TaskState::Ready {
                ready_dec(&mut ready_counts, task.priority);
            }
            task.state = TaskState::Running;
            if task.remaining_slice == 0 {
                task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
            }
        }
        // *CURRENT_PID.borrow(cs).borrow_mut() = pid;
        CURRENT_PID.store(pid, Ordering::Relaxed);

/*         if let Some(flag) = READY_SET
            .borrow(cs)
            .borrow_mut()
            .get_mut(prio as usize)
        {
            *flag = false;
        } */

        next_sp = sp;
    });

    next_sp
}

pub fn sleep_ms(ms: u32) {
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            let now = systick::now();
            let wake = now.wrapping_add(systick::ms_to_ticks(ms));
            current.state = TaskState::Sleeping;
            current.wake_tick = wake;
            current.has_timeout = false;
        }
    });

    cortex_m::peripheral::SCB::set_pendsv();
}

pub fn block_current(timeout_ms: Option<u32>) {
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        if let Some(current) = tasks.get_mut(current_pid).and_then(|t| t.as_mut()) {
            let (has_timeout, wake_tick) = if let Some(ms) = timeout_ms {
                let now = systick::now();
                (true, now.wrapping_add(systick::ms_to_ticks(ms)))
            } else {
                (false, 0)
            };

            current.state = TaskState::Blocked;
            current.has_timeout = has_timeout;
            current.wake_tick = wake_tick;
        }
    });

    cortex_m::peripheral::SCB::set_pendsv();
}

pub fn unblock(pid: usize) -> bool {
    let mut unblocked = false;
    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();
        if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
            if task.state != TaskState::Ready {
                task.state = TaskState::Ready;
                task.has_timeout = false;
                task.wake_tick = 0;
                task.remaining_slice = DEFAULT_TIME_SLICE_TICKS;
                ready_inc(&mut ready_counts, task.priority);
                unblocked = true;
            }
        }
    });

    if unblocked && SCHED_STARTED.load(Ordering::Relaxed) {
        cortex_m::peripheral::SCB::set_pendsv();
    }

    unblocked
}

pub fn delete_task(pid: usize) -> bool {
    if pid == IDLE_PID {
        return false;
    }

    let mut removed = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        if let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) {
            match task.state {
                TaskState::Ready => ready_dec(&mut ready_counts, task.priority),
                TaskState::Running => need_switch = true,
                _ => {}
            }

            tasks[pid] = None;
            removed = true;

            if CURRENT_PID.load(Ordering::Relaxed) == pid {
                CURRENT_PID.store(INVALID_PID, Ordering::Relaxed);
                need_switch = true;
            }
        }
    });

    if need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        cortex_m::peripheral::SCB::set_pendsv();
    }

    removed
}

pub fn exit_current() -> ! {
    let pid = CURRENT_PID.load(Ordering::Relaxed);
    let _ = delete_task(pid);
    cortex_m::peripheral::SCB::set_pendsv();
    loop {
        cortex_m::asm::wfi();
    }
}

pub fn set_priority(pid: usize, new_prio: u8) -> bool {
    if new_prio as usize >= MAX_PRIORITY || pid == IDLE_PID {
        return false;
    }

    let mut updated = false;
    let mut need_switch = false;

    free(|cs| {
        let mut tasks = TASKS.borrow(cs).borrow_mut();
        let mut ready_counts = READY_COUNTS.borrow(cs).borrow_mut();

        let Some(task) = tasks.get_mut(pid).and_then(|t| t.as_mut()) else {
            return;
        };

        let old_prio = task.priority;
        if old_prio == new_prio {
            updated = true;
            return;
        }

        if task.state == TaskState::Ready {
            ready_dec(&mut ready_counts, old_prio);
            ready_inc(&mut ready_counts, new_prio);
        }

        task.priority = new_prio;
        updated = true;

        let current_pid = CURRENT_PID.load(Ordering::Relaxed);
        if current_pid != pid {
            if let Some(current) = tasks.get(current_pid).and_then(|t| t.as_ref()) {
                if current.state == TaskState::Running && new_prio < current.priority {
                    need_switch = true;
                }
            }
        } else if let Some(best_prio) = highest_ready_priority(READY_MASK.load(Ordering::Relaxed)) {
            if best_prio < new_prio {
                need_switch = true;
            }
        }
    });

    if updated && need_switch && SCHED_STARTED.load(Ordering::Relaxed) {
        cortex_m::peripheral::SCB::set_pendsv();
    }

    updated
}

fn pick_next_ready(
    tasks: &mut [Option<Tcb>; MAX_TASKS],
    current_pid: usize,
) -> Option<(usize, u8, *mut u32)> {
    let best_prio = highest_ready_priority(READY_MASK.load(Ordering::Relaxed))?;
    let start = if current_pid == INVALID_PID {
        0
    } else {
        (current_pid + 1) % MAX_TASKS
    };

    for i in 0..MAX_TASKS {
        let idx = (start + i) % MAX_TASKS;
        if let Some(task) = tasks[idx].as_ref() {
            if task.state == TaskState::Ready && task.priority == best_prio {
                return Some((idx, task.priority, task.sp));
            }
        }
    }

    None
}

#[inline]
fn time_after_eq(now: u32, target: u32) -> bool {
    now.wrapping_sub(target) < 0x8000_0000
}

#[inline]
fn ready_inc(ready_counts: &mut [u8; MAX_PRIORITY], prio: u8) {
    let idx = prio as usize;
    if idx >= MAX_PRIORITY {
        return;
    }
    ready_counts[idx] = ready_counts[idx].saturating_add(1);
    if ready_counts[idx] == 1 {
        READY_MASK.fetch_or(1u32 << idx, Ordering::Relaxed);
    }
}

#[inline]
fn ready_dec(ready_counts: &mut [u8; MAX_PRIORITY], prio: u8) {
    let idx = prio as usize;
    if idx >= MAX_PRIORITY {
        return;
    }
    if ready_counts[idx] > 0 {
        ready_counts[idx] -= 1;
        if ready_counts[idx] == 0 {
            READY_MASK.fetch_and(!(1u32 << idx), Ordering::Relaxed);
        }
    }
}

#[inline]
fn highest_ready_priority(mask: u32) -> Option<u8> {
    for prio in 0..MAX_PRIORITY {
        if (mask & (1u32 << prio)) != 0 {
            return Some(prio as u8);
        }
    }
    None
}

fn init_idle_task(tasks: &mut [Option<Tcb>; MAX_TASKS], ready_counts: &mut [u8; MAX_PRIORITY]) {
    let stack = unsafe {
        let stack_ptr = core::ptr::addr_of_mut!(IDLE_STACK.0) as *mut u32;
        core::slice::from_raw_parts_mut(stack_ptr, IDLE_STACK_WORDS)
    };

    let stack_start = stack.as_mut_ptr();
    let stack_end = unsafe { stack_start.add(stack.len()) };
    let sp = init_stack_frame(stack_start, stack_end, idle_task, 0);

    tasks[IDLE_PID] = Some(Tcb::init(
        IDLE_PID,
        sp,
        IDLE_PRIORITY,
        DEFAULT_TIME_SLICE_TICKS,
        stack_start,
        stack_end,
        idle_task,
        0,
    ));
    ready_inc(ready_counts, IDLE_PRIORITY);
}

fn idle_task(_arg: usize) -> ! {
    loop {
        soft_timer::dispatch();
        #[cfg(feature = "bench")]
        {
            if crate::bench::idle_allows_wfi() {
                cortex_m::asm::wfi();
            } else {
                cortex_m::asm::nop();
            }
        }
        #[cfg(not(feature = "bench"))]
        {
            cortex_m::asm::wfi();
        }
    }
}

const INITIAL_XPSR: u32 = 0x0100_0000;

#[inline(never)]
fn task_exit_trap() -> ! {
    exit_current()
}

fn init_stack_frame(
    stack_start: *mut u32,
    stack_end: *mut u32,
    entry: fn(usize) -> !,
    arg: usize,
) -> *mut u32 {
    let mut sp = stack_end as usize;
    sp &= !0x7;
    let sp = sp as *mut u32;

    let pc = (entry as usize as u32) | 1;
    let lr = (task_exit_trap as usize as u32) | 1;

    unsafe {
        // 1) 先为“硬件帧”预留 8 words（R0..xPSR），放在靠近 stack_end 的位置
        let hw = sp.sub(8);
        let sw = hw.sub(8);

        if sw < stack_start {
            loop { cortex_m::asm::bkpt(); }
        }

        hw.add(0).write(arg as u32);                      // R0
        hw.add(1).write(0);                               // R1
        hw.add(2).write(0);                               // R2
        hw.add(3).write(0);                               // R3
        hw.add(4).write(0);                               // R12
        hw.add(5).write(lr);                              // LR
        hw.add(6).write(pc);                              // PC  (必须是 0x0800_xxxx)
        hw.add(7).write(INITIAL_XPSR);                    // xPSR (必须是 0x0100_0000)

        // 2) 再为“软件帧”预留 8 words（R4..R11），放在更低地址处
        let sw = hw.sub(8);
        for i in 0..8 {
            sw.add(i).write(0);
        }

        // 返回 sp(sw)：boot.S 会先 pop R4..R11，然后 PSP 正好指向 hw(R0)
        sw
    }
}

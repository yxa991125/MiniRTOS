# 调度器 O(1) 改造记录

## 1. 目标
- 将就绪任务选择路径从“扫描任务表”升级为 O(1) 风格的选择方式。
- 在保持调度器对外 API 基本稳定的前提下，替换内部就绪集合与超时管理的数据结构。

## 2. 改造前设计
- `READY_MASK` 用于记录哪些优先级上存在 Ready 任务。
- 实际选任务时，`pick_next_ready` 仍然会线性扫描整个 `TASKS` 表。
- `start_first_task` 也依赖扫描任务表来找到第一个 Ready 任务。
- 同优先级轮转依赖 PID 顺序，而不是显式的 FIFO ready queue。
- `tick_at` 对 `Sleeping/Blocked` 的超时唤醒依赖全表扫描，因此 tick 路径是 O(n)。

## 3. 主要改动
- 第一阶段：为每个优先级引入显式 ready queue。
- 为 `Tcb` 扩展以下 ready 字段：
- `ready_prev`
- `ready_next`
- `in_ready_queue`
- 将原先只统计数量的 ready 管理方式替换为基于队列的辅助函数：
- `ready_push_back`
- `ready_remove`
- `ready_pop_highest`
- `highest_ready_priority` 直接基于 ready bitmap 进行优先级查找，使最高优先级选择路径达到 O(1) 风格。
- 第二阶段：引入 timeout wheel，替换 `tick_at` 的全表超时扫描。
- 为 `Tcb` 扩展以下 timeout 字段：
- `timeout_prev`
- `timeout_next`
- `in_timeout_queue`
- `timeout_rounds`
- 新增超时管理辅助函数：
- `timeout_push`
- `timeout_remove`
- `process_timeout_slot`

## 4. 已修改的调度路径
- `init`
- 重置任务表和 ready queue
- 重置 timeout wheel
- 重新构建 idle task，并将其作为 Ready 任务入队
- `create_task`
- 创建任务
- 按优先级直接插入对应的 ready queue
- `start_first_task`
- 不再扫描任务表
- 直接从最高优先级 ready queue 取出任务启动
- `context_switch`
- 如果当前任务仍然可运行，则显式重新入队
- 直接从 ready queue 中取下一个任务
- `tick_at`
- 按 tick 推进 timeout wheel，仅处理当前槽位上的超时任务
- 将到期的睡眠/超时任务转入 ready queue
- 在时间片耗尽或被更高优先级任务抢占时，把当前任务移到队尾
- `sleep_ms`
- 不再依赖后续 tick 的全表扫描
- 当前任务进入 timeout wheel，等待到期唤醒
- `block_current`
- 带超时阻塞的任务进入 timeout wheel
- `unblock`
- 任务恢复可运行时，先从 timeout wheel 移除，再转入 ready queue
- `delete_task`
- 如果任务处于 ready queue 或 timeout wheel 中，先移出再删除
- `set_priority`
- 从旧优先级 ready queue 移除
- 更新优先级
- 重新插入新优先级 ready queue

## 5. 行为影响
- 就绪任务选择不再依赖对 `TASKS` 的线性扫描。
- 同优先级调度现在遵循显式的 FIFO ready queue。
- 超时唤醒不再在每个 tick 上扫描整个任务表，而是按 timeout wheel 当前槽位处理。
- bench 中的 `scheduler_scale` 和 `scheduler_o1_check` 不再混入任务表扫描带来的伪差异。

## 6. 已执行验证
- `cargo build`
- `cargo build --features bench`
- `cargo build --release --features bench`

## 7. 当前剩余限制
- timeout wheel 当前按槽位链表组织，单个槽位内仍可能出现 O(k) 遍历，其中 `k` 为该槽位任务数。
- 如果 `tick_at` 因长时间关中断而一次性追赶多个 tick，仍会按丢失的 tick 数逐步推进 wheel。
- 下一步更适合补齐的是跨桶、跨轮、长延时、tick wrap-around 场景的硬件回归测试，而不是继续改 ready 选择路径。

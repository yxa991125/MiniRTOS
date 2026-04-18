# 串口反馈数据分析与后续安排

## 1. 文档目的
- 基于已有硬件 bench 数据给出结构化分析。
- 说明当前 RTOS 的稳定性、性能基线与后续工作重点。

## 2. 数据来源
当前主要参考六批数据：
- 当前稳定基线：`runs/bench/20260325_172035/`
- 旧版稳定基线：`runs/bench/20260324_190953/`
- 旧版基础基线：`runs/bench/20260324_155928/`
- 首轮 mutex / PI 实跑问题目录：`runs/bench/20260324_185313/`
- `IrqMutex` 归因诊断目录：`runs/bench/20260324_222048/`
- `queue / IRQ / semaphore` 归因诊断目录：`runs/bench/20260325_131119/`
- `queue / IRQ` clean breakdown 诊断目录：`runs/bench/20260325_140028/`
- 公共唤醒恢复路径优化回归：`runs/bench/20260325_164200/`
- 队列 ISR 快速发送路径回归：`runs/bench/20260325_172035/`

关键文件：
- `runs/bench/20260325_172035/baseline_summary.csv`
- `runs/bench/20260324_222048/mutex_lock_attribution_summary.csv`
- `runs/bench/20260325_131119/latency_attribution_summary.csv`
- `runs/bench/20260325_140028/clean_breakdown_summary.csv`

## 3. 当前总判断
- `20260325_172035` 已成为当前代码版本的稳定 10 轮硬件基线。
- ready queue、timeout wheel、阻塞式 mutex / priority inheritance 与采集脚本扩展都已完成硬件闭环。
- 通过把 idle 任务移出 ready queue，并让 idle 不参与时间片轮转，再加上队列 ISR 快速发送路径，`queue` clean spike 已经不再稳定复现。
- 当前问题重心已经从“功能缺口”转为“长期复验稀有长尾、长期基线与后续调优”。

## 4. 稳定基线结论（`runs/bench/20260325_172035/`）
- `10/10` 轮都出现：`boot ok (F411)`、`bench complete`、`scheduler_o1_check ... verdict=likely_o1`
- `context_switch_a_to_b`：`avg_p50 = 426cy`，`sample_p50 = 424cy`
- `tim2_irq_to_task`：`avg_p50 = 668cy`
- `queue_wake_latency`：`avg_p50 = 663cy`
- `queue_end_to_end_latency`：`avg_p50 = 864cy`
- `mutex_waiter_wake_latency`：`avg_p50 = 1348cy`
- `priority_inheritance_enter_latency`：`avg_p50 = 1536cy`
- `priority_inheritance_exit_latency`：`avg_p50 = 2206cy`
- timeout wheel 五项验证全部通过，`fail_total = 0`

## 5. 已完成的归因结论
### 5.1 `IrqMutex` 尖峰归因（`runs/bench/20260324_222048/`）
- `spikes_total = 5`
- `irq_spikes_total = 5`
- `clean_spikes_total = 0`
- `systick_spikes_total = 5`
- 结论：当前 `IrqMutex` 的偶发尖峰主要来自 `SysTick` 重叠，而不是锁实现本身。

### 5.2 `queue / IRQ / semaphore` 归因（`runs/bench/20260325_131119/`）
- `10/10` 轮日志完整包含四条 attribution 输出，且全部以 `bench complete` 结束，无 `fault:`
- 聚合结果：
- `semaphore_give_to_taskb_wake`：`spikes_total = 109`，`irq_spikes_total = 109`，`clean_spikes_total = 0`，`systick_spikes_total = 109`
- `tim2_irq_to_task`：`spikes_total = 117`，`irq_spikes_total = 89`，`clean_spikes_total = 28`
- `queue_wake_latency`：`spikes_total = 245`，`irq_spikes_total = 190`，`clean_spikes_total = 55`
- `queue_end_to_end_latency`：`spikes_total = 277`，`irq_spikes_total = 222`，`clean_spikes_total = 55`

结论拆开说：
- `semaphore`：高尾已经可以归因到 `SysTick`
- `queue / IRQ`：高尾只有一部分来自 `SysTick`，仍然存在明确的 `clean_spikes`

### 5.3 `queue / IRQ` clean breakdown（`runs/bench/20260325_140028/`）
- `10/10` 轮日志完整包含：
- `tim2_irq_to_task_clean_breakdown`
- `queue_wake_latency_clean_breakdown`
- `queue_end_to_end_latency_clean_breakdown`
- `clean_breakdown_summary.csv` 聚合结果：
- `tim2_irq_to_task_clean_breakdown`：`clean_spikes_total = 28`，`resume_dominant = 28`，`unblock_dominant = 0`
- `queue_wake_latency_clean_breakdown`：`clean_spikes_total = 80`，`resume_dominant = 80`，`unblock_dominant = 0`
- `queue_end_to_end_latency_clean_breakdown`：`clean_spikes_total = 80`，`resume_dominant = 80`，`send / unblock / recv dominant = 0`
- 同批 `latency_attribution_summary.csv` 中，`semaphore_give_to_taskb_wake` 额外出现了 `1` 个 `clean_spike`；目前看更像低频离散样本，还不足以推翻“semaphore 主要由 `SysTick` 主导”的判断

进一步结论：
- `queue` clean spike 不在 `send` 路径
- `queue` clean spike 不在 `recv` 路径
- `queue / IRQ` clean spike 也不在 `kernel::unblock()` 这一段
- 当前 clean spike 全部集中在 `resume` gap，即从 `unblock` 完成到任务真正恢复执行之间的公共路径
- 因此下一步不应继续盯住队列实现本身，而应转向 `PendSV / context_switch / exception return` 这条共用唤醒恢复链路

### 5.4 公共唤醒恢复路径优化回归（`runs/bench/20260325_164200/`）
- 本轮优化：
- idle 任务不再进入 ready queue
- idle 任务不再参与时间片轮转
- 与 `runs/bench/20260325_140028/` 对比：
- `tim2_irq_to_task avg_p50`：`728cy -> 674cy`
- `queue_wake_latency avg_p50`：`725cy -> 663cy`
- `queue_end_to_end_latency avg_p50`：`950cy -> 881cy`
- `tim2_irq_to_task clean_spikes_total`：`28 -> 0`
- `queue_wake_latency clean_spikes_total`：`80 -> 56`
- `queue_end_to_end_latency clean_spikes_total`：`80 -> 61`
- `clean_breakdown_summary.csv` 仍显示剩余 `queue` clean spike 全部由 `resume` phase 主导

结论：
- 公共唤醒恢复路径优化已经生效
- `IRQ-to-task` clean spike 问题已经关闭
- 剩余问题只保留在 `queue` 路径，而且仍然是 `resume` dominant，而不是 `send / recv / unblock`

### 5.5 队列 ISR 快速发送路径回归（`runs/bench/20260325_172035/`）
- 本轮优化：
- `SyncMsgQueue::send_from_isr()`
- bench 的 queue 场景改为使用 ISR 快速发送路径
- 与 `runs/bench/20260325_164200/` 对比：
- `queue_end_to_end_latency avg_p50`：`881cy -> 864cy`
- `queue_wake_latency avg_p50`：`663cy -> 663cy`
- `queue_wake_latency clean_spikes_total`：`56 -> 21`
- `queue_end_to_end_latency clean_spikes_total`：`61 -> 21`
- `queue` clean spike 仅在 `run_08` 单轮出现
- 同批次 `tim2_irq_to_task` 在 `run_03` / `run_06` 出现稀有 spikes，但没有形成所有轮次持续存在的模式

结论：
- `queue` 剩余尾延迟收敛这项工作已经完成
- `queue` clean spike 不再稳定复现
- 后续不应再把它当成稳定缺陷处理，而应转为更长周期复验，区分环境噪声与真实回归

## 6. 首轮 mutex / PI 实跑暴露的问题（`runs/bench/20260324_185313/`）
- `BlockingMutex::acquire()` 的 owner boost 时序过晚，导致首样本可能在真正完成继承前恢复执行
- 采集脚本当时用 `Int32` 解析 cycle 数，碰到 `u32` 级大样本会溢出
- 这些问题已经修复，不再影响当前基线和 attribution 诊断

## 7. 当前工作项状态
### 7.1 8.2 P1（已完成）
- `BlockingMutex`
- 基础 priority inheritance
- 三项 bench 指标与硬件回归

### 7.2 8.3 P2（已完成）
- 采集脚本已支持抽取：
- `timeout_wheel_*`
- `scheduler_scale`
- `scheduler_o1_check`
- 相关 CSV 与 summary 已在硬件侧稳定生成

### 7.3 8.4 P1（已完成）
- `mutex_lock_unlock_attribution` 已完成 bench 输出与脚本抽取
- `IrqMutex` 尖峰已明确归因到 `SysTick`

### 7.4 8.5 P1（已完成首轮归因闭环）
- `semaphore` 的高尾已经可以归因到 `SysTick`
- `queue / IRQ` 的 attribution 也已在硬件侧跑通
- 但 `queue / IRQ` 仍保留 `clean_spikes`，说明后续要继续做实现级排查

### 7.5 8.5 P1（已完成细分归因）
- 已增加 `*_clean_breakdown` 输出与脚本抽取
- 已在 `runs/bench/20260325_140028/` 上完成 10 轮硬件回归
- 结论已经从“queue / IRQ 有 clean spike”推进到“queue / IRQ 的 clean spike 全部由公共 `resume` gap 主导”

### 7.6 8.6 P1（已完成首轮性能调优）
- 已完成“idle 不入 ready queue + idle 不参与时间片”的调度器优化
- 已在 `runs/bench/20260325_164200/` 上完成 10 轮硬件回归
- `tim2_irq_to_task` clean spike 已降为 `0`
- `queue` clean spike 数量明显下降，但尚未清零

### 7.7 8.7 P1（已完成）
- 已完成 `SyncMsgQueue::send_from_isr()` 优化
- 已在 `runs/bench/20260325_172035/` 上完成 10 轮硬件回归
- `queue` clean spike 从“多轮稳定存在”收敛到“单轮偶发”

## 8. 后续工作建议
### 8.1 P0（已完成）
- 已沉淀 10 轮稳定 `release bench` 基线

### 8.2 P1（已完成）
- 已完成 mutex / priority inheritance 功能与硬件回归

### 8.3 P2（已完成）
- 已完成 timeout / scheduler 采集脚本扩展与硬件回归

### 8.4 P1（已完成）
- 已完成 `IrqMutex` 尖峰归因，结论明确指向 `SysTick`

### 8.5 P1（已完成）
- `semaphore` 已完成归因
- `queue / IRQ` clean spike 也已完成细分归因
- 已确认 `send / recv / unblock` 不是主因

### 8.6 P1（已完成）
- 已完成公共唤醒恢复路径的首轮拆分与优化
- 已确认优化方向有效

### 8.7 P1（已完成）
- `queue` 剩余尾延迟收敛已经完成
- 当前不再把 `queue` 当作稳定缺陷源

### 8.8 P2（下一步）
- 做更长周期的 bench / soak，观察 `run_03`、`run_06`、`run_08` 这类低频长尾是否还能重复出现
- 如果后续仍然稳定复现，再重新打开更细粒度诊断

## 9. 总结
- 当前项目已经有一组稳定的 10 轮硬件性能基线。
- mutex / PI、timeout wheel、scheduler O(1) 与 `IrqMutex` 尖峰归因都已经闭环。
- 当前最值得继续推进的工作，是通过更长周期复验区分低频环境噪声与真实性能回归。


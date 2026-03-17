# CortexOS 未来开发计划讨论

## 1. 文档目的
- 用于集中讨论未来开发方向、优先级、里程碑和资源安排。
- 本文档关注“接下来做什么”，不记录详细变更历史。
- 具体实现变更记录仍写入 `CODEX_LOG.md`，测试结果仍写入 `TESTING.md`。

## 2. 当前阶段目标
- 把 `CortexOS` 从“可运行原型”推进到“可测量、可优化、可扩展的 RTOS 工程”。
- 形成稳定的内核能力基线，再逐步补齐同步原语、驱动与工程化能力。

## 3. 当前优先级草案
| 优先级 | 方向 | 目标 | 说明 |
|---|---|---|---|
| P0 | 调度器 | 实现真正 O(1) ready 选择 | 替换当前线性扫描 `pick_next_ready` |
| P0 | 同步原语 | 增加阻塞式 mutex + 优先级继承 | 解决真实 RTOS 场景缺口 |
| P1 | 性能基线 | 固化 release bench 数据 | 建立可复现优化基线 |
| P1 | 诊断能力 | 增加栈水位、运行时统计 | 便于调试和回归分析 |
| P2 | 外设与驱动 | 扩展更多中断/DMA 场景 | 提升工程实用性 |
| P2 | 工程化 | 减少警告、改善 IDE 检查体验 | 降低开发摩擦 |

## 4. 近期候选议题
- 调度器 ready queue 结构如何设计。
- mutex 所有权、等待队列和优先级继承策略。
- bench 指标是否继续扩展到内存池、事件组、消息队列容量压力场景。
- 是否增加 host 侧最小单元测试或仿真测试。
- 如何定义“阶段性完成”的验收标准。

## 5. 调度器专题：当前策略与问题
### 5.1 当前设计策略
- 使用 `READY_MASK` 和 `READY_COUNTS` 维护“哪个优先级上有 ready task”。
- 任务创建、唤醒、时间片耗尽时，只更新任务状态和对应优先级计数。
- 选最高优先级时，先从 `READY_MASK` 找到最高 ready priority。
- 但真正选中具体任务时，仍通过 `pick_next_ready` 线性扫描整个 `TASKS` 表。
- `start_first_task` 也通过扫描任务表找到第一个 ready task。
- 同优先级轮转本质上依赖“从当前 pid 之后继续扫表”的副作用，而不是显式 ready queue。

### 5.2 当前存在的问题
- 调度选择不是 O(1)：位图找到 priority 是 O(1) 倾向，但任务选择仍是 O(n) 扫表。
- 同优先级轮转不可控：轮转顺序受 pid 分布影响，不是稳定 FIFO。
- bench scaling 结果会受任务槽位分布影响，不能真实反映 O(1) 调度特征。
- `start_first_task`、`context_switch`、部分优先级调整路径都依赖扫表逻辑，维护成本高。
- 当前 `tick_at` 对 `Sleeping/Blocked` 的超时处理仍是全表扫描，因此即使 ready 选择变成 O(1)，整体调度路径也还不是完全 O(1)。

### 5.3 升级到 O(1) 需要做什么
- 为每个优先级增加独立 ready queue，而不是只保留 ready count。
- 在 `Tcb` 中增加 ready 链表或队列指针，例如 `ready_prev` / `ready_next`。
- 在调度器中维护：
- `READY_MASK`
- `READY_HEAD[prio]`
- `READY_TAIL[prio]`
- 将 `ready_inc/ready_dec` 升级为真正的 `enqueue_ready(pid)` / `dequeue_ready(pid)`。
- `pick_next_ready` 改为：
- 从 `READY_MASK` 找最高优先级
- 直接取该优先级队头任务
- `context_switch` 改为显式队列操作：
- 运行态被抢占或时间片耗尽时，放回对应优先级队尾
- 被选中运行时，从队头摘下
- `sleep_ms` / `block_current` / `unblock` / `delete_task` / `set_priority` 统一改为“状态变化时同步维护 ready queue”。
- `start_first_task` 不再扫 `TASKS`，而是直接从 ready queue 取首个任务。
- 同优先级轮转规则改为严格 FIFO，避免依赖 pid 顺序。

### 5.4 实施顺序建议
1. 先引入 ready queue 数据结构和辅助函数，不改外部 API。
2. 再替换 `start_first_task` 与 `context_switch` 的扫表逻辑。
3. 再修正 `block/sleep/unblock/delete/set_priority` 的 ready queue 维护。
4. 最后跑 `bench` 中的 `scheduler_scale` 与 `scheduler_o1_check` 验证趋势是否改善。

### 5.5 验收标准
- `pick_next_ready` 不再遍历 `TASKS`。
- `start_first_task` 不再遍历 `TASKS`。
- 同优先级任务轮转顺序稳定、可预期。
- `scheduler_scale` 在 `2/8/32` 任务下的 `per_switch_avg` 趋势明显优于当前实现。
- 默认固件和 bench 固件都能正常构建、运行。

## 6. 里程碑草案
| 里程碑 | 内容 | 验收标准 | 状态 |
|---|---|---|---|
| M1 | 调度器 O(1) 化 | scaling 指标趋势合理，代码路径不再线性扫表 | 待讨论 |
| M2 | mutex/PI 落地 | 新增 API、bench 指标、基本功能验证通过 | 待讨论 |
| M3 | 性能基线固化 | release bench 结果可重复记录 | 待讨论 |
| M4 | 工程质量收敛 | 文档、警告、IDE 配置达到可维护状态 | 待讨论 |

## 7. 讨论记录
| 日期 | 议题 | 结论 | 后续动作 |
|---|---|---|---|
| 2026-03-17 | 初始化文档 | 建立独立计划讨论文档 | 后续按主题补充 |
| 2026-03-17 | 调度器 O(1) 方案 | 明确当前策略、问题和具体改造步骤 | 后续进入设计实现 |

## 8. 开放问题
- 调度器是否要保留当前位图 + 扫描结构作为简化版本。
- bench 是否继续与默认固件保持 feature 隔离。
- 是否需要单独增加 `ROADMAP` 风格的对外版本文档。

## 9. 使用方式
- 讨论“接下来做什么”时，优先更新本文档。
- 每次讨论只改动相关议题，不把实现细节写成长日志。
- 当某项计划进入实施阶段，再把结论同步到 `README.md`、`CODEX_LOG.md`、`TESTING.md`。

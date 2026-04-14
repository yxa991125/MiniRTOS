# `collect_release_bench.ps1` 使用说明

## 1. 脚本目的
- 自动完成多轮 `release bench` 采集。
- 每轮执行：烧录固件、打开串口、复位目标、读取串口输出、导出 CSV。
- 串口读取来自 `System.IO.Ports.SerialPort`，不是 `probe-rs` 在转发 UART。

## 2. 输入前提
- 已执行：

```powershell
cargo build --release --features bench
```

- 板卡已连接，`probe-rs list` 能识别探针。
- 串口未被其他工具占用。
- 固件会在结束时打印：`bench complete`

## 3. 参数说明
| 参数 | 默认值 | 说明 |
|---|---|---|
| `-Chip` | `STM32F411RETx` | 芯片型号 |
| `-Port` | 无 | 串口号，必填，例如 `COM6` |
| `-Baud` | `115200` | 串口波特率 |
| `-Runs` | `10` | 连续采集轮数 |
| `-Speed` | `100` | SWD 速度 |
| `-Binary` | `target/thumbv7em-none-eabihf/release/CortexOS` | 固件路径 |
| `-OutputRoot` | `bench_runs` | 输出目录根路径 |
| `-ReadTimeoutMs` | `120000` | 单轮读取超时 |
| `-ResetDelayMs` | `200` | 复位后到开始读串口的等待时间 |
| `-NoFlash` | 关闭 | 跳过每轮 `probe-rs download`，只做 reset + 串口采集 |

常用命令：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COM6
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COM6 -Runs 10 -ReadTimeoutMs 180000
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/collect_release_bench.ps1 -Port COM6 -Runs 30 -ReadTimeoutMs 180000 -NoFlash
```

推荐的长周期 bench 工作流：
1. 顺序执行 `cargo build --release --features bench`
2. 手动执行一次：

```powershell
probe-rs download --chip STM32F411RETx --protocol swd --speed 100 --verify target/thumbv7em-none-eabihf/release/CortexOS
```

3. 然后使用 `-NoFlash` 做多轮 reset-only 采集，避免重复烧录拉长测试时间

## 4. 执行流程
脚本内部按下面顺序工作：
1. 检查 `-Port` 是否提供。
2. 创建本轮输出目录：`bench_runs/<timestamp>/`。
3. 进入 `for ($run = 1; $run -le $Runs; $run++)` 循环。
4. 若未使用 `-NoFlash`，执行：

```powershell
probe-rs download --chip <Chip> --protocol swd --speed <Speed> --verify <Binary>
```

5. 在本轮 `reset` 之前打开串口，并清空串口缓冲。
6. 执行：

```powershell
probe-rs reset --chip <Chip> --protocol swd --speed <Speed>
```

7. 等待 `ResetDelayMs`。
8. 读取串口行数据，直到看到 `bench complete` 或达到 `ReadTimeoutMs`。
9. 本轮采集结束后关闭串口。
10. 将该轮原始输出保存为 `run_XX.log`。
11. 用正则解析 bench 指标，写入内存汇总表。
12. 全部轮次结束后导出 CSV。

## 5. 脚本解析哪些指标
### 5.1 count-based 指标
匹配形式：

```text
bench:<name> count=<count> skipped=<skipped> min=<min>cy/... avg=<avg>cy/... p50=<p50>cy/... p95=<p95>cy/... max=<max>cy/...
```

导出到：
- `summary.csv`
- `baseline_summary.csv`

### 5.2 timeout wheel 验证行
导出到：
- `timeout_validation.csv`
- `timeout_validation_summary.csv`

### 5.3 scheduler 指标
导出到：
- `scheduler_scale.csv`
- `scheduler_scale_summary.csv`
- `scheduler_o1.csv`
- `scheduler_o1_summary.csv`

### 5.4 延迟归因指标
当前覆盖：
- `semaphore_give_to_taskb_wake_attribution`
- `tim2_irq_to_task_attribution`
- `queue_wake_latency_attribution`
- `queue_end_to_end_latency_attribution`
- `mutex_lock_unlock_attribution`

导出到：
- `latency_attribution.csv`
- `latency_attribution_summary.csv`

说明：旧目录可能仍然使用 `mutex_lock_attribution.csv` / `mutex_lock_attribution_summary.csv`。

### 5.5 clean breakdown 指标
当前覆盖：
- `tim2_irq_to_task_clean_breakdown`
- `queue_wake_latency_clean_breakdown`
- `queue_end_to_end_latency_clean_breakdown`

导出到：
- `clean_breakdown.csv`
- `clean_breakdown_summary.csv`

## 6. 输出文件说明
- `run_*.log`：逐轮原始串口日志
- `summary.csv`：逐轮逐指标原始摘要
- `baseline_summary.csv`：按指标聚合后的基线摘要
- `timeout_validation*.csv`：timeout wheel 验证结果
- `scheduler_scale*.csv`：调度缩放结果
- `scheduler_o1*.csv`：O(1) 判定结果
- `latency_attribution*.csv`：延迟归因结果
- `clean_breakdown*.csv`：clean spike 细分归因结果

## 7. 适用场景
- 采集 `5~10` 轮 `release bench`
- 对比修改前后的性能基线
- 判断异常样本是不是偶发抖动
- 分析 `queue / IRQ / semaphore / mutex` 的高尾是否与中断重叠有关
- 继续定位 `queue / IRQ` 的 clean spike 是落在 `unblock`、`resume` 还是 `send / recv` 子阶段

## 8. 当前不做的事情
- 不自动执行 `cargo build`
- 不自动识别正确串口
- 不自动判断串口输出“是否合理”
- 不自动判断 `queue / mutex` 长尾是不是异常
- 不自动汇总 `fault:*` 行

## 9. 常见失败原因
### 9.1 未提供 `-Port`
会直接报错：

```text
请使用 -Port 指定串口，例如: .\scripts\collect_release_bench.ps1 -Port COM6
```

### 9.2 `probe-rs` 找不到探针
典型报错：

```text
No connected probes were found.
```

### 9.3 `probe-rs` 能枚举探针，但无法打开
典型报错：

```text
Failed to open probe
USB error
reset not supported by WinUSB
```

这通常不是固件问题，而是主机侧 ST-Link 驱动 / USB 状态问题。

### 9.4 串口号正确但日志为空
优先检查：
- 当前 `COM` 口是否真的是板子的 USART2 VCP
- 是否有其他串口工具占用
- `ResetDelayMs` 是否过短

### 9.5 日志有内容，但始终等不到 `bench complete`
优先检查：
- `ReadTimeoutMs` 是否过短
- 当前 `BENCH_SAMPLES` 是否过大

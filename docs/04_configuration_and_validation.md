# 04 配置、频率与验证边界

## 实际提供的 `r_hand.toml`

| 字段 | 当前值 | 含义与边界 |
| --- | ---: | --- |
| `enabled` | `false` | 默认仍运行原位置跟踪 |
| `feedback_hz` | 10 | 每个舵机及完整八舵机快照的名义刷新频率 |
| `command_hz` | 100 | soft controller 目标更新频率 |
| `goal_speed_rad_s` | 1.0 | 写入 SCS0009 目标速度；真实效果需实机核对 |
| `load_filter_alpha` | 0.2 | excess load EMA 系数 |
| `contact_confirm_samples` | 5 | 连续接触确认帧 |
| `release_confirm_samples` | 3 | 当前源码未作为释放门槛使用 |
| `max_position_step_rad` | 0.01 | 每次闭合目标最大步长 |
| `communication_timeout_ms` | 200 | 反馈过期阈值 |
| `baseline_samples` | 25 | 每个 finger pair 的基线样本数 |
| `max_temperature_c` | 60 | 尚需结合硬件与厂家资料验证 |
| `min_voltage_v` / `max_voltage_v` | 4.5 / 7.2 | 依赖电压比例配置 |
| `voltage_scale_v_per_raw` | 0.1 | 源码注释明确要求在真实总线验证 |
| `fault_action` | `hold` | Fault 时保持 |
| `fault_backoff_rad` | 0.02 | 仅 `backoff` 模式使用 |

当前 `finger4.motor1.invert = true`，与较早启动日志里的 `false` 不同。文档以实际提供的 TOML 为准，并把这个差异保留下来。

## 未标定阈值

四根手指当前相同：

```toml
contact_load_threshold = 180
target_hold_load = 220
max_load = 450
```

它们不是通用安全参数。正常机械预载曾出现接近 `915`、`1000` 和有符号 `-990` 的 raw load，这正是后来增加 baseline compensation 的原因。

标定前至少要：

1. 手完全无接触；
2. 检查机械零位和是否卡滞；
3. 多个姿态采集 raw baseline；
4. 分别记录轻触、稳定持握和异常阻塞；
5. 核对温度、电压与外部测量；
6. 为每根手指分别确定阈值；
7. 保留能立即张开和断电的人工手段。

## 软件验证的时间边界

交接文档记录，较早版本执行过：

```bash
cargo fmt --check
cargo test -p AHControl --offline
cargo build -p AHControl --offline
cargo clippy -p AHControl --offline
```

当时 12 项测试通过。随后本地又有手动修改；交接文档曾指出 `main.rs` 出现使用 `#` 作为 Rust 注释的编译错误风险。

当前实际提供的 `main.rs` 中没有发现该 `#` 注释，说明该问题可能已经被修正，或当前文件来自另一个时间点。但当前只提供源码子集，缺少 `Cargo.toml`、`Cargo.lock`、`lib.rs` 和若干 binary，无法在本仓库重建并重新执行上述检查。

因此当前状态应写成：

| 验证项 | 状态 |
| --- | --- |
| 历史 12 项单元测试 | 交接记录确认曾通过 |
| 当前公开源码快照重新构建 | 未执行，源码不完整 |
| 真实硬件逐步日志与人工测试 | 交接记录确认做过 |
| 力阈值、机械零位、温度/电压比例 | 未完成系统标定 |
| 自动化成功率或 benchmark | 未进行，不提供虚构数字 |


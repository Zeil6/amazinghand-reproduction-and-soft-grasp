# 06 源码证据与缺失项

## 当前实际提供并随分支保存的源码

```text
source_snapshot/Demo/AHControl/src/main.rs
source_snapshot/Demo/AHControl/src/config.rs
source_snapshot/Demo/AHControl/src/feedback.rs
source_snapshot/Demo/AHControl/src/soft_grasp.rs
source_snapshot/Demo/AHControl/src/safety.rs
source_snapshot/Demo/AHControl/config/r_hand.toml
```

这些文件足以核对核心状态机、反馈读取、安全检查和实际 TOML，但不是完整可构建仓库。

## 交接文档提到、但本次没有实际提供的文件

| 路径 | 状态 |
| --- | --- |
| `AHControl/src/bin/monitor_feedback.rs` | 缺失；只保留职责和命令记录 |
| `AHControl/src/bin/calibrate_load.rs` | 缺失 |
| `AHControl/src/bin/soft_grasp_test.rs` | 缺失 |
| `AHControl/src/bin/enable_feedback_responses.rs` | 缺失 |
| `dataflow_tracking_real.yml` 的本地修改版 | 缺失；不能验证新增 outputs 的 YAML 是否与代码一致 |
| `docs/soft_grasp_control.md` | 缺失 |
| 本地 `Cargo.toml` / `Cargo.lock` | 缺失；依赖版本只能依据交接审计 |
| `AHControl/src/lib.rs` | 缺失；公开 snapshot 无法单独编译 |

我没有根据交接文档补写这些文件，也没有创建一个看似完整但未经验证的工程。

## 从当前源码确认的关键事实

### `feedback.rs`

- 用 protocol-v1 unicast `read_raw_data(id, 56, 8)`；
- 同步写 goal position、goal speed、torque enable；
- raw load、temperature、voltage 都来自 SCS0009 连续寄存器块。

### `soft_grasp.rs`

- 每根手指维护独立状态和 previous/hold target；
- 新 physical pair frame 才推进滤波；
- excess load 先减 baseline，再 EMA；
- closing step 限幅；
- opening 立即绕过 Holding；
- `release_confirm_samples` 当前没有参与门控。

### `safety.rs`

- invalid/stale feedback 先判 FeedbackTimeout；
- 再检查温度；
- 再检查电压；
- overload 在 per-finger controller 中检查。

### `main.rs`

- soft mode 启动时写目标速度；
- 先发送零目标并 seed initial targets；
- 每次只轮询一个 servo ID；
- 所有反馈有效、所有基线完成后才输出 soft-grasp command；
- 发布三个 Dora 输出的代码存在；
- `fault_action` 与 `shutdown_action` 分开处理。

### `r_hand.toml`

- 四个 finger pair 对应 ID 1—8；
- `finger4.motor1.invert = true`；
- `soft_grasp.enabled = false`；
- 阈值明确注释为未标定的起始占位。


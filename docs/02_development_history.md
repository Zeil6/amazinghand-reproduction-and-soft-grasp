# 02 真实开发顺序

## 1. Dora 在节点初始化前失败

出现：

```text
failed to set tracing global subscriber for hand_controller
a global default trace dispatcher has already been set
```

最初只看到“hand_controller 还没 ready”。检查后发现，`AHControl` 和 Dora 都尝试设置全局 tracing subscriber。全局 subscriber 只能设置一次。

修复方向是移除 `AHControl` 自己的全局 tracing 初始化，让 Dora 管理该环境。这个问题发生在节点业务循环之前，与舵机反馈协议无关。

## 2. 位置能写，反馈却读超时

`monitor_feedback` 出现：

```text
SCS0009 feedback read failed: Operation timed out
```

我一开始容易把“舵机可以运动”理解成“串口读写都正常”。后来意识到：

- sync write 是广播写，不要求每个舵机回复；
- read 必须收到目标 ID 的状态包；
- 舵机可能不支持当前 broadcast sync-read 方式；
- response status level 可能没有启用到需要的回复等级。

于是反馈实现从 broadcast sync read 改为：

```rust
read_raw_data(id, 56, 8)
```

即逐个 ID 的 protocol-v1 unicast 读取。

交接记录还增加了 `enable_feedback_responses`，显式写 SCS0009 register 8 `response_status_level`。因为这是持久设置，工具要求 `--apply`，避免误改舵机配置。

## 3. 柔性模式只动一次后冻结

这个现象后来被拆成多层原因：

1. 反馈无效时，安全门正确地阻止继续输出；
2. 未做基线补偿时，正常机械预载已经接近 `915`、`1000` 或 `-990`；
3. 占位 `max_load = 450` 会把正常预载直接判成超载；
4. 第一次软抓取命令把当前 IK 目标当作 previous target，步长限制没有约束第一跳；
5. SCS0009 目标速度没有被显式降低。

对应修改：

- 所有反馈有效且 baseline ready 后才进入软抓取指令；
- 从初始零目标 seed `previous_target`；
- soft mode 写入 `goal_speed_rad_s`；
- 引入负载基线和 excess load。

我没有通过盲目增大 `max_load` 解决，因为那会掩盖预载、零位、卡死和真实过载之间的区别。

## 4. Fault 日志刷屏

Fault 原先每个 100 Hz command 周期都会打印。修正后只在“进入 Fault 的状态变化”时打印详细诊断：

```text
raw
baseline
filtered raw
estimated
max_load
```

全局安全故障也只在 reason 变化时输出。

## 5. 反馈读取让动作分段

八个舵机共享同一串口。一次连续读取八个 ID 会在发送位置命令的总线上形成长阻塞，画面中运动变得更分段。

当前策略：

```text
command target: 100 Hz
one feedback ID: every 1 / (10 × 8) s = 12.5 ms
one servo refresh: 10 Hz
all-eight snapshot: 10 Hz
```

这不是“同时达到 100 Hz 完整反馈”，而是在命令频率与完整反馈刷新之间做取舍。下一步更合理的架构是一个独占串口的 worker，对写命令和读反馈做优先级调度，而不是让多个线程/进程各自打开同一串口。


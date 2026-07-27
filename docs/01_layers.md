# 01 四个层级怎样配合

## Rust：写控制逻辑的语言

`Demo/AHControl/src/main.rs` 使用 Rust：

- 读取手部配置；
- 打开串口；
- 创建 SCS0009 controller；
- 接收 Dora 输入；
- 按 metadata 取出八个模型关节目标；
- 应用 offset 和 invert；
- 同步写入八个舵机。

选择 Rust 不会自动让控制“安全”或“实时”，但强类型、错误处理和较低运行时开销很适合硬件 I/O 节点。

## Cargo：管理 Rust 项目

Cargo 负责：

- 读取 `Cargo.toml`；
- 解析依赖；
- 使用 `Cargo.lock` 固定解析结果；
- 编译 crate；
- 运行 test、clippy 和 binary；
- 管理 workspace 中多个 package。

Cargo 不是机器人中间件，也不负责 Python 节点之间发消息。

## Dora：编排完整系统

Dora 读取 dataflow YAML，知道：

- 有哪些 node；
- 每个 node 如何 build；
- 从哪里启动；
- 有哪些 input/output；
- 哪个 output 连接哪个 input；
- 哪些输入由 timer 周期产生。

它允许 HandTracking 和 AHSimulation 用 Python，AHControl 用 Rust。

## Zenoh：通信运行时

启动日志中的：

```text
zenoh::net::runtime: Using ZID
```

表示通信运行时建立了身份和会话。它不是 MediaPipe、IK 或舵机协议。只要没有明确的 bind/transport 错误，看到 ZID 本身不代表故障。


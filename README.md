# 在 AmazingHand 中理解 Dora、Rust 与 Cargo

我一开始把 Dora、Rust 和 Cargo 当成三种并列的“机器人软件”。真正跑过 AmazingHand 后，我才分清：

| 名称 | 所在层级 | 在本项目中的工作 |
| --- | --- | --- |
| Rust | 编程语言 | 编写 `AHControl` 硬件控制节点 |
| Cargo | Rust 的包、依赖与构建工具 | 解析 `Cargo.toml`/`Cargo.lock`，编译 `AHControl` |
| Dora | 数据流编排与跨语言通信框架 | 启动 Python/Rust 节点，连接 topic 和定时器，管理生命周期 |
| Zenoh | Dora 使用的通信运行时之一 | 建立会话、发现和传输；启动日志中的 ZID 属于这一层 |

所以它们不是同一层级，也不能互相替代。

## 阅读顺序

1. [四个层级怎样配合](docs/01_layers.md)
2. [AmazingHand 数据流](docs/02_dataflow.md)
3. [Cargo workspace 与 `-p AHControl`](docs/03_cargo_workspace.md)
4. [如何用层级判断错误](docs/04_debugging_with_layers.md)

[返回 `main`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp)


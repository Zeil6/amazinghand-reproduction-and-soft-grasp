# 03 Cargo 构建与依赖问题

## 背景

运行：

```bash
cargo build -p AHControl
```

时，编译并不是在 `AHControl` 自己的业务代码处首先失败，而是在 Dora 依赖内部失败。

## 原始证据

日志同时出现：

```text
Compiling dora-message v0.6.0
Compiling dora-message v0.4.4
Compiling dora-node-api v0.3.11
```

随后出现：

```text
error[E0432]: unresolved import `dora_metrics::init_meter_provider`
```

以及：

```text
expected `dora_message::config::NodeId`,
found `dora_core::config::NodeId`

note: there are multiple different versions of crate
`dora_message` in the dependency graph
```

同类错误还影响 `OperatorId`、`DataId` 和 `Input`。

## 初步判断为何需要修正

只看 `expected X, found X`，容易误以为 Rust 编译器在自相矛盾。实际两个类型的名字相同，但来自不同 crate 版本；Rust 会把它们视为完全不同的类型。

因此根因不是“给变量做一次类型转换”，而是依赖图中 Dora 组件版本没有对齐。

## 检查方法

```bash
cargo tree -p AHControl
cargo tree -p AHControl -d
cargo tree -i dora-message@0.4.4
cargo tree -i dora-message@0.6.0
```

我需要回答的是：

1. 哪个直接依赖拉入 `dora-node-api 0.3.11`；
2. 哪个组件已经是 0.3.13；
3. `Cargo.lock` 是否锁住了不兼容组合；
4. workspace 中是否有其他 package 施加了约束。

## 能确认的修复结果

后续软抓取交接审计记录：

```text
dora / dora-node-api pinned to 0.3.13
rustypot resolved to 1.5.0 from manifest range 1.1.0
```

这说明最终工作副本选择统一 Dora 0.3.13，而不是在业务代码里绕过类型错误。

但原始材料没有保存每一条实际执行的 `cargo update` 或锁文件修改命令，所以我不把推测的命令写成已执行记录。若重新处理，应先修改 `Cargo.toml` 约束，再审查 `Cargo.lock` 和 `cargo tree`，避免直接删除 lockfile 后让所有依赖无控制升级。

## 验证边界

交接文档记录的上一次软件验证包括：

```bash
cargo fmt --check
cargo test -p AHControl --offline
cargo build -p AHControl --offline
cargo clippy -p AHControl --offline
```

当时 12 项单元测试通过。但这发生在后续本地手动修改之前，不能等价为当前工作树已验证；而且这组测试属于后续软抓取版本，不属于原版复现成果。

## 复盘

以后看到 Rust 类型“不一致”时，我会先检查类型的完整来源和重复依赖版本：

```text
错误位置在依赖内部
  -> 查看 Cargo.lock
  -> cargo tree -d
  -> 对齐同一组件族版本
  -> 再看业务代码
```

这类问题属于依赖解析与版本一致性问题，不是串口、摄像头或 Rust 语法问题。


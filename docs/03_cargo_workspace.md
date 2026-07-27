# 03 Cargo workspace 与 `-p AHControl`

## crate、package、workspace

- **crate**：Rust 编译单元，可能是 library 或 binary。
- **package**：由一个 `Cargo.toml` 描述，可以包含多个 crate target。
- **workspace**：把多个 package 放在一个统一构建和 lockfile 范围中。

在 AmazingHand `Demo` 目录运行：

```bash
cargo build -p AHControl
```

其中 `-p` 是 `--package` 的缩写，用来从 workspace 中选择名为 `AHControl` 的 package。它不是“Python”或“port”的缩写。

## 构建时发生了什么

```text
读取 workspace/Cargo.toml
  -> 找到 AHControl package
  -> 合并版本约束
  -> 使用 Cargo.lock 的具体解析结果
  -> 编译依赖 crate
  -> 编译 AHControl
  -> 生成 target/debug/AHControl
```

Dora YAML 中：

```yaml
build: cargo build -p AHControl
path: target/debug/AHControl
```

第一行负责构建，第二行负责运行生成的 binary。这两步的错误类型不同：

- build 失败：依赖、Rust 类型或链接问题；
- path 启动失败：文件不存在、权限或动态链接问题；
- 启动后失败：配置、串口、Dora 环境或业务逻辑问题。

## `Cargo.toml` 与 `Cargo.lock`

`Cargo.toml` 写“允许什么范围”，`Cargo.lock` 记录“这次到底选了什么版本”。

本次构建错误中，`dora-node-api 0.3.11` 与其他 Dora 0.3.13 组件拉入不同 `dora-message`，说明只看一行直接依赖不够；必须结合 lockfile 和完整依赖图。


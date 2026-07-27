# 04 Dora 运行错误与级联退出

## 背景

再次运行：

```bash
dora run dataflow_tracking_real.yml
```

时，三个节点都在日志中出现错误。最初如果只看最后的 `hand_controller`，很容易误判为 Rust 控制节点坏了。

## 第一故障点

日志最先显示 `hand_tracker` 在导入 OpenCV 时失败：

```text
ImportError: libGL.so.1: cannot open shared object file:
No such file or directory
```

随后 Dora 明确记录：

```text
Node `hand_tracker` failed
```

而 `hand_simulation` 和 `hand_controller` 的核心错误都是：

```text
subscribe failed: Node hand_tracker exited before initializing dora
```

## 根因与后果

根因位于 HandTracking 的运行环境：OpenCV 动态库加载需要 `libGL.so.1`，但系统无法找到它。

后两个错误是级联结果：

```text
hand_tracker 退出
  -> hand_simulation 的输入源消失
  -> hand_controller 的上游链路无法建立
  -> Dora 停止整个 required dataflow
```

因此不应该分别“修”三个节点。

## 修复记录的边界

在 Ubuntu 上，这类缺失通常通过安装提供 `libGL.so.1` 的系统包（常见为 `libgl1`），或在确实不需要 GUI 时选择 headless OpenCV 变体解决。

但本次保留下来的材料只有失败日志，没有保存最终执行的系统安装命令。公开记录因此把该步骤标为“待复核执行”，不写成已经运行：

```bash
# 常见检查方向，不代表本次已保存的执行记录
ldconfig -p | rg 'libGL\.so\.1'
dpkg -S libGL.so.1
```

## 为什么 Zenoh 日志不是错误

启动时出现 `Using ZID`、监听地址和 scout multicast，是 Dora 底层通信运行时建立会话的日志。除非后面明确出现 transport bind、连接或权限失败，它们不是本次根因。

公开仓库不保留真实局域网地址、ZID、dataflow UUID 和本机绝对路径；这些字段对解释 `libGL` 根因没有必要。

## 排查方法

我后来采用：

1. 找到最早出现的 traceback；
2. 区分 primary failure 与 consequential error；
3. 单独运行首个失败节点的 import 检查；
4. 只修首因，再重新启动整个 dataflow；
5. 确认每个节点先后进入 ready，而不是只看 daemon 已启动。


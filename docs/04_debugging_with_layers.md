# 04 用层级判断错误

## 同一条链路的不同错误

| 现象 | 优先检查层 |
| --- | --- |
| `requires-python` 不满足 | Python 包元数据 |
| Rust 的同名类型不匹配 | Cargo 依赖图 |
| `libGL.so.1` 缺失 | Ubuntu 动态库 |
| `Node hand_tracker exited` | Dora dataflow 的首个失败节点 |
| `/dev/ttyACM0` permission denied | 操作系统设备权限 |
| 舵机方向相反 | `r_hand.toml` offset/invert 与机械标定 |

我以前容易把所有问题都归为“Dora 没配好”。现在会先问：

```text
代码有没有编译？
binary 有没有启动？
节点有没有连接 Dora？
topic 名是否一致？
串口有没有打开？
舵机有没有收到正确目标？
```

## 一个典型例子

`hand_tracker` 因 OpenCV 找不到 `libGL.so.1` 退出后，两个下游节点报告订阅失败。这时：

- Rust 编译并不是首因；
- Zenoh 的 ZID 也不是首因；
- AHSimulation 的 Node 初始化错误是上游退出的结果；
- 应先让 `python -c "import cv2"` 成功。

理解层级后，我不再逐条追着最后一屏错误改代码，而是沿时间顺序找到第一个不可恢复失败。


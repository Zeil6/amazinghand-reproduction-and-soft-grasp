# 04 Dora、Zenoh 与多频率数据流

## Dora 负责什么

Dora 不是 MediaPipe、Mink 或舵机驱动。它负责：

- build 和启动 node；
- 连接 input/output；
- 产生 timer；
- 跨 Python/Rust 传递 Arrow 数据；
- 管理 dataflow 生命周期；
- 在上游失败时结束依赖链。

## 真机与仿真

仿真链：

```text
hand_tracker
  -> hand simulation
  -> MuJoCo Viewer
```

真机链多一个：

```text
hand simulation
  -> hand_controller
  -> serial bus
  -> SCS0009
```

AHSimulation 仍然位于真机链中，因为它承担运动重定向和 IK，不只是“显示一个仿真窗口”。

## Zenoh 日志

Dora 启动时的 Zenoh ZID、地址和 scout 信息表示通信运行时会话建立。它们有助于定位网络/transport 层，但不解释 OpenCV import、Cargo 类型或舵机零位问题。

## 级联故障

一次真实日志中：

```text
hand_tracker: ImportError libGL.so.1
  -> hand_simulation subscribe failed
  -> hand_controller subscribe failed
  -> dataflow stopped
```

这说明分析 Dora 日志要按时间找首因，不能把三个退出节点当成三个独立根因。

## 运行条件

- 摄像头需要能够被 OpenCV 打开；
- `libGL` 等系统动态库要满足 OpenCV；
- Python 节点依赖要在同一环境；
- Rust binary 要先构建成功；
- 串口路径、权限、波特率正确；
- 八个舵机需要外部供电；
- dataflow 的 topic 和 metadata 名称必须与源码版本一致。


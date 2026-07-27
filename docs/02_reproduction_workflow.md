# 02 原版复现流程

## 我采用的分层顺序

原版真机链路可以概括为：

```text
Camera
  -> HandTracking / MediaPipe
  -> AHSimulation / MuJoCo + Mink IK
  -> AHControl / Rust
  -> 8 × SCS0009
```

上游 2026-07-27 固定快照使用：

```text
r_hand_pos -> mj_r_joints_pos
```

我的本地副本和后续交接材料使用：

```text
hand_pos -> mj_joints_pos
```

运行前必须以当前 `dataflow_tracking_real.yml` 为准，不能把两套 topic 名混用。

## 推荐检查顺序

### 1. 确认 Python 环境

```bash
conda activate amazinghand
python --version
python -c "import cv2, mediapipe, mujoco, mink, pyarrow"
```

这一步只验证 Python 导入，不代表 Dora 或舵机已经可用。

### 2. 确认 Rust/Cargo

```bash
rustc --version
cargo --version
cargo check -p AHControl
```

`cargo check` 更适合先看类型和依赖问题；需要真正生成二进制时再运行：

```bash
cargo build -p AHControl
```

### 3. 检查摄像头

```bash
ls -l /dev/video*
v4l2-ctl --list-devices
```

本地 HandTracking 源码默认 `cv2.VideoCapture(0)`。多个摄像头同时存在时，编号可能变化，不能仅凭上次的 index 推断。

### 4. 检查串口

```bash
ls -l /dev/ttyACM*
groups
```

上游 dataflow 默认 `/dev/ttyACM0`、`1_000_000` baud。设备路径不是硬件身份，重新插拔后可能变化。

### 5. 先仿真，再真机

先验证 `HandTracking -> AHSimulation`，确认四个目标和八个模型关节输出合理；再连接电机电源、检查零位、offset、invert 和运动方向。

### 6. 运行 dataflow

```bash
dora run dataflow_tracking_real.yml
```

我现在不会看到最后一个节点报错就直接改最后一个节点。Dora 会把上游失败传播给订阅者，真正的第一故障点通常出现在日志更前面。

## 一次可复核的正常启动线索

历史日志中能确认：

- Zenoh runtime 启动并打印 ZID；
- Dora 依次 spawn `hand_controller`、`hand_simulation`、`hand_tracker`；
- `hand_controller` 读取 `AHControl/config/r_hand.toml`；
- 配置包含四组 finger、八个 SCS0009 ID；
- 后续节点是否正常，仍要继续检查每个节点的 `ready` 和首个异常。

这些日志只说明编排器开始工作，不等于手部跟踪和真机运动已经全部成功。


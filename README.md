# AmazingHand 复现与负载反馈软抓取记录

<p align="center">
  <a href="assets/videos/gesture_tracking_demo.mp4">
    <img src="assets/images/gesture_tracking_demo.gif" alt="AmazingHand 手势识别与连续动作跟踪演示" width="600">
  </a>
</p>

<p align="center"><sub>手势识别与连续动作跟踪演示（点击 GIF 查看 MP4 清晰版）</sub></p>

<p align="center">
  <a href="assets/videos/amazinghand_hardware_motion_demo.mp4">
    <img src="assets/images/amazinghand_hardware_motion_demo.gif" alt="AmazingHand 真机连续动作演示" width="420">
  </a>
</p>

<p align="center"><sub>AmazingHand 真机连续动作演示（点击 GIF 查看 MP4 清晰版）</sub></p>

这个仓库记录我复现 [Pollen Robotics AmazingHand](https://github.com/pollen-robotics/AmazingHand) 的过程，以及在原有位置跟踪链路上尝试加入 SCS0009 舵机负载反馈的开发记录。

它不是一份“已经完成全部验证”的成品说明。仓库保留我遇到的依赖问题、运行故障、判断修正和当前仍未完成的硬件标定。

## 我为什么复现这个项目

AmazingHand 把摄像头手部跟踪、MuJoCo/Mink 逆运动学、Dora 数据流和八个串行总线舵机放在同一条真机链路中。对我来说，复现它的价值不只是让机械手跟着人手运动，而是把感知、运动重定向、硬件控制、通信和安全边界串起来理解。

原版位置跟踪运行起来后，我继续追问：机械手接触物体后，控制器是否知道它已经接触？如果舵机负载、温度或通信状态异常，程序会怎样处理？这才形成了后续的负载反馈软抓取实验。

## 原项目与我的工作边界

| 范围 | 内容 |
| --- | --- |
| AmazingHand 原有设计 | 四指八自由度机械结构；SCS0009 舵机；MediaPipe 手部跟踪；MuJoCo/Mink 运动重定向；Dora 节点编排；Rust `AHControl` 位置指令下发 |
| 我完成的复现工作 | conda 环境配置；Python、Rust、Cargo、Dora 安装与运行；摄像头和串口检查；依赖冲突与运行错误定位；仿真和真机链路验证 |
| 我后来新增的实验功能 | 读取舵机位置、负载、温度和电压反馈；启动负载基线；接触判断；步长限制；Holding/Fault 状态；反馈超时和安全动作；辅助诊断思路 |

“新增”只表示相对于我复现的原始控制节点所做的修改。它不是经过计量标定的力控制，也不代表相关阈值已经适用于其他机械手。

## 研究过程

| 时间 | 记录 |
| --- | --- |
| 2026-07-13 | 创建 conda 环境，处理 HandTracking Python 版本声明与 `AHControl` Cargo 依赖问题，开始跑通 Dora 三节点 |
| 2026-07-14 | 结合项目区分 Dora、Rust 和 Cargo 所在层级，整理构建与运行错误的排查方法 |
| 2026-07-23 | 再次运行时遇到 `libGL.so.1` 缺失及下游级联退出；确认原控制链缺少反馈闭环 |
| 2026-07-24—27 | 阅读关键源码，开发并记录基于 SCS0009 负载反馈的软抓取第一阶段；整理配置、风险、视频和交接证据 |

## 分支导航

| 分支 | 主要内容 | 当前状态 |
| --- | --- | --- |
| [`reproduction-troubleshooting`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp/tree/reproduction-troubleshooting) | 原项目复现顺序、真实日志证据、依赖与运行问题复盘 | 已整理；部分最终修复命令因记录不足标为待核对 |
| [`dora-rust-cargo-notes`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp/tree/dora-rust-cargo-notes) | 结合 AmazingHand 理解 Dora、Rust、Cargo 和 Zenoh | 已整理，后续可随源码阅读补充 |
| [`soft-grasp-development`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp/tree/soft-grasp-development) | SCS0009 负载反馈软抓取的源码证据、开发顺序、安全边界和视频对照 | 实验性开发；尚未完成硬件力阈值标定 |
| [`amazinghand-project-analysis`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp/tree/amazinghand-project-analysis) | HandTracking、AHSimulation、AHControl 与 Dora 数据流解析 | 已完成第一轮源码级整理 |

`main` 只保留边界、总览和入口，完整专题内容位于各自分支。

## 已确认的软硬件范围

| 项目 | 已确认信息 |
| --- | --- |
| 机械手 | AmazingHand，四根手指、八个驱动自由度 |
| 舵机 | 8 × Feetech SCS0009，ID 1—8 |
| 感知 | 普通摄像头；本地 `main.py` 默认 `cv2.VideoCapture(0)`；MediaPipe Hands |
| 运动重定向 | MuJoCo 模型 + Mink 微分逆运动学 |
| 编排 | Dora；后期交接材料固定 `dora-node-api = 0.3.13` |
| 控制 | Rust `AHControl`；默认串口 `/dev/ttyACM0`，默认波特率 `1_000_000` |
| Python | 实际 conda 环境记录为 Python 3.12.13；上游不同子包的版本声明并不完全一致 |
| 操作系统 | Ubuntu；本次材料未保存可独立复核的发行版信息 |

Rust 工具链、MuJoCo/Mink 的实际安装版本没有被完整保存，因此不补写具体值。

## 上游版本与许可证

核对日期：2026-07-27。

- 上游仓库：[`pollen-robotics/AmazingHand`](https://github.com/pollen-robotics/AmazingHand)
- 本次源码对照快照：[`3e8241074df3436a3044ced4881e3bb2133aa725`](https://github.com/pollen-robotics/AmazingHand/tree/3e8241074df3436a3044ced4881e3bb2133aa725)
- 上游 `v1.0`：[`23a262c94748ac061a63c6d32158a7f094c25b6e`](https://github.com/pollen-robotics/AmazingHand/tree/23a262c94748ac061a63c6d32158a7f094c25b6e)
- 软件许可证：[Apache License 2.0](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/LICENSE)
- 上游 README 同时说明机械设计采用 [CC BY 4.0](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/README.md)

仓库中的说明文档由我根据复现记录整理；专题分支中如包含修改后的上游代码片段，会保留 Apache 2.0 许可证并单独说明来源和修改边界。

## 安全与实验限制

- `raw_load` 和 `estimated_load` 是舵机反馈量，不是牛顿单位的指尖力。
- `contact_load_threshold = 180`、`target_hold_load = 220`、`max_load = 450` 是尚未完成硬件标定的占位参数。
- 机械零位、装配预载、舵机方向和供电会改变反馈含义，不能直接照搬参数。
- 反馈读取与位置写入共享串口总线；提高反馈频率可能让运动变得分段。
- 当前默认仍是原位置跟踪模式，`soft_grasp.enabled = false`。
- 上一次通过的软件检查发生在后续本地修改之前；当前材料不足以声称最新工作树已全部构建、测试和真机安全验证。

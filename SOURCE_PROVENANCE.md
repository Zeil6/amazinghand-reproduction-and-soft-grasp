# 源码来源与修改边界

## 上游

- 仓库：[`pollen-robotics/AmazingHand`](https://github.com/pollen-robotics/AmazingHand)
- 对照快照：[`3e8241074df3436a3044ced4881e3bb2133aa725`](https://github.com/pollen-robotics/AmazingHand/tree/3e8241074df3436a3044ced4881e3bb2133aa725)
- 软件许可证：Apache License 2.0

上游 `Demo/AHControl/src/main.rs` 原本负责读取配置、接收 `mj_r_joints_pos`、应用 offset/invert 并同步写位置目标。

## 本分支的 source snapshot

`source_snapshot/` 是用户当前提供的本地修改文件子集，不是完整上游 fork，也不保证独立构建。

相对于上游控制节点，当前子集增加或重构了：

- serde TOML 配置和 legacy 格式兼容；
- 位置限位；
- SCS0009 feedback abstraction；
- unicast 寄存器读取；
- soft-grasp controller；
- baseline、滤波、接触与 Holding；
- temperature/voltage/timeout safety；
- round-robin feedback scheduling；
- Dora feedback/state outputs；
- fault/shutdown action。

本仓库没有把上游机械结构、MediaPipe、MuJoCo/Mink 链路或 SCS0009 支持描述成个人原创。

`LICENSE-APACHE-2.0` 是上游软件许可证副本。修改文件应结合本说明阅读；本仓库没有擅自替换上游许可证。

为清楚满足再分发边界，`source_snapshot/` 中每个修改文件顶部也写明了上游来源和本地修改范围。

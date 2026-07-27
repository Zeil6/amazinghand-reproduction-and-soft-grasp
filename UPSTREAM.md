# 上游来源与版本记录

## 固定快照

本仓库在 2026-07-27 对照：

```text
repository: pollen-robotics/AmazingHand
default branch: main
commit: 3e8241074df3436a3044ced4881e3bb2133aa725
release v1.0 commit: 23a262c94748ac061a63c6d32158a7f094c25b6e
```

固定到 commit 的源码入口：

- [HandTracking/main.py](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/Demo/HandTracking/HandTracking/main.py)
- [AHSimulation/mj_mink_right.py](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/Demo/AHSimulation/AHSimulation/mj_mink_right.py)
- [AHControl/src/main.rs](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/Demo/AHControl/src/main.rs)
- [dataflow_tracking_real.yml](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/Demo/dataflow_tracking_real.yml)

## 一个需要保留的版本差异

上游固定快照中的右手消息名是 `r_hand_pos` 和 `mj_r_joints_pos`。本次拿到的本地工作副本与软抓取交接材料使用 `hand_pos` 和 `mj_joints_pos`，finger metadata 也从 `r_finger1` 等变成 `finger1` 等。

这说明本地复现副本并不等同于 2026-07-27 的上游 `main`。各专题文档会分别注明自己依据的是“上游固定快照”还是“本地提供源码”，避免把两套命名混写成同一版本。

## 许可证

上游软件使用 Apache License 2.0；上游 README 说明机械设计使用 CC BY 4.0。本仓库不重新许可上游项目，也不把上游设计描述成个人原创。


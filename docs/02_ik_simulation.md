# 02 AHSimulation：MuJoCo 与 Mink IK

## MuJoCo 在这里做什么

`mj_mink_right.py` 从 MJCF `scene.xml` 加载机械手模型，并创建：

- `mink.Configuration`；
- 四个指尖 `FrameTask`；
- `PostureTask`；
- `EqualityConstraintTask`。

这条链路主要把 MuJoCo 当作机械手模型、关节状态和 Viewer 容器。源码中的 `mujoco.mj_step()` 被注释，不能把它描述成完整接触动力学仿真。

## Mink 如何把指尖目标变成关节目标

HandTracking 输出的四个向量经过比例和偏移，写入四个 mocap target：

```text
r_tip1 -> finger1_target
r_tip2 -> finger2_target
r_tip3 -> finger3_target
r_tip4 -> finger4_target
```

每次 IK tick：

1. FrameTask 读取 mocap target；
2. `mink.solve_ik()` 求关节速度；
3. `configuration.integrate_inplace()` 更新模型配置；
4. 从八个 motor joint 读取 qpos；
5. 每个 `tick_ctrl` 发布八维数组。

## 八个值怎样排列

metadata 写入：

```text
finger1 -> [0, 1]
finger2 -> [2, 3]
finger3 -> [4, 5]
finger4 -> [6, 7]
```

上游快照使用 `r_finger1` 等带前缀名称，本地副本使用 `finger1`。Rust 配置中的 `finger_name` 必须匹配。

## 多频率与时间步

上游 YAML：

| 事件 | 周期 | 理论频率 |
| --- | ---: | ---: |
| HandTracking tick | 50 ms | 20 Hz |
| AHSimulation IK tick | 2 ms | 500 Hz |
| joint output tick | 10 ms | 100 Hz |

源码还创建 `RateLimiter(frequency=1000.0)`，并把 `rate.dt = 0.001 s` 传给 Mink，但 Dora IK tick 是 0.002 s，且没有调用 `rate.sleep()`。所以“1000 Hz”不是实际调度频率，dt 也与 timer 不一致。

这属于 demo 的时间尺度风险，后续应使用真实测量 dt 或与 timer 一致的固定 dt，并统计求解耗时和 deadline miss。


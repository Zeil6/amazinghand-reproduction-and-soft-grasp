# 01 功能边界与架构

## 它是什么

八个 SCS0009 舵机在总线寄存器中提供当前位置、速度、负载、输入电压和温度。我的第一阶段修改让 `AHControl` 在发送位置目标的同时读取这些反馈，并用负载变化判断“可能接触”，再限制继续闭合。

它不是：

- 标定过的指尖力传感器；
- 牛顿单位的力闭环；
- 独立的触觉阵列；
- 已完成安全认证的控制器；
- 通用于所有 AmazingHand 装配的阈值。

## 从开环位置跟踪到局部反馈

原节点收到八个模型关节目标后，应用 offset/invert，然后同步写入八个舵机。控制器并不知道：

- 实际位置是否跟上；
- 手指是否接触物体；
- 舵机是否过载；
- 温度、电压或通信是否异常。

修改后的控制层增加：

```text
requested target
  -> limit close step
  -> read servo feedback
  -> compensate startup preload
  -> filter load
  -> contact / fault decision
  -> command or hold/backoff/torque-off
```

这个反馈只在 `AHControl` 层形成局部闭环；视觉目标生成和 Mink IK 没有因此变成力控制。

## 反馈结构

当前 `ServoFeedback` 包含：

```text
id
position_rad
speed
raw_load
filtered_load
temperature_c
voltage_v
timestamp
valid
```

SCS0009 寄存器 56—63 被一次 unicast raw read 读取。负载使用有符号原始量转换，电压按配置比例换算。`voltage_scale_v_per_raw = 0.1` 仍要求用真实总线测量复核。

## 基线与 estimated load

交接材料用下面的单帧关系说明静态预载补偿：

```text
excess_load = max(0, abs(raw_load) - baseline_raw_load)
```

实际 `soft_grasp.rs` 随后对 `excess_load` 做 EMA：

```text
filtered_excess =
    alpha * excess_load
  + (1 - alpha) * previous_filtered_excess
```

状态机使用的是两个舵机 `filtered_excess` 中的较大值。因此文档中的 `estimated_load` 更准确地说是“经过基线扣除并低通滤波的 excess load”。

基线补偿只能减小静态机械预载的影响，不能：

- 修复错误 offset 或 invert；
- 掩盖机构卡死；
- 识别基线阶段已经接触物体；
- 把 raw load 变成指尖力；
- 替代每台手的标定。


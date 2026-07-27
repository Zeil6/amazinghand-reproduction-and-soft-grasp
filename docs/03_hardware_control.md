# 03 AHControl：八个目标到八个舵机

## 为什么控制节点使用 Rust

AHControl 要处理：

- 强类型的 Dora 输入；
- 配置解析；
- 串口和二进制舵机协议；
- 八个 ID 的同步写；
- 启停和错误传播。

Rust 适合把协议和配置错误显式化，但硬件安全仍需要额外的限位、反馈、超时和测试。

## 四根手指与八个 ID

当前实际 `r_hand.toml`：

| finger | motor IDs | 数组索引 |
| --- | --- | --- |
| finger1 | 1, 2 | 0, 1 |
| finger2 | 3, 4 | 2, 3 |
| finger3 | 5, 6 | 4, 5 |
| finger4 | 7, 8 | 6, 7 |

每根手指的两个舵机通过并联/差动机构共同决定运动，不能简单等同为两个完全独立的纯 flexion 和 abduction 轴。

## offset、invert 与限位

本地修改版映射为：

```text
position = clamp(model_target + offset, min, max)
if invert:
    command = -position
else:
    command = position
```

这里 invert 会连 offset 的结果一起取反，这是继承并保留的语义。

- `offset`：补偿装配与机械零位；
- `invert`：适配舵机安装方向；
- `min_position_rad` / `max_position_rad`：限制模型目标。

这些参数必须针对真实手标定。错误 offset 或 invert 可能让“算法看起来正确”的目标变成机构卡死。

## 原版控制边界

上游固定快照的 AHControl：

1. 使能扭矩；
2. 移动到 offset；
3. 初始化 Dora node；
4. 接收模型关节数组；
5. 应用 offset/invert；
6. sync write 八个目标。

它没有在运行循环中读取实际位置、负载、温度或电压，也没有接触状态机。这就是我后来选择在 AHControl 层加入反馈的原因。


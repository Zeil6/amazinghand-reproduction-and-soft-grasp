# 01 HandTracking：关键点到四个目标

## 输入与节拍

HandTracking 使用 OpenCV 打开摄像头。上游 dataflow 每 50 ms 发送一次 `tick`，理论感知频率约 20 Hz。

本地源码默认：

```python
cap = cv2.VideoCapture(0)
```

每个 tick：

1. 读取一帧；
2. 水平翻转；
3. 转 RGB；
4. 运行 MediaPipe Hands；
5. 过滤 handedness 与置信度；
6. 计算四个目标；
7. 通过 PyArrow/Dora 发送。

## 不是直接发送 21 个关键点

代码选取：

| 机械手 finger | 人手关键点 |
| --- | --- |
| tip1 | index fingertip - index MCP |
| tip2 | middle fingertip - middle MCP |
| tip3 | ring fingertip - ring MCP |
| tip4 | thumb tip - thumb MCP |

小拇指关键点用于构造掌心坐标系，但没有映射成第五根机械手手指，因为 AmazingHand 是四指结构。

## 掌心坐标系

代码用：

- wrist 到 middle MCP 形成一个轴；
- wrist 到 pinky/index MCP 提供掌平面方向；
- 叉乘得到法向；
- 组成旋转矩阵 `R`；
- 把四个 fingertip-MCP 向量变换到掌心参考系。

因此 `hand_pos` / `r_hand_pos` 的概念形状是：

```text
1 record × 4 named tips × 3 coordinates
```

它不是图像像素坐标，也不是八个舵机角。

## 当前局限

- handedness 和镜像关系写在代码逻辑中；
- 没有输出 timestamp、tracking validity 或目标年龄；
- 没有显式滤波；
- 检测失败时只是停止发布；
- 摄像头断开与手保持静止在下游不易区分；
- 掌心坐标系退化时缺少异常处理。

这些限制不会阻止 demo 运行，但会影响可靠遥操作。


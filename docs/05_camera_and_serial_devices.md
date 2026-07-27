# 05 摄像头与串口设备

## 摄像头

本地 HandTracking 源码使用：

```python
cap = cv2.VideoCapture(0)
```

这只表示选择枚举 index 0，不表示绑定某个固定物理摄像头。多个相机、USB 重新插拔或系统重启后，index 可能变化。

建议先检查：

```bash
v4l2-ctl --list-devices
ls -l /dev/v4l/by-id/
```

再用小脚本逐个打开 index，核对画面和分辨率。长期运行时，`/dev/v4l/by-id/` 比单纯记住 `0` 更可靠，但 OpenCV 是否能直接使用该路径需要实际验证。

## 串口

上游 dataflow 默认：

```text
/dev/ttyACM0
baudrate = 1_000_000
```

检查：

```bash
ls -l /dev/ttyACM*
udevadm info --query=property --name=/dev/ttyACM0
groups
```

常见问题包括：

- 当前用户不在 `dialout` 组；
- 设备重新插拔后从 `ttyACM0` 变成 `ttyACM1`；
- 另一个进程占用串口；
- 只有 USB 通信，没有给八个舵机提供合适的外部电源；
- 配置中的 ID、offset 或 invert 与实际装配不一致。

## 真机前的最小检查

1. 机械手远离物体；
2. 确认外部供电和公共地；
3. 只连接控制器，列出串口；
4. 核对 `r_hand.toml` 的 8 个 ID；
5. 低风险地检查零位和单指方向；
6. 确认没有第二个 Dora 或诊断程序占用同一串口；
7. 最后再启动完整视觉链路。

设备路径问题、权限问题和控制算法问题属于不同层级。看到“手不动”时，不能直接归因于 IK。


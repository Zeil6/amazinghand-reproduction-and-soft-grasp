# 06 问题分类与复盘

## 我复现中遇到的四类问题

| 大类 | 本次例子 | 首选证据 |
| --- | --- | --- |
| 包元数据与解释器边界 | Python 3.12.13 被 `<=3.12` 排除 | `python --version`、`pyproject.toml` |
| Rust 依赖图不一致 | `dora-message 0.4.4` 与 `0.6.0` 共存 | `Cargo.lock`、`cargo tree -d` |
| 系统动态库缺失 | OpenCV 找不到 `libGL.so.1` | 最早 traceback、`ldd`、`ldconfig` |
| 数据流级联错误 | 上游 `hand_tracker` 退出导致两个下游订阅失败 | Dora 事件时间顺序 |

摄像头 index、串口权限和设备路径则属于 I/O 与操作系统设备层。

## 以后我会怎样定位

```text
1. 固定现场
   保存命令、cwd、环境名、版本和完整日志

2. 找第一故障点
   不把级联错误当成多个独立根因

3. 判断层级
   Python metadata / Rust dependency / system library / Dora graph / device

4. 做最小检查
   import、cargo check、单节点、单设备

5. 一次只改一类变量
   改完记录差异，不同时升级全部依赖

6. 回到完整链路验证
   确认节点 ready、消息流和真机行为
```

## 为什么“之前能跑”不是反证

之前能运行，只能证明当时那套解释器、包、lockfile、系统动态库、设备枚举和启动方式能组合工作。隔一段时间后，任何一层发生漂移，都可能让同一条命令失败。

真正可复现需要同时记录：

- Git commit；
- `Cargo.lock`；
- conda 环境和 `pip freeze`；
- 系统库；
- dataflow YAML；
- 摄像头与串口身份；
- 硬件配置和外部供电。

这次复现让我把“重装一下试试”改成“先证明错误在哪一层”。这比记住某条一次性命令更可迁移。


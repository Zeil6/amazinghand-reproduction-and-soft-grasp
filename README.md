# AmazingHand 原版复现与排错记录

这个分支只记录原版 AmazingHand 的环境、构建、启动和设备问题。SCS0009 负载读取、接触判断、Holding/Fault、基线补偿以及软抓取阶段的串口竞争不在这里，它们只放在 [`soft-grasp-development`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp/tree/soft-grasp-development)。

我没有把所有报错压缩成一句“重装依赖即可”。这次复现最重要的收获，是先判断错误发生在哪一层，再决定检查环境、依赖树、dataflow 还是设备。

## 阅读顺序

1. [环境与 Python 版本](docs/01_environment_setup.md)
2. [原版复现流程](docs/02_reproduction_workflow.md)
3. [Cargo 构建与依赖问题](docs/03_build_and_dependency_issues.md)
4. [Dora 运行与级联错误](docs/04_dora_runtime_issues.md)
5. [摄像头和串口设备](docs/05_camera_and_serial_devices.md)
6. [问题分类与复盘](docs/06_debugging_review.md)

## 证据边界

| 内容 | 状态 |
| --- | --- |
| `dora-message` 多版本导致 Cargo 类型不匹配 | 有原始构建日志 |
| `libGL.so.1` 缺失导致 `hand_tracker` 首先退出 | 有原始 Dora 日志 |
| Python 3.12.13 与 `requires-python = ">=3.9,<=3.12"` 的比较问题 | 有聊天记录和上游固定源码 |
| Cargo 冲突最终执行的每一条修复命令 | 未完整保存，文档只记录能够确认的版本统一结果 |
| `libGL.so.1` 最终执行的系统安装命令 | 未保留，文档不伪造“已执行” |

原始日志包含本机绝对路径、局域网地址、Zenoh ZID 和 dataflow UUID。公开仓库只保留与根因直接相关的最小摘录，不上传整份未脱敏日志。

[返回 `main`](https://github.com/Zeil6/amazinghand-reproduction-and-soft-grasp)


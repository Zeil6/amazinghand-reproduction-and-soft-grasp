# 01 环境创建与 Python 版本

## 当时在做什么

我希望用 conda 管理 AmazingHand 环境，而不是直接使用 `uv`。实际创建的环境名是 `amazinghand`，Python 版本记录为 `3.12.13`。

## 现象：明明是 3.12，却被判不满足

安装 HandTracking 时，安装器给出的核心信息是：包要求 Python `>=3.9,<=3.12`，当前 `3.12.13` 不满足。

我最初把 `3.12` 理解成整个 3.12 系列。实际的版本比较会把 `3.12` 看成 `3.12.0`，因此：

```text
3.12.13 > 3.12.0
```

上游固定快照中的声明确实是：

```toml
requires-python = ">=3.9,<=3.12"
```

这更接近包元数据边界写法问题，而不是 Python 3.12.13 本身无法运行。若意图允许整个 3.12 系列，更清楚的写法应是：

```toml
requires-python = ">=3.9,<3.13"
```

相关上游文件：[HandTracking/pyproject.toml](https://github.com/pollen-robotics/AmazingHand/blob/3e8241074df3436a3044ced4881e3bb2133aa725/Demo/HandTracking/pyproject.toml)。

## 我后来怎样看这类问题

遇到 `requires-python` 报错时，我会先区分三件事：

1. 解释器实际版本是多少；
2. 包元数据允许的区间是什么；
3. 这是代码兼容性问题，还是仅仅元数据把补丁版本排除了。

建议检查：

```bash
conda activate amazinghand
python --version
python -m pip --version
python -m pip install -e HandTracking
```

本次材料没有保存我最终选择“修改本地元数据”还是“换成 3.12.0”的完整终端记录，因此这里只记录根因和可复核的修复方向，不把某条命令写成已执行事实。

## 为什么之前能运行，后来又缺库

后续再次运行时出现 `libGL.so.1` 缺失。这让我意识到，“同名 conda 环境还在”不等于“运行环境完全没变”：

- Python 包与 Ubuntu 系统动态库是两层依赖；
- `pip install -e` 只处理声明过的 Python 依赖，不会自动保证系统 `libGL` 存在；
- 可能切换了 shell、conda 环境或 Python 入口；
- 重新解析依赖后，OpenCV 变体可能发生变化；
- 系统清理、镜像更新或软件升级可能改变动态库；
- 未固定的包版本会让隔一段时间后的安装结果不同。

以后我会保存：

```bash
conda env export --from-history
python -m pip freeze
which python
which dora
ldd "$(python -c 'import cv2, pathlib; print(pathlib.Path(cv2.__file__).resolve())')" 
```

最后一条命令需要根据 OpenCV 实际 `.so` 路径调整。重点是同时记录 Python 层和系统动态库层，而不是只看 `pip list`。


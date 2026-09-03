"""Python 状态契约，对应 Rust `StateProvider`。

子类实现 `snapshot(key) -> dict | StateView` 与 `report(feedback)`，经 PyO3
反向包装成 Rust trait 对象。状态是 hint：可持有跨请求记忆，超时应返回空视图。
"""

from __future__ import annotations


class StateProvider:
    """状态槽插件契约。与算法侧 `AlgorithmProvider` 对位。"""

    name = "unnamed"

    def snapshot(self, key):
        raise NotImplementedError("{0} must implement snapshot()".format(type(self).__name__))

    def report(self, feedback):
        raise NotImplementedError("{0} must implement report()".format(type(self).__name__))

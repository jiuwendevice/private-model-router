from __future__ import annotations

"""Python 测试算法：通过 PyO3 的 ``register_algorithm`` 反向注册到 Rust。

不是生产默认路径。示例与验证用 `CostAwareAlgorithm`，再 `register_algorithm`。
公共契约在 `openjiuwen.algorithm_provider.AlgorithmProvider`。
"""

from .cost_aware_algorithm import CostAwareAlgorithm

__all__ = ["CostAwareAlgorithm"]

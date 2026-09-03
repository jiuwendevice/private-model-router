from __future__ import annotations

"""外部团队供稿 SDK。契约定义在 `openjiuwen.algorithm.algorithm_provider`。

内置示意实现不在这里，见 `openjiuwen.algorithm.test_algorithm`。
"""

from openjiuwen.algorithm.algorithm_provider import AlgorithmProvider as PyAlgorithm
from openjiuwen.algorithm.algorithm_provider import check_purity

__all__ = ["PyAlgorithm", "check_purity"]

from __future__ import annotations

"""外部团队供稿 SDK。契约定义在 `openjiuwen.algorithm_provider`。

测试算法实现不在这里，见顶层包 `test_algo`。
"""

from openjiuwen.algorithm_provider import AlgorithmProvider as PyAlgorithm
from openjiuwen.algorithm_provider import check_purity

__all__ = ["PyAlgorithm", "check_purity"]

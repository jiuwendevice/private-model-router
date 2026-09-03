from __future__ import annotations

"""示意算法：与 crates/algorithms 同名同功能的 Python 骨架实现。

不是生产默认路径。云侧实验 / 文档示例从这里取类，再 `register_algorithm`。
公共契约在上一层 `algorithm.algorithm_provider.AlgorithmProvider`。
"""

from openjiuwen.algorithm.test_algorithm.ensemble import Ensemble
from openjiuwen.algorithm.test_algorithm.passthrough import Passthrough
from openjiuwen.algorithm.test_algorithm.rule_cascade import RuleCascade
from openjiuwen.algorithm.test_algorithm.signal import Signal
from openjiuwen.algorithm.test_algorithm.weighted import Weighted

REGISTRY = {
    "passthrough": Passthrough,
    "weighted": Weighted,
    "rule_cascade": RuleCascade,
    "signal": Signal,
    "ensemble": Ensemble,
}

__all__ = [
    "REGISTRY",
    "Passthrough",
    "Weighted",
    "RuleCascade",
    "Signal",
    "Ensemble",
]

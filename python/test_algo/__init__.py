from __future__ import annotations

"""Python 测试算法：通过 PyO3 的 ``register_algorithm`` 反向注册到 Rust。

不是生产默认路径。云侧实验 / 文档示例从这里取类，再 `register_algorithm`。
公共契约在 `openjiuwen.algorithm_provider.AlgorithmProvider`。
"""

from .ensemble import Ensemble
from .passthrough import Passthrough
from .rule_cascade import RuleCascade
from .signal import Signal
from .weighted import Weighted

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

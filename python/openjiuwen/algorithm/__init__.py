from __future__ import annotations

"""内置 Python 算法注册表：算法名 → 算法类。

与 crates/algorithms 同名同功能；构建配置按算法名选 rust 或 python，避免双份入产物。
"""

from openjiuwen.algorithm.ensemble import Ensemble
from openjiuwen.algorithm.passthrough import Passthrough
from openjiuwen.algorithm.rule_cascade import RuleCascade
from openjiuwen.algorithm.signal import Signal
from openjiuwen.algorithm.weighted import Weighted

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

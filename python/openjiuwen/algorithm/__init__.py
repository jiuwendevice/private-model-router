from __future__ import annotations

"""Python 算法包：公共契约在 `algorithm_provider`，示意实现在 `test_algorithm/`。

`AlgorithmProvider` / `check_purity` 是对外接口。passthrough 等只是骨架示意。
"""

from openjiuwen.algorithm.algorithm_provider import (
    Algorithm,
    AlgorithmProvider,
    check_purity,
)
from openjiuwen.algorithm.test_algorithm import (
    REGISTRY,
    Ensemble,
    Passthrough,
    RuleCascade,
    Signal,
    Weighted,
)

__all__ = [
    "AlgorithmProvider",
    "Algorithm",
    "check_purity",
    "REGISTRY",
    "Passthrough",
    "Weighted",
    "RuleCascade",
    "Signal",
    "Ensemble",
]

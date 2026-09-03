"""另一个外部团队的最小示例：选过滤后目录里的最后一个目标。"""

from __future__ import annotations

from typing import Any

from ..algorithm_provider import AlgorithmProvider


class LastAvailableAlgorithm(AlgorithmProvider):
    """与 CostAware 对照：不看成本，取 ``ctx.targets`` 的末项。"""

    name = "python_last_available"

    def decide(self, request: Any, ctx: Any):
        del request
        if not ctx.targets:
            raise ValueError("no available target")
        selected = ctx.targets[-1]
        return {
            "selected_model_id": selected,
            "reasoning": "python_last_available: last remaining target",
            "is_answer_call": True,
        }

"""团队用 Python 编写路由算法的最小示例。"""

from __future__ import annotations

from types import MappingProxyType
from typing import Any, Mapping

from ..algorithm_provider import AlgorithmProvider


class CostAwareAlgorithm(AlgorithmProvider):
    """从 Rust runtime 提供的候选目标中选择成本最低者。

    成本表是类属性；换表就再写一个子类，不要给构造函数传参。
    """

    name = "python_cost_aware"
    costs: Mapping[str, float] = MappingProxyType({})

    def decide(self, request: Any, ctx: Any):
        del request
        if not ctx.targets:
            raise ValueError("no available target")
        table = type(self).costs
        selected = min(
            ctx.targets,
            key=lambda target: (table.get(target, float("inf")), target),
        )
        return {
            "selected_model_id": selected,
            "reasoning": "python_cost_aware: lowest configured cost",
            "is_answer_call": True,
        }

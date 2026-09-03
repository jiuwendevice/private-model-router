"""团队用 Python 编写路由算法的最小示例。"""

from __future__ import annotations

from types import MappingProxyType
from typing import Any, Mapping, Optional

from ..algorithm_provider import AlgorithmProvider


class CostAwareAlgorithm(AlgorithmProvider):
    """从 Rust runtime 提供的候选目标中选择成本最低者。"""

    name = "python_cost_aware"

    def __init__(self, costs: Optional[Mapping[str, float]] = None) -> None:
        # 只读配置；decide 不修改 self，不读时钟，不做 I/O。
        self._costs = MappingProxyType(dict(costs or ()))

    def decide(self, request: Any, ctx: Any):
        del request  # 该策略只需要已过滤的目标集。
        if not ctx.targets:
            raise ValueError("no available target")

        selected = min(
            ctx.targets,
            key=lambda target: (self._costs.get(target, float("inf")), target),
        )
        return {
            "selected_model_id": selected,
            "reasoning": "python_cost_aware: lowest configured cost",
            "is_answer_call": True,
        }

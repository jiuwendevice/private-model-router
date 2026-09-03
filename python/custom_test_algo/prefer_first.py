"""宿主用已安装的 openjiuwen 编写自定义算法：选过滤后的第一个目标。"""

from __future__ import annotations

from typing import Any

from openjiuwen import AlgorithmProvider


class PreferFirstAlgorithm(AlgorithmProvider):
    """包外算法：导入本类即按 ``name`` 写入槽位。"""

    name = "custom_prefer_first"

    def decide(self, request: Any, ctx: Any):
        del request
        if not ctx.targets:
            raise ValueError("no available target")
        selected = ctx.targets[0]
        return {
            "selected_model_id": selected,
            "reasoning": "custom_prefer_first: first remaining target",
            "is_answer_call": True,
        }

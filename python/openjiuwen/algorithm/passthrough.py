from __future__ import annotations

"""与 Rust passthrough.rs 同功能：选第一个可用目标。"""

from openjiuwen.contrib import PyAlgorithm


class Passthrough(PyAlgorithm):
    name = "passthrough"

    def decide(self, request, ctx):
        targets = list(getattr(ctx, "targets", []) or [])
        exclusions = set(getattr(request, "exclusions", []) or [])
        for model in targets:
            if model not in exclusions:
                return {
                    "selected_model_id": model,
                    "reasoning": "passthrough: first available target",
                    "is_answer_call": True,
                }
        raise ValueError("no available target after exclusions")

from __future__ import annotations

from openjiuwen.algorithm_provider import AlgorithmProvider
from .passthrough import Passthrough


class RuleCascade(AlgorithmProvider):
    name = "rule_cascade"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "rule_cascade: stub, falls back to first target"
        return decision

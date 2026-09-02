from __future__ import annotations

from openjiuwen.algorithm.passthrough import Passthrough
from openjiuwen.contrib import PyAlgorithm


class RuleCascade(PyAlgorithm):
    name = "rule_cascade"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "rule_cascade: stub, falls back to first target"
        return decision

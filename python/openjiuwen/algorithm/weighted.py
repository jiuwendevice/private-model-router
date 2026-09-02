from __future__ import annotations

from openjiuwen.algorithm.passthrough import Passthrough
from openjiuwen.contrib import PyAlgorithm


class Weighted(PyAlgorithm):
    name = "weighted"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "weighted: stub, falls back to first target"
        return decision

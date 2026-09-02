from __future__ import annotations

from openjiuwen.algorithm.passthrough import Passthrough
from openjiuwen.contrib import PyAlgorithm


class Ensemble(PyAlgorithm):
    name = "ensemble"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "ensemble: stub, falls back to first target"
        return decision

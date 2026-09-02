from __future__ import annotations

from openjiuwen.algorithm.passthrough import Passthrough
from openjiuwen.contrib import PyAlgorithm


class Signal(PyAlgorithm):
    name = "signal"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "signal: stub, falls back to first target"
        return decision

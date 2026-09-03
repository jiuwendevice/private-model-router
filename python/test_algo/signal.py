from __future__ import annotations

from openjiuwen.algorithm_provider import AlgorithmProvider
from .passthrough import Passthrough


class Signal(AlgorithmProvider):
    name = "signal"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "signal: stub, falls back to first target"
        return decision

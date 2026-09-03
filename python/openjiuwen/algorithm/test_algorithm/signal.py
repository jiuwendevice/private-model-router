from __future__ import annotations

from openjiuwen.algorithm.algorithm_provider import AlgorithmProvider
from openjiuwen.algorithm.test_algorithm.passthrough import Passthrough


class Signal(AlgorithmProvider):
    name = "signal"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "signal: stub, falls back to first target"
        return decision

from __future__ import annotations

from openjiuwen.algorithm_provider import AlgorithmProvider
from .passthrough import Passthrough


class Weighted(AlgorithmProvider):
    name = "weighted"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "weighted: stub, falls back to first target"
        return decision

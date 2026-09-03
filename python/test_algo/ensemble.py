from __future__ import annotations

from openjiuwen.algorithm_provider import AlgorithmProvider
from .passthrough import Passthrough


class Ensemble(AlgorithmProvider):
    name = "ensemble"

    def decide(self, request, ctx):
        inner = Passthrough()
        decision = inner.decide(request, ctx)
        decision["reasoning"] = "ensemble: stub, falls back to first target"
        return decision

from __future__ import annotations

import openjiuwen
from openjiuwen.discover import bundled_algorithms
from openjiuwen.test_algo.cost_aware import CostAwareAlgorithm
from openjiuwen.test_algo2.last_available import LastAvailableAlgorithm


def test_package_layout():
    algo = CostAwareAlgorithm({"a": 2.0, "b": 1.0})
    class _Ctx:
        targets = ["a", "b"]
    class _Req:
        exclusions = []
    decision = algo.decide(_Req(), _Ctx())
    assert decision["selected_model_id"] == "b"
    openjiuwen.check_purity(algo, _Req(), _Ctx())
    assert CostAwareAlgorithm in bundled_algorithms()
    assert LastAvailableAlgorithm in bundled_algorithms()
    assert issubclass(CostAwareAlgorithm, openjiuwen.AlgorithmProvider)
    assert issubclass(LastAvailableAlgorithm, openjiuwen.AlgorithmProvider)
    assert openjiuwen.Algorithm is openjiuwen.AlgorithmProvider

from __future__ import annotations

import openjiuwen
import test_algo


def test_package_layout():
    algo = test_algo.CostAwareAlgorithm({"a": 2.0, "b": 1.0})
    class _Ctx:
        targets = ["a", "b"]
    class _Req:
        exclusions = []
    decision = algo.decide(_Req(), _Ctx())
    assert decision["selected_model_id"] == "b"
    openjiuwen.check_purity(algo, _Req(), _Ctx())
    from test_algo import CostAwareAlgorithm as Sample
    assert Sample is test_algo.CostAwareAlgorithm
    assert issubclass(Sample, openjiuwen.AlgorithmProvider)
    assert openjiuwen.Algorithm is openjiuwen.AlgorithmProvider

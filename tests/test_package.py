from __future__ import annotations

import openjiuwen
import openjiuwen.algorithm
import openjiuwen.contrib


def test_package_layout():
    assert "passthrough" in openjiuwen.algorithm.REGISTRY
    algo = openjiuwen.algorithm.Passthrough()
    class _Ctx:
        targets = ["a", "b"]
    class _Req:
        exclusions = []
    decision = algo.decide(_Req(), _Ctx())
    assert decision["selected_model_id"] == "a"
    openjiuwen.algorithm.check_purity(algo, _Req(), _Ctx())
    openjiuwen.contrib.check_purity(algo, _Req(), _Ctx())
    from openjiuwen.algorithm.test_algorithm import Passthrough as Sample
    assert Sample is openjiuwen.algorithm.Passthrough
    assert issubclass(Sample, openjiuwen.algorithm.AlgorithmProvider)
    assert openjiuwen.algorithm.Algorithm is openjiuwen.algorithm.AlgorithmProvider

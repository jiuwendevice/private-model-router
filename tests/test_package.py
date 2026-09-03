from __future__ import annotations

import openjiuwen
import openjiuwen.contrib
import test_algo


def test_package_layout():
    assert "passthrough" in test_algo.REGISTRY
    algo = test_algo.Passthrough()
    class _Ctx:
        targets = ["a", "b"]
    class _Req:
        exclusions = []
    decision = algo.decide(_Req(), _Ctx())
    assert decision["selected_model_id"] == "a"
    openjiuwen.check_purity(algo, _Req(), _Ctx())
    openjiuwen.contrib.check_purity(algo, _Req(), _Ctx())
    from test_algo import Passthrough as Sample
    assert Sample is test_algo.Passthrough
    assert issubclass(Sample, openjiuwen.AlgorithmProvider)
    assert openjiuwen.Algorithm is openjiuwen.AlgorithmProvider

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
    openjiuwen.contrib.check_purity(algo, _Req(), _Ctx())

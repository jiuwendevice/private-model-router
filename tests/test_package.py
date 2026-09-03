from __future__ import annotations

import openjiuwen


class _ExampleAlgorithm(openjiuwen.PyAlgorithm):
    name = "example"

    def decide(self, request, ctx):
        return {
            "selected_model_id": ctx.targets[0],
            "reasoning": "package smoke test",
            "is_answer_call": True,
        }


class _Context:
    targets = ["a", "b"]


def test_package_root_exposes_python_algorithm_contract():
    algorithm = _ExampleAlgorithm()
    decision = openjiuwen.check_purity(algorithm, object(), _Context())
    assert decision["selected_model_id"] == "a"
    assert "PyAlgorithm" in openjiuwen.__all__
    assert "register_algorithm" in openjiuwen.__all__

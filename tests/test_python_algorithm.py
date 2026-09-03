from __future__ import annotations

import pytest  # type: ignore[import-not-found]

from openjiuwen import AlgorithmProvider, check_purity
from openjiuwen.test_algo.cost_aware import CostAwareAlgorithm
from openjiuwen.test_algo2.last_available import LastAvailableAlgorithm


class _Context:
    targets = ["fast-expensive", "slow-cheap"]


class CheapSlow(CostAwareAlgorithm):
    name = "python_cheap_slow"
    costs = {"fast-expensive": 10.0, "slow-cheap": 1.0}


def test_python_algorithm_is_pure():
    decision = check_purity(CheapSlow(), object(), _Context(), rounds=3)
    assert decision["selected_model_id"] == "slow-cheap"


def test_algorithm_provider_requires_name():
    with pytest.raises(TypeError, match="name"):
        class MissingName(AlgorithmProvider):
            def decide(self, request, ctx):
                del request, ctx
                return {"selected_model_id": "x", "reasoning": "x", "is_answer_call": True}


def test_algorithm_provider_requires_decide():
    with pytest.raises(TypeError, match="decide"):
        class MissingDecide(AlgorithmProvider):
            name = "missing_decide"


def test_algorithm_provider_requires_no_arg_constructor():
    with pytest.raises(TypeError, match="no arguments"):
        class NeedsArgs(AlgorithmProvider):
            name = "needs_args"

            def __init__(self, costs):
                self.costs = costs

            def decide(self, request, ctx):
                del request, ctx
                return {"selected_model_id": "x", "reasoning": "x", "is_answer_call": True}


def test_bundled_algorithm_is_installed_on_import(tmp_path):
    pytest.importorskip(
        "openjiuwen._openjiuwen",
        reason="run `maturin develop` before the PyO3 integration test",
    )
    from openjiuwen import Router

    config = tmp_path / "python-algorithm.toml"
    config.write_text(
        """
algorithm = "python_cost_aware"
[state]
backend = "memory"
[targets]
models = ["fast-expensive", "slow-cheap"]
""".strip(),
        encoding="utf-8",
    )

    router = Router.from_config(str(config))
    decision = router.route_sync({})
    assert router.algorithm_name() == CostAwareAlgorithm.name
    # 无参默认安装：成本均为 inf，按目标名排序取最小。
    assert decision.selected_model_id == "fast-expensive"


def test_second_team_algorithm_is_installed_on_import(tmp_path):
    pytest.importorskip(
        "openjiuwen._openjiuwen",
        reason="run `maturin develop` before the PyO3 integration test",
    )
    from openjiuwen import Router

    config = tmp_path / "python-algorithm.toml"
    config.write_text(
        """
algorithm = "python_last_available"
[state]
backend = "memory"
[targets]
models = ["fast-expensive", "slow-cheap"]
""".strip(),
        encoding="utf-8",
    )

    router = Router.from_config(str(config))
    decision = router.route_sync({})
    assert router.algorithm_name() == LastAvailableAlgorithm.name
    assert decision.selected_model_id == "slow-cheap"
    assert decision.reasoning == "python_last_available: last remaining target"


def test_python_algorithm_round_trips_through_rust(tmp_path):
    pytest.importorskip(
        "openjiuwen._openjiuwen",
        reason="run `maturin develop` before the PyO3 integration test",
    )
    from openjiuwen import Router

    config = tmp_path / "python-algorithm.toml"
    config.write_text(
        """
algorithm = "python_cheap_slow"
[state]
backend = "memory"
[targets]
models = ["fast-expensive", "slow-cheap"]
""".strip(),
        encoding="utf-8",
    )

    router = Router.from_config(str(config))
    decision = router.route_sync({})
    assert router.algorithm_name() == CheapSlow.name
    assert decision.selected_model_id == "slow-cheap"
    assert decision.reasoning == "python_cost_aware: lowest configured cost"

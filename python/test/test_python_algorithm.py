from __future__ import annotations

import pytest

from cost_aware_algorithm import CostAwareAlgorithm
from openjiuwen import check_purity, unregister_algorithm


class _Context:
    targets = ["fast-expensive", "slow-cheap"]


def test_python_algorithm_is_pure():
    algorithm = CostAwareAlgorithm(
        {"fast-expensive": 10.0, "slow-cheap": 1.0}
    )
    decision = check_purity(algorithm, object(), _Context(), rounds=3)
    assert decision["selected_model_id"] == "slow-cheap"


def test_python_algorithm_round_trips_through_rust(tmp_path):
    pytest.importorskip(
        "openjiuwen._openjiuwen",
        reason="run `maturin develop` before the PyO3 integration test",
    )
    from openjiuwen import Router

    algorithm = CostAwareAlgorithm(
        {"fast-expensive": 10.0, "slow-cheap": 1.0}
    )
    assert algorithm.register() == algorithm.name

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

    try:
        # from_config 在 Rust 注册表找到 Python 对象，将其包装为
        # Algorithm trait。route() 进入 Rust 决策循环后再回调 decide()。
        router = Router.from_config(str(config))
        decision = router.route()
        assert router.algorithm_name() == algorithm.name
        assert decision.selected_model_id == "slow-cheap"
        assert decision.reasoning == "python_cost_aware: lowest configured cost"
    finally:
        unregister_algorithm(algorithm.name)

from __future__ import annotations

import pytest  # type: ignore[import-not-found]

from custom_test_algo import PreferFirstAlgorithm
from openjiuwen import check_purity
from openjiuwen.discover import bundled_algorithms


def test_custom_algorithm_is_not_auto_installed():
    assert PreferFirstAlgorithm not in bundled_algorithms()


def test_custom_algorithm_is_pure():
    class _Ctx:
        targets = ["fast-expensive", "slow-cheap"]

    decision = check_purity(PreferFirstAlgorithm(), object(), _Ctx(), rounds=3)
    assert decision["selected_model_id"] == "fast-expensive"


def test_custom_algorithm_auto_registers_on_import(tmp_path):
    pytest.importorskip(
        "openjiuwen._openjiuwen",
        reason="run `maturin develop` before the PyO3 integration test",
    )
    from openjiuwen import Router

    config = tmp_path / "custom-algorithm.toml"
    config.write_text(
        """
algorithm = "custom_prefer_first"
[state]
backend = "memory"
[targets]
models = ["fast-expensive", "slow-cheap"]
""".strip(),
        encoding="utf-8",
    )

    router = Router.from_config(str(config))
    decision = router.route_sync({})
    assert router.algorithm_name() == PreferFirstAlgorithm.name
    assert decision.selected_model_id == "fast-expensive"
    assert decision.reasoning == "custom_prefer_first: first remaining target"

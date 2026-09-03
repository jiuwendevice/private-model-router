"""使用随包默认的 ``test_algo.CostAwareAlgorithm``，不调用 ``register_algorithm``。"""

from __future__ import annotations

import pytest  # type: ignore[import-not-found]

pytest.importorskip("openjiuwen._openjiuwen")

from openjiuwen import Feedback, Outcome, Router
from openjiuwen.discover import bundled_algorithms
from openjiuwen.test_algo.cost_aware import CostAwareAlgorithm

PROFILE = {
    "algorithm": "python_cost_aware",
    "state": {"backend": "memory"},
    "targets": {"models": ["fast-expensive", "slow-cheap"]},
}


def test_default_test_algo_is_bundled():
    assert CostAwareAlgorithm in bundled_algorithms()
    assert CostAwareAlgorithm.name == "python_cost_aware"


def test_default_test_algo_from_config_without_register():
    router = Router.from_config(PROFILE)
    assert router.algorithm_name() == "python_cost_aware"
    decision = router.route_sync(
        {
            "messages": [{"role": "user", "content": "hi"}],
            "session_id": "s-default",
            "agent_id": "host",
        }
    )
    # 无参默认安装：成本均为 inf，按目标名取最小。
    assert decision.selected_model_id == "fast-expensive"
    assert decision.reasoning == "python_cost_aware: lowest configured cost"
    assert decision.is_answer_call is True


def test_default_test_algo_report_unavailable_excludes_on_next_route():
    router = Router.from_config(PROFILE)
    req = {
        "messages": [{"role": "user", "content": "hi"}],
        "session_id": "s-exclude",
        "agent_id": "host",
    }
    first = router.route_sync(req)
    assert first.selected_model_id == "fast-expensive"
    router.report_sync(
        Feedback.ok(
            first,
            latency_ms=1,
            session_id="s-exclude",
            agent_id="host",
            outcome=Outcome.UNAVAILABLE,
        )
    )
    second = router.route_sync(req)
    assert second.selected_model_id == "slow-cheap"
    assert second.reasoning == "python_cost_aware: lowest configured cost"

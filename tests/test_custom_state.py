from __future__ import annotations

import pytest  # type: ignore[import-not-found]

from openjiuwen import Outcome, StateProvider
from openjiuwen.test_algo.cost_aware import CostAwareAlgorithm


class ExclusionStore(StateProvider):
    """把 UNAVAILABLE 写入排除表；与 Rust MemoryState 行为对齐的最小 Python 实现。"""

    name = "python_exclusion_store"

    def __init__(self):
        self._exclusions = {}

    def snapshot(self, key):
        slot = (key.session_id, key.agent_id)
        return {"exclusions": list(self._exclusions.get(slot, [])), "affinity": None}

    def report(self, feedback):
        if feedback.outcome != Outcome.UNAVAILABLE:
            return
        slot = (feedback.key.session_id, feedback.key.agent_id)
        seen = self._exclusions.setdefault(slot, [])
        if feedback.selected_model_id not in seen:
            seen.append(feedback.selected_model_id)


def test_python_state_provider_is_not_an_algorithm():
    assert not hasattr(ExclusionStore(), "decide")
    assert issubclass(ExclusionStore, StateProvider)


def test_custom_state_injected_on_from_config():
    pytest.importorskip("openjiuwen._openjiuwen")
    from openjiuwen import Feedback, Router

    store = ExclusionStore()
    router = Router.from_config(
        {
            "algorithm": "passthrough",
            "state": {"backend": "memory"},
            "targets": {"models": ["fast-local", "strong-cloud"]},
        },
        state=store,
    )
    req = {"session_id": "s-py-state", "agent_id": "host"}
    first = router.route_sync(req)
    assert first.selected_model_id == "fast-local"
    router.report_sync(
        Feedback.ok(
            first,
            latency_ms=1,
            session_id="s-py-state",
            agent_id="host",
            outcome=Outcome.UNAVAILABLE,
        )
    )
    second = router.route_sync(req)
    assert second.selected_model_id == "strong-cloud"


def test_register_state_selected_by_backend_name():
    pytest.importorskip("openjiuwen._openjiuwen")
    from openjiuwen import Feedback, Router, register_state

    store = ExclusionStore()
    assert register_state(store) == "python_exclusion_store"
    router = Router.from_config(
        {
            "algorithm": "passthrough",
            "state": {"backend": "python_exclusion_store"},
            "targets": {"models": ["fast-local", "strong-cloud"]},
        }
    )
    req = {"session_id": "s-named-state", "agent_id": "host"}
    first = router.route_sync(req)
    router.report_sync(
        Feedback.ok(
            first,
            latency_ms=1,
            session_id="s-named-state",
            agent_id="host",
            outcome=Outcome.UNAVAILABLE,
        )
    )
    second = router.route_sync(req)
    assert second.selected_model_id == "strong-cloud"


def test_custom_state_works_with_python_algorithm():
    pytest.importorskip("openjiuwen._openjiuwen")
    from openjiuwen import Router

    router = Router.from_config(
        {
            "algorithm": "python_cost_aware",
            "state": {"backend": "memory"},
            "targets": {"models": ["fast-expensive", "slow-cheap"]},
        },
        state=ExclusionStore(),
    )
    assert router.algorithm_name() == CostAwareAlgorithm.name
    decision = router.route_sync({"session_id": "s-mix", "agent_id": "host"})
    assert decision.selected_model_id == "fast-expensive"

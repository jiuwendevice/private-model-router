from __future__ import annotations

import pytest

pytest.importorskip("openjiuwen._openjiuwen")

import openjiuwen
from openjiuwen import Feedback, Message, Outcome, RequestMetadata, RouteHint, RouteRequest, Router


PROFILE = {
    "algorithm": "passthrough",
    "state": {"backend": "memory"},
    "targets": {"models": ["fast-local", "strong-cloud"]},
}


def test_from_config_dict_routes_first_target():
    router = Router.from_config(PROFILE)
    assert router.algorithm_name() == "passthrough"
    decision = router.route_sync(
        {
            "messages": [{"role": "user", "content": "hi"}],
            "session_id": "s1",
            "agent_id": "a1",
        }
    )
    assert decision.selected_model_id == "fast-local"
    assert decision.target == "fast-local"
    assert decision.is_answer_call is True


def test_report_unavailable_excludes_on_next_route():
    router = Router.from_config(PROFILE)
    req = RouteRequest(
        messages=[Message("user", "hi")],
        metadata=RequestMetadata(session_id="s1", agent_id="a1"),
    )
    first = router.route_sync(req, RouteHint())
    assert first.selected_model_id == "fast-local"
    router.report_sync(
        Feedback.ok(
            first,
            latency_ms=1,
            key=req.routing_key(),
            outcome=Outcome.UNAVAILABLE,
        )
    )
    second = router.route_sync(req)
    assert second.selected_model_id == "strong-cloud"


def test_async_route_and_report():
    import asyncio

    async def body():
        router = Router.from_config(PROFILE)
        decision = await router.route({"session_id": "async", "agent_id": "a"})
        assert decision.selected_model_id == "fast-local"
        await router.report(
            Feedback.ok(decision, latency_ms=3, session_id="async", agent_id="a")
        )

    asyncio.run(body())


def test_python_algorithm_register_and_replace():
    from openjiuwen.algorithm.test_algorithm import Passthrough

    openjiuwen.register_algorithm(Passthrough())
    router = Router.from_toml(
        """
algorithm = "passthrough"
[state]
backend = "memory"
[targets]
models = ["alpha", "beta"]
"""
    )
    router.replace_algorithm(Passthrough())
    decision = router.route_sync({"exclusions": ["alpha"]})
    assert decision.selected_model_id == "beta"
    assert "passthrough" in decision.reasoning


def test_state_client_constructs():
    client = openjiuwen.StateClient("http://127.0.0.1:9", timeout_ms=5)
    assert client.endpoint == "http://127.0.0.1:9"
    router = Router.from_config(
        {
            "algorithm": "passthrough",
            "state": {"backend": "remote", "endpoint": "http://ignored"},
            "targets": {"models": ["only"]},
        },
        state=client,
    )
    assert router.route_sync({}).selected_model_id == "only"

from __future__ import annotations

import pytest  # type: ignore[import-not-found]

pytest.importorskip("openjiuwen._openjiuwen")

from react_agent import ReActAgent, build_router


def test_react_agent_routes_retries_and_answers():
    router = build_router()
    assert router.algorithm_name() == "passthrough"
    answer, trace = ReActAgent(router).run("What is 21 * 2?")
    assert answer == "42"
    assert trace == [
        "fast-local",
        "strong-cloud",
        "strong-cloud",
    ]

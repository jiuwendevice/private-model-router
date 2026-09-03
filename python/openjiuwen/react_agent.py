"""最小 ReAct 宿主示例：决策与执行分离。

`python/openjiuwen` 不是「用 Python 实现路由内核」的方案。它是云侧宿主门面：
经 PyO3 调 Rust `Router`。本模块演示宿主自己跑 ReAct 循环——每次调模型前
`route`，调用后 `report`；模型由 mock 扮演，不发真实网络。

与 `tests/react_agent.rs` 同一条剧本：

    python -m openjiuwen.react_agent
"""

from __future__ import annotations

from openjiuwen import (
    Feedback,
    Message,
    Outcome,
    RequestMetadata,
    RouteRequest,
    Router,
    RoutingKey,
)

PROFILE = {
    "algorithm": "passthrough",
    "state": {"backend": "memory"},
    "targets": {"models": ["fast-local", "strong-cloud"]},
}


class MockBackend:
    """`fast-local` 不可用；`strong-cloud` 按 ReAct 剧本回复。"""

    def invoke(self, model, prompt):
        if model == "fast-local":
            raise Unavailable("unavailable")
        if "Observation:" in prompt:
            return "Thought: I have the result.\nFinal Answer: 42"
        return "Thought: I should calculate.\nAction: calc[21*2]"


class Unavailable(Exception):
    pass


def _line_after(text, prefix):
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith(prefix):
            return stripped[len(prefix):].strip()
    return None


def _split_tool(action):
    if "[" not in action:
        return action, ""
    name, rest = action.split("[", 1)
    return name.strip(), rest.rstrip("]").strip()


def parse_turn(text):
    thought = _line_after(text, "Thought:") or ""
    answer = _line_after(text, "Final Answer:")
    if answer is not None:
        return ("finish", thought, answer)
    action = _line_after(text, "Action:")
    if action is None:
        raise ValueError("mock llm must emit Action or Final Answer")
    tool, arg = _split_tool(action)
    return ("act", thought, tool, arg)


def run_tool(name, arg):
    if name != "calc":
        raise AssertionError("this skeleton only ships a calc tool")
    lhs, rhs = arg.split("*", 1)
    return str(int(lhs.strip()) * int(rhs.strip()))


class ReActAgent:
    """宿主侧最小 ReAct：路由选模 → 自己调模型 → 自己跑工具。"""

    def __init__(self, router: Router, session_id="sess-react", agent_id="react-agent"):
        self.router = router
        self.backend = MockBackend()
        self.session_id = session_id
        self.agent_id = agent_id

    def routing_key(self):
        return RoutingKey(session_id=self.session_id, agent_id=self.agent_id)

    def _request(self, question):
        return RouteRequest(
            messages=[Message("user", question)],
            metadata=RequestMetadata(
                session_id=self.session_id,
                agent_id=self.agent_id,
            ),
        )

    def route(self, question, trace):
        decision = self.router.route_sync(self._request(question))
        trace.append(decision.selected_model_id)
        print("  route → {0} ({1})".format(decision.selected_model_id, decision.reasoning))
        return decision

    def report(self, model, outcome):
        self.router.report_sync(
            Feedback(
                self.routing_key(),
                model,
                outcome=outcome,
                latency_ms=1,
            )
        )

    def call_model(self, prompt, trace):
        for _ in range(4):
            decision = self.route(prompt, trace)
            try:
                text = self.backend.invoke(decision.selected_model_id, prompt)
            except Unavailable as exc:
                print("  {0} failed ({1}), report Unavailable".format(
                    decision.selected_model_id, exc
                ))
                self.report(decision.selected_model_id, Outcome.UNAVAILABLE)
                continue
            self.report(decision.selected_model_id, Outcome.OK)
            return decision, text
        raise LookupError("no available target after exclusions")

    def run(self, question):
        prompt = "Question: {0}\n".format(question)
        trace = []
        print("ReAct: {0}".format(question))
        for step in range(1, 5):
            print("step {0}".format(step))
            _decision, text = self.call_model(prompt, trace)
            prompt += text + "\n"
            parsed = parse_turn(text)
            if parsed[0] == "finish":
                _kind, thought, answer = parsed
                print("  thought: {0}".format(thought))
                print("  final: {0}".format(answer))
                return answer, trace
            _kind, thought, tool, arg = parsed
            observation = run_tool(tool, arg)
            print("  thought: {0}".format(thought))
            print("  action: {0}[{1}] → {2}".format(tool, arg, observation))
            prompt += "Observation: {0}\n".format(observation)
        raise RuntimeError("max ReAct steps exceeded")


def build_router() -> Router:
    return Router.from_config(PROFILE)


def main():
    router = build_router()
    answer, trace = ReActAgent(router).run("What is 21 * 2?")
    print("trace: {0}".format(trace))
    print("answer: {0}".format(answer))
    return answer, trace


if __name__ == "__main__":
    main()

"""Python 宿主集成示例：如何在一个 Python 程序中接入 openjiuwen-router。

演示宿主侧的标准闭环：

    构造请求 → router.route_sync 选模型 → 宿主自己调模型 → router.report_sync 回报
    失败时把模型加入 exclusions 并回报 Unavailable，下一次 route 自动换模

运行前提（在仓库根目录执行）：

    maturin develop
    python examples/python_integration.py

模型调用由 mock 扮演，不发真实网络；接入真实业务时把 call_model 换成
你的模型客户端即可，路由相关代码不需要改动。
"""

from __future__ import annotations

from openjiuwen import Feedback, Outcome, Router

# 与 config/edge.toml 等价的 dict 配置；也可以直接传 TOML 文件路径。
PROFILE = {
    "algorithm": "passthrough",
    "state": {"backend": "memory", "ttl_secs": 300, "max_entries": 1024},
    "targets": {"models": ["fast-local", "strong-cloud"]},
}

# 稳定的会话标识：route 与 report 必须使用同一组，否则状态无法闭环。
SESSION_ID = "demo-session"
AGENT_ID = "python-host-example"


class ModelUnavailable(Exception):
    """模拟模型侧故障；真实场景对应超时、限流、5xx 等。"""


def call_model(model_id, messages):
    """模拟模型客户端：fast-local 不可用，strong-cloud 正常响应。"""
    if model_id == "fast-local":
        raise ModelUnavailable(model_id)
    return "[{0}] {1}".format(model_id, messages[-1]["content"])


def invoke_with_routing(router, messages, max_attempts=4):
    """宿主调用模板：每次调用前 route，调用后 report，失败排除后重试。"""
    exclusions = []
    for _ in range(max_attempts):
        request = {
            "messages": messages,
            "session_id": SESSION_ID,
            "agent_id": AGENT_ID,
            "exclusions": exclusions,
        }
        selection = router.route_sync(request)
        model_id = selection.selected_model_id
        print("route → {0} ({1})".format(model_id, selection.reasoning))

        try:
            reply = call_model(model_id, messages)
        except ModelUnavailable:
            print("{0} 不可用，report Unavailable 后重试".format(model_id))
            router.report_sync(
                Feedback.ok(
                    selection,
                    latency_ms=1,
                    session_id=SESSION_ID,
                    agent_id=AGENT_ID,
                    outcome=Outcome.UNAVAILABLE,
                )
            )
            exclusions.append(model_id)
            continue

        router.report_sync(
            Feedback.ok(
                selection,
                latency_ms=1,
                session_id=SESSION_ID,
                agent_id=AGENT_ID,
                outcome=Outcome.OK,
            )
        )
        return reply

    raise RuntimeError("no available target after retries")


def main():
    router = Router.from_config(PROFILE)
    print("algorithm: {0}".format(router.algorithm_name()))

    reply = invoke_with_routing(router, [{"role": "user", "content": "hello"}])
    print("reply: {0}".format(reply))

    # 第二次调用：strong-cloud 已在 state 中建立亲和，直接命中。
    reply = invoke_with_routing(router, [{"role": "user", "content": "hello again"}])
    print("reply: {0}".format(reply))


if __name__ == "__main__":
    main()

from __future__ import annotations

"""远程状态配置。骨架占位，真正客户端由 PyO3 扩展暴露。"""


class StateClientConfig:
    def __init__(self, endpoint, timeout_ms=5):
        self.endpoint = endpoint
        self.timeout_ms = timeout_ms

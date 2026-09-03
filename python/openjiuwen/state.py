from __future__ import annotations

"""远程状态客户端配置。真正的客户端是 PyO3 `StateClient`。"""


class StateClientConfig:
    def __init__(self, endpoint, timeout_ms=5):
        self.endpoint = endpoint
        self.timeout_ms = timeout_ms

    def client(self):
        from openjiuwen import StateClient

        return StateClient(self.endpoint, timeout_ms=self.timeout_ms)

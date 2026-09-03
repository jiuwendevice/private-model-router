"""PyO3 扩展 `_openjiuwen` 的类型桩。Pylance / Pyright 靠它跳转，运行时仍加载 `.pyd`。"""

from typing import Any, List, Optional, Union

class Message:
    role: str
    content: str
    def __init__(self, role: str, content: str) -> None: ...

class RequestMetadata:
    session_id: Optional[str]
    agent_id: Optional[str]
    def __init__(self, session_id: Optional[str] = ..., agent_id: Optional[str] = ...) -> None: ...
    def routing_key(self) -> RoutingKey: ...

class RoutingKey:
    session_id: str
    agent_id: str
    def __init__(self, session_id: Optional[str] = ..., agent_id: Optional[str] = ...) -> None: ...

class RouteHint:
    cache_affinity: Optional[str]
    def __init__(self, cache_affinity: Optional[str] = ...) -> None: ...

class RouteRequest:
    messages: List[Message]
    metadata: RequestMetadata
    exclusions: List[str]
    def __init__(
        self,
        messages: Optional[List[Message]] = ...,
        metadata: Optional[RequestMetadata] = ...,
        exclusions: Optional[List[str]] = ...,
    ) -> None: ...
    def routing_key(self) -> RoutingKey: ...

class FeedbackStats:
    sample_count: int

class StateView:
    affinity: Optional[str]
    exclusions: List[str]
    stats: FeedbackStats

class RouteContext:
    targets: List[str]
    view: StateView
    seed: int

class ModelSelection:
    selected_model_id: str
    reasoning: str
    is_answer_call: bool
    def __init__(
        self,
        selected_model_id: str,
        reasoning: str,
        is_answer_call: bool = ...,
    ) -> None: ...
    @property
    def target(self) -> str: ...

Decision = ModelSelection

class Feedback:
    key: RoutingKey
    selected_model_id: str
    outcome: str
    latency_ms: int
    cache_valid: Optional[bool]
    def __init__(
        self,
        key: RoutingKey,
        selected_model_id: str,
        outcome: str = ...,
        latency_ms: int = ...,
        cache_valid: Optional[bool] = ...,
    ) -> None: ...
    @classmethod
    def ok(
        cls,
        decision: Any,
        latency_ms: int,
        *,
        key: Any = ...,
        session_id: Optional[str] = ...,
        agent_id: Optional[str] = ...,
        selected_model_id: Optional[str] = ...,
        cache_valid: Optional[bool] = ...,
        outcome: str = ...,
    ) -> Feedback: ...

class StateClient:
    def __init__(self, endpoint: str, timeout_ms: int = ...) -> None: ...
    @property
    def endpoint(self) -> str: ...
    @property
    def timeout_ms(self) -> int: ...

class Router:
    @staticmethod
    def from_config(config: Union[str, dict], *, state: Any = ...) -> Router: ...
    @staticmethod
    def from_toml(text: str) -> Router: ...
    def route(self, request: Any, hint: Any = ...) -> ModelSelection: ...
    def report(self, feedback: Any) -> None: ...
    def algorithm_name(self) -> str: ...
    def with_kv_coordinator(self, cb: Any) -> None: ...
    def replace_algorithm(self, obj: Any) -> None: ...
    def replace_state(self, state: Any) -> None: ...

def register_algorithm(obj: Any) -> str: ...

OK: str
OVERFLOW: str
UNAVAILABLE: str
REJECTED: str

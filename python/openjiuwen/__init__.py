"""openjiuwen 的 Python 公开接口。

Python 包根目录只放置宿主接口和 PyO3 集成契约；团队编写的
Python 算法示例位于 ``python/test``，不作为内置算法发布。
"""

from __future__ import annotations

from importlib import import_module
from typing import Any

__all__ = [
    "Decision",
    "PyAlgorithm",
    "RouteContext",
    "RouteRequest",
    "Router",
    "check_purity",
    "register_algorithm",
    "unregister_algorithm",
]

_NATIVE_EXPORTS = {"Router", "Decision", "RouteRequest", "RouteContext"}


class PyAlgorithm:
    """Python 路由算法的最小契约。

    子类实现 ``decide(request, ctx)``；该方法必须是无 I/O、无可变状态的
    纯函数。调用 ``register()`` 后，PyO3 会将对象包装成 Rust
    ``Algorithm`` trait 对象。
    """

    name = "unnamed"

    def decide(self, request: Any, ctx: Any) -> Any:
        raise NotImplementedError(
            "{0} must implement decide()".format(type(self).__name__)
        )

    def register(self) -> str:
        return register_algorithm(self)


def check_purity(algorithm: PyAlgorithm, request: Any, ctx: Any, rounds: int = 2) -> Any:
    """用相同输入重复调用算法，检查输出是否稳定。"""

    if rounds < 2:
        raise ValueError("rounds must be at least 2")
    results = [algorithm.decide(request, ctx) for _ in range(rounds)]
    first = results[0]
    for other in results[1:]:
        if other != first:
            raise AssertionError(
                "PyAlgorithm {0} is not pure: {1} != {2}".format(
                    getattr(algorithm, "name", type(algorithm).__name__), first, other
                )
            )
    return first


def register_algorithm(algorithm: PyAlgorithm) -> str:
    """把 Python 算法注册到 PyO3/Rust 算法池。"""

    try:
        from openjiuwen._openjiuwen import register_algorithm as native_register
    except ImportError as exc:  # pragma: no cover - 由未构建扩展的环境触发
        raise ImportError(
            "native extension `_openjiuwen` is not built; run `maturin develop`"
        ) from exc
    return native_register(algorithm)


def unregister_algorithm(name: str) -> bool:
    """从 PyO3/Rust 算法池移除一个 Python 算法。"""

    try:
        from openjiuwen._openjiuwen import unregister_algorithm as native_unregister
    except ImportError as exc:  # pragma: no cover - 由未构建扩展的环境触发
        raise ImportError(
            "native extension `_openjiuwen` is not built; run `maturin develop`"
        ) from exc
    return native_unregister(name)


def __getattr__(name: str) -> Any:
    if name in _NATIVE_EXPORTS:
        try:
            _openjiuwen = import_module("openjiuwen._openjiuwen")
        except ImportError as exc:  # pragma: no cover - 由未构建扩展的环境触发
            raise ImportError(
                "native extension `_openjiuwen` is not built; run `maturin develop`"
            ) from exc
        return getattr(_openjiuwen, name)
    raise AttributeError("module {0!r} has no attribute {1!r}".format(__name__, name))

"""Python 算法契约，对应 Rust `AlgorithmProvider`。

子类必须：设置稳定 ``name``、实现 ``decide(request, ctx)``、能无参构造。
类定义时校验并写入槽位。配置放在类属性上，不要靠构造参数。
必须守纯函数：不 I/O、不调模型。
"""

from __future__ import annotations

from typing import Any, Callable, List, Optional, Type


_register: Optional[Callable[[Any], str]] = None
_pending: List[Type["AlgorithmProvider"]] = []


def bind_register(register: Optional[Callable[[Any], str]]) -> None:
    """由包初始化注入内部登记函数；扩展未构建时为 None。"""
    global _register
    _register = register
    if register is None:
        return
    waiting = list(_pending)
    _pending.clear()
    for cls in waiting:
        _commit(cls)


def _validate(cls: Type["AlgorithmProvider"]) -> Any:
    if getattr(cls, "decide", None) is AlgorithmProvider.decide:
        raise TypeError("{0} must implement decide(request, ctx)".format(cls.__name__))
    name = getattr(cls, "name", "unnamed")
    if not isinstance(name, str) or not name or name == "unnamed":
        raise TypeError("{0} must set a non-empty class attribute name".format(cls.__name__))
    try:
        return cls()
    except TypeError as exc:
        raise TypeError(
            "{0} must be constructible with no arguments; put config on the class".format(
                cls.__name__
            )
        ) from exc


def _commit(cls: Type["AlgorithmProvider"]) -> None:
    instance = _validate(cls)
    if _register is None:
        if cls not in _pending:
            _pending.append(cls)
        return
    _register(instance)


class AlgorithmProvider:
    """算法槽插件契约。与 state 侧 `StateProvider` 对位。"""

    name = "unnamed"

    def __init_subclass__(cls, **kwargs):
        super(AlgorithmProvider, cls).__init_subclass__(**kwargs)
        _commit(cls)

    def decide(self, request, ctx):
        raise NotImplementedError("{0} must implement decide()".format(type(self).__name__))


# 旧名。新代码请用 AlgorithmProvider。
Algorithm = AlgorithmProvider


def check_purity(algo, request, ctx, rounds=2):
    """同输入双调用比对输出，辅助验收纯函数纪律。"""
    results = [algo.decide(request, ctx) for _ in range(rounds)]
    first = results[0]
    for other in results[1:]:
        if other != first:
            raise AssertionError("AlgorithmProvider {0} is not pure: {1} != {2}".format(
                getattr(algo, "name", type(algo).__name__), first, other
            ))
    return first

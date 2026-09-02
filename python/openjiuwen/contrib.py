from __future__ import annotations

"""PyAlgorithm SDK 基类。外部团队供稿用，不是内置实现的家。"""


class PyAlgorithm:
    """Python 算法内核的兼容通道。

    子类实现 `decide(request, ctx) -> Decision`，经 PyO3 反向包装成 Rust
    `Algorithm` trait 对象后注册进同一注册表。必须守纯函数契约。
    """

    name = "unnamed"

    def decide(self, request, ctx):
        raise NotImplementedError("{0} must implement decide()".format(type(self).__name__))


def check_purity(algo, request, ctx, rounds=2):
    """同输入双调用比对输出，辅助验收纯函数纪律。"""
    results = [algo.decide(request, ctx) for _ in range(rounds)]
    first = results[0]
    for other in results[1:]:
        if other != first:
            raise AssertionError("PyAlgorithm {0} is not pure: {1} != {2}".format(
                getattr(algo, "name", type(algo).__name__), first, other
            ))
    return first

from __future__ import annotations

"""Python 算法契约，对应 Rust `AlgorithmProvider`。

子类实现 `decide(request, ctx) -> dict | ModelSelection`，经 PyO3 反向包装成
Rust trait 对象后注册进同一注册表。必须守纯函数：不 I/O、不调模型。
"""


class AlgorithmProvider:
    """算法槽插件契约。与 state 侧 `StateProvider` 对位。"""

    name = "unnamed"

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

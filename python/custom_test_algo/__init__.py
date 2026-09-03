"""安装 openjiuwen wheel 之后，宿主侧自己写的算法示例。

不在 ``openjiuwen`` 包内，``discover`` 不会扫描到这里；必须显式
``register_algorithm``，再把 profile 的 ``algorithm`` 写成同一稳定名。
"""

from .prefer_first import PreferFirstAlgorithm

__all__ = ["PreferFirstAlgorithm"]

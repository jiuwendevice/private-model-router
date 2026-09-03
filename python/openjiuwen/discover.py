"""扫描 `openjiuwen` 下各子包中的 ``AlgorithmProvider``，与具体算法实现解耦。

算法团队把实现放在并列目录（``test_algo``、``test_algo2`` …）。本模块按包扫描，
不按名导入任何一个团队包。``import openjiuwen`` 时用无参构造写入 Rust 槽。
"""

from __future__ import annotations

import importlib
import inspect
import pkgutil
import sys
from types import ModuleType
from typing import Any, Callable, List, Optional, Set, Type

from .algorithm_provider import AlgorithmProvider

__all__ = ["bundled_algorithms", "install"]


def _host_package() -> ModuleType:
    host_name = __name__.rpartition(".")[0]
    if not host_name:
        raise RuntimeError("discover.py must be imported as openjiuwen.discover")
    return sys.modules[host_name]


def bundled_algorithms() -> List[Type[AlgorithmProvider]]:
    """收集所有子包里带稳定 ``name`` 的算法类。"""
    pkg = _host_package()
    found: List[Type[AlgorithmProvider]] = []
    seen: Set[str] = set()
    prefix = pkg.__name__ + "."
    for _finder, name, ispkg in pkgutil.iter_modules(pkg.__path__, prefix):
        short = name.rsplit(".", 1)[-1]
        if not ispkg or short.startswith("_"):
            continue
        try:
            sub = importlib.import_module(name)
        except ImportError:
            continue
        _collect(sub, found, seen)
        sub_path = getattr(sub, "__path__", None)
        if sub_path is None:
            continue
        for _sf, subname, _ispkg in pkgutil.walk_packages(sub_path, name + "."):
            try:
                _collect(importlib.import_module(subname), found, seen)
            except ImportError:
                continue
    return found


def install(register: Optional[Callable[[Any], str]] = None) -> List[str]:
    """把发现的算法以无参实例写入 Rust 槽。"""
    if register is None:
        from openjiuwen import register_algorithm as register
    if register is None:
        return []
    return [register(cls()) for cls in bundled_algorithms()]


def _collect(
    module: ModuleType,
    found: List[Type[AlgorithmProvider]],
    seen: Set[str],
) -> None:
    for attr in vars(module).values():
        if not inspect.isclass(attr):
            continue
        if attr is AlgorithmProvider or not issubclass(attr, AlgorithmProvider):
            continue
        name = getattr(attr, "name", "unnamed")
        if not name or name == "unnamed" or name in seen:
            continue
        seen.add(name)
        found.append(attr)

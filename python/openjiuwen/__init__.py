"""openjiuwen — 云侧 Python 门面。

Router / Decision 由 PyO3 扩展 `_openjiuwen` 重导出；扩展未构建时包仍可导入，
内置 Python 算法与 contrib SDK 可独立使用。
"""

from __future__ import annotations

from . import algorithm, contrib

__all__ = ["algorithm", "contrib", "Router", "Decision", "Feedback", "StateClient"]


def __getattr__(name: str):
    if name in ("Router", "Decision", "Feedback", "StateClient"):
        try:
            from openjiuwen._openjiuwen import Decision, Router  # type: ignore
        except ImportError as exc:  # pragma: no cover
            raise ImportError(
                "native extension `_openjiuwen` is not built; run `maturin develop`"
            ) from exc
        mapping = {
            "Router": Router,
            "Decision": Decision,
            "Feedback": None,
            "StateClient": None,
        }
        value = mapping[name]
        if value is None:
            raise AttributeError(
                "{0} is not wired in the skeleton PyO3 module yet".format(name)
            )
        return value
    raise AttributeError("module {0!r} has no attribute {1!r}".format(__name__, name))

from __future__ import annotations

import sys
from pathlib import Path

# 已安装 wheel 时用 site-packages 里的 `_openjiuwen`。
# 未安装时把源码 `python/` 放进 path，纯 Python 算法测试仍能跑。
try:
    import openjiuwen._openjiuwen  # noqa: F401
except ImportError:
    src = Path(__file__).resolve().parents[1] / "python"
    sys.path.insert(0, str(src))

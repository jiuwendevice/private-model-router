from __future__ import annotations

import sys
from pathlib import Path

# 始终优先仓库 `python/`，避免 site-packages 里的旧 wheel 挡住本地改动。
# 原生扩展需 `maturin develop`，把 `_openjiuwen` 编进该目录。
src = Path(__file__).resolve().parents[1] / "python"
sys.path.insert(0, str(src))

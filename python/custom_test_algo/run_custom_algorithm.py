"""演示：安装 wheel 后导入自己的 AlgorithmProvider 子类，再走 Rust runtime。

    python python/custom_test_algo/run_custom_algorithm.py
"""

from __future__ import annotations

import tempfile
from pathlib import Path

from custom_test_algo import PreferFirstAlgorithm
from openjiuwen import Router


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        config = Path(directory) / "custom-algorithm.toml"
        config.write_text(
            """
algorithm = "custom_prefer_first"
[state]
backend = "memory"
[targets]
models = ["fast-expensive", "slow-cheap"]
""".strip(),
            encoding="utf-8",
        )
        router = Router.from_config(str(config))
        decision = router.route_sync({})
        print(
            "Rust runtime selected {0}: {1}".format(
                decision.selected_model_id, decision.reasoning
            )
        )
        assert router.algorithm_name() == PreferFirstAlgorithm.name


if __name__ == "__main__":
    main()

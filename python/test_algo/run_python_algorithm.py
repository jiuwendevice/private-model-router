"""运行一次 Python 算法 -> PyO3 -> Rust runtime -> Python 的完整链路。"""

from __future__ import annotations

import tempfile
from pathlib import Path

from cost_aware_algorithm import CostAwareAlgorithm
from openjiuwen import Router, register_algorithm


def main() -> None:
    algorithm = CostAwareAlgorithm(
        {"fast-expensive": 10.0, "slow-cheap": 1.0}
    )
    register_algorithm(algorithm)

    with tempfile.TemporaryDirectory() as directory:
        config = Path(directory) / "python-algorithm.toml"
        config.write_text(
            """
algorithm = "python_cost_aware"
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


if __name__ == "__main__":
    main()

# Python 目录

`python/` 明确分为两部分：

```text
python/
├── openjiuwen/                 # 正式包根：项目面向 Python 的公开接口
│   └── __init__.py            # Router / ModelSelection / 注册入口
└── test/                       # 不发布：团队 Python 算法与回接 Rust 的示例
    ├── cost_aware_algorithm.py
    ├── run_python_algorithm.py
    └── test_python_algorithm.py
```

`openjiuwen` 是 maturin 混合工程的 Python 包根。PyO3 生成的
`openjiuwen._openjiuwen` 负责进入 Rust runtime，也负责把已注册的
Python 算法回调为 Rust `AlgorithmProvider` trait。`test/` 仅用于接入示例和验证，
不是生产内置算法包。

完整示例见 [`test/README.md`](test/README.md)。

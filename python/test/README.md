# Python 算法回接 Rust 示例

[`cost_aware_algorithm.py`](cost_aware_algorithm.py) 演示团队如何用 Python 实现
`decide(request, ctx)`。集成链路是：

```text
CostAwareAlgorithm.register()
  -> PyO3 register_algorithm(obj)
  -> PyAlgorithmAdapter { Py<PyAny> }
  -> Rust Box<dyn Algorithm>
  -> runtime::Router::route()
  -> adapter 获取 GIL 并回调 obj.decide(request, ctx)
  -> Python decision 转换为 Rust Decision
```

运行时先构建扩展，再执行示例测试：

```bash
maturin develop
python python/test/run_python_algorithm.py
pytest python/test/test_python_algorithm.py -v
```

算法名必须与 TOML 中的 `algorithm` 一致。Python 算法只能做同步纯计算：
不发网络请求、不读写文件、不修改跨请求状态。

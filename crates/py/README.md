# openjiuwen（PyO3 / L5）

## 简介

`crates/py` 是 openjiuwen-router 的 **L5 宿主集成层**：把 L4 `runtime::Router` 编成 Python 原生扩展 `_openjiuwen`。本 crate **不含路由逻辑**——只做类型转换与同步调用。云侧 async 与算法 SDK 在 `python/openjiuwen/`，Python 测试算法在 `python/test_algo/`。

Python 宿主只看北向门面：`from_config` 装配，`route` 取 `ModelSelection`，自己调模型，再 `report`。决策与执行仍然分离；流量不经过本层。

本 crate 不在 workspace `default-members` 中。日常 `cargo test` 不编 PyO3；要给 Python 用时走 `maturin`。

## 为什么要单独一层 PyO3

- **内核仍是 Rust**：端侧 crate 直连、云侧经本层，同一套 `Router` / 协议类型。
- **跨边界只传值**：北向载荷是 `RouteRequest`、`ModelSelection`、`Feedback`；`RouteContext` / `StateView` / `Decision` 留在 Rust（`Decision` 投影为 `ModelSelection` 再出界）。
- **语言差异被门面吸收**：Python 侧 `await router.route` 只是包一层；内核 `route` 仍是同步纯函数求值。
- **算法可反向占用槽位**：`register_algorithm` 把 Python `AlgorithmProvider` 包装成 Rust trait，与 Rust 算法同一注册表、同一决策循环。

## 仓库结构

```text
crates/py/                          # 本 crate：编出 _openjiuwen
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs            # pymodule；PyRouter / StateClient / Kv 回调
    ├── types.rs          # 协议类型的 pyclass
    ├── convert.rs        # dict / pyclass → 协议结构
    ├── adapter.rs        # Python 算法 → AlgorithmProvider trait
    └── error.rs          # RouterError → Python 异常

python/openjiuwen/                  # 用户面包（maturin python-source）
├── __init__.py           # 北向重导出 + async Router 门面
├── _openjiuwen.pyi       # 扩展类型桩（跳转用）
├── py.typed              # PEP 561 typed 包标记
└── algorithm_provider.py # 公共契约 AlgorithmProvider

python/test_algo/                   # CostAwareAlgorithm：register_algorithm 回接 Rust
tests/react_agent.py                # 最小 Python ReAct 宿主示例
```

`pyproject.toml` 把两边打进同一个 wheel：`module-name = "openjiuwen._openjiuwen"`。

## 快速开始

### 环境要求

Rust `stable`，Python `>= 3.8`，以及 `maturin`。Windows GNU 链接见仓库根目录 `.cargo/config.toml`。

### 编译扩展

在已激活的虚拟环境中、仓库根目录：

```bash
maturin develop
# 或 maturin build --out target/py-wheels && pip install target/py-wheels/*.whl
```

只检查 Rust 侧能否通过 PyO3 编译（不安装 wheel）：

```bash
cargo check -p openjiuwen
```

未构建扩展时，`openjiuwen.AlgorithmProvider` 仍可导入；`Router.from_config` 会提示先 `maturin develop`。

## 北向接口（Python 宿主）

`import openjiuwen` 之后，宿主只用这些名字。对应内部是 `runtime::Router` 与 `openjiuwen-protocol`。

### 装配与决策

| Python | 对应内部 | 说明 |
|--------|----------|------|
| `Router.from_config(path \| dict, *, state=None)` | `Router::from_config` / `from_profile` | dict 便于配置中心注入；`state=` 可传入 `StateClient` 覆盖 profile |
| `Router.from_toml(text)` | `Router::from_toml` | 测试或下发文本 |
| `await router.route(request, hint=None)` | `Router::route` | 云侧 async 门面；内核同步。返回 `ModelSelection` |
| `router.route_sync(request, hint=None)` | 同上 | 不要 event loop 时用 |
| `await router.report(feedback)` | `Router::report` | fire-and-forget；Python 侧 async 立刻返回 |
| `router.report_sync(feedback)` | 同上 | 同步转发 state |
| `router.algorithm_name()` | `Router::algorithm_name` | 当前算法槽稳定名 |
| `router.with_kv_coordinator(cb)` | `KvCacheCoordinator::on_switch` | 保存 Python 回调；`route` 尚未触发切换 |
| `router.replace_algorithm(obj)` | `Router::replace_algorithm` | 热替换算法槽（通常是 `AlgorithmProvider` 子类） |
| `router.replace_state(state)` | `Router::replace_state` | 热替换 state 槽，目前接受 `StateClient` |
| `StateClient(endpoint, timeout_ms=5)` | `openjiuwen_state::RemoteState` | 显式远程客户端；注入 `from_config(..., state=...)` |
| `register_algorithm(obj)` | `PyAlgorithmAdapter` | 按 `obj.name` 写入进程内注册表；`from_config` 优先于 Rust 内置 |

`request` 可以是 `RouteRequest` 或 dict（`messages` / `metadata` 或顶层 `session_id`+`agent_id` / `exclusions`）。`hint` 可以是 `RouteHint`、`str`（当作 `cache_affinity`）、dict 或 `None`。

`route` 的返回值是 **`ModelSelection`**（`Decision` 是同一类型的别名）。字段：`selected_model_id` / `reasoning` / `is_answer_call`；`target` 是 `selected_model_id` 的别名。

### 协议类型（跨边界载荷）

蓝图约定：PyO3 边界上编解码的是这三类。其余为 Rust 内部结构，仅在反向绑定回调 Python 算法时再投影出来。

| Python 类型 | 对应 protocol | 北向用途 |
|-------------|---------------|----------|
| `RouteRequest` | `RouteRequest` | 入参：`messages` + `metadata` + `exclusions` |
| `Message` | `Message` | `role` / `content`；本层不解释角色语义 |
| `RequestMetadata` | `RequestMetadata` | `session_id` / `agent_id` → `routing_key()` |
| `RoutingKey` | `RoutingKey` | 与 `Feedback.key` 必须同一把，排除才对得上 |
| `RouteHint` | `RouteHint` | 每请求 hint，目前 `cache_affinity` |
| `ModelSelection`（`Decision`） | `ModelSelection` | `route` 出参；宿主拿 `selected_model_id` 去调模型 |
| `Feedback` | `Feedback` | 回报：`key` + `selected_model_id` + `outcome` + `latency_ms` |
| `Outcome` | `Outcome` | `OK` / `OVERFLOW` / `UNAVAILABLE` / `REJECTED`（字符串常量） |

`Feedback.ok(decision, latency_ms, *, key=..., session_id=..., agent_id=..., outcome=...)` 从决策上取模型名。`OVERFLOW` / `UNAVAILABLE` 写入 state 排除表；`REJECTED` 不排除；`OK` 更新亲和。

### 反向绑定会看到、宿主通常不构造

| Python 类型 | 说明 |
|-------------|------|
| `RouteContext` | `targets: list[str]`、`view`、`seed`；传给 `AlgorithmProvider.decide` |
| `StateView` | `affinity` / `exclusions` / `stats`；可为空，算法必须能降级 |
| `openjiuwen.AlgorithmProvider` | 供稿基类：实现 `name` + `decide(request, ctx)` |
| `openjiuwen.check_purity` | 同输入双调用，辅助验收纯函数 |
| `test_algo.CostAwareAlgorithm` | Python 回接示例：按配置成本选目标 |

Python 算法必须守纯函数：不 I/O、不调模型、随机性只用 `ctx.seed`。端侧没有解释器，这些实现只随 wheel 走云侧。

### 异常

| Python 异常 | 来源 `RouterError` |
|-------------|-------------------|
| `ValueError`（`config: ...`） | `Config`：未知算法、未知 backend、缺 `endpoint`、TOML 失败 |
| `RuntimeError`（`algorithm:` / `state:`） | `Algorithm` / `State` |
| `LookupError` | `NoTarget`：排除后目录为空 |

没有单独的 `openjiuwen.RouterError` 类型。

与 Rust 北向契约 [`RouterProvider`](../runtime/README.md) 的对应：

| Rust `RouterProvider` | Python |
|-----------------------|--------|
| `route(request, hint) -> ModelSelection` | `await router.route` / `route_sync` |
| `report(feedback)` | `await router.report` / `report_sync` |
| `algorithm_name()` | `router.algorithm_name()` |

装配（`from_config`）与南向替换不在该 trait 上，Python 仍走 `Router` 类方法。

### 北向调用链

```text
Python 宿主
    Router.from_config(path | dict)
    await router.route(RouteRequest | dict, hint?)
        → _openjiuwen（类型转换）
        → runtime snapshot(RoutingKey) → decide
        → ModelSelection
    宿主自己调用 selected_model_id
    await router.report(Feedback)
        → state 写回；下一轮 snapshot 才看见排除 / 亲和
```

## 样例 1：装配、路由、回报

```python
from openjiuwen import Feedback, Outcome, Router

router = Router.from_config("config/cloud.toml")
# 或从配置中心：
# router = Router.from_config({
#     "algorithm": "passthrough",
#     "state": {"backend": "memory"},
#     "targets": {"models": ["fast-local", "strong-cloud"]},
# })

decision = await router.route({
    "messages": [{"role": "user", "content": "What is 21 * 2?"}],
    "session_id": "sess-1",
    "agent_id": "host-app",
})
# 宿主自己调用 decision.selected_model_id（或 decision.target）

await router.report(Feedback.ok(
    decision,
    latency_ms=40,
    session_id="sess-1",
    agent_id="host-app",
    outcome=Outcome.OK,
))
```

同步路径把 `await` 换成 `route_sync` / `report_sync`。

## 样例 2：失败排除与 Python 算法

```python
from openjiuwen import Feedback, Outcome, RouteRequest, RequestMetadata, Router, register_algorithm
from test_algo import CostAwareAlgorithm

register_algorithm(CostAwareAlgorithm({"fast-local": 1.0, "strong-cloud": 10.0}))

router = Router.from_toml("""
algorithm = "python_cost_aware"
[state]
backend = "memory"
[targets]
models = ["fast-local", "strong-cloud"]
""")

req = RouteRequest(
    messages=[],
    metadata=RequestMetadata(session_id="s1", agent_id="a1"),
)
first = router.route_sync(req)
router.report_sync(Feedback.ok(
    first, latency_ms=1, key=req.routing_key(), outcome=Outcome.UNAVAILABLE,
))
second = router.route_sync(req)  # 同一把 RoutingKey，下一轮不再选 fast-local
```

端到端 ReAct 宿主（与 `tests/react_agent.rs` 同一剧本）：

```bash
python tests/react_agent.py
```

## 主要模块

### `lib.rs`

`#[pymodule] _openjiuwen`：注册 `Router`、`StateClient`、协议 pyclass、`register_algorithm`。`Decision` 是 `ModelSelection` 的模块别名。

### `types.rs` / `convert.rs`

北向类型的 pyclass，以及 dict → 协议结构。宿主可以只传 dict，不必手写每个 pyclass。

### `adapter.rs`

进程内 `name → Py<PyAny>`。`from_config` 先查这张表，没有再走 `registry::create_algorithm`。

### `python/openjiuwen/__init__.py`

用户面包：`Router` 在这里加 async；协议类型从 `_openjiuwen` 重导出。类型检查走 `_openjiuwen.pyi`。

## 测试与检查

```bash
cargo check -p openjiuwen
pytest tests/test_package.py              # 不需要扩展（算法 SDK）
pytest tests/test_native_router.py        # 需要已安装的 _openjiuwen
pytest tests/test_react_agent.py
python tests/react_agent.py
```

协议类型本身的词汇表与北向契约见 [`../protocol/README.md`](../protocol/README.md)。Rust 宿主不经本层，见 [`../runtime/README.md`](../runtime/README.md)。

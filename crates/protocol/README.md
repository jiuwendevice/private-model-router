# openjiuwen-protocol

## 简介

`openjiuwen-protocol` 是 openjiuwen-router 的 **L1 协议层**：全部跨模块类型的词汇表。本 crate **只有数据、没有插件行为**，且 **零依赖**——不引用 state / algorithms / runtime，也不引入 serde、tokio 等外部库。

其他 crate 只经本层对话：宿主构造 `RouteRequest`，算法返回 `Decision`，state 产出 `StateView`、吸收 `Feedback`。任何一层可独立替换，只要仍说这套类型。

`RouteContext`（`targets` / `view` / `seed`）在 `openjiuwen-algorithms` 中定义，不属于本 crate。

## 为什么协议层必须零依赖

- **替换边界硬**：算法、state、runtime 互不直接依赖，只共享本层类型。
- **端云同构**：同一套结构在进程内引用传递，也可作为跨语言 / RPC 的载荷规格（`ModelSelection`）。
- **无隐藏 I/O**：协议层不发网络、不读时钟；`latency_ms` 用整数毫秒，避免依赖时间库。
- **可测试**：构造几个结构体就能驱动 `decide` / `snapshot` / `report`，不必拉完整运行时。

## 仓库结构

```text
crates/protocol/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs            # 重导出全部公开类型
    ├── request.rs        # RouteRequest / RoutingKey / TargetSet / RouteHint
    ├── decision.rs       # Decision
    ├── selection.rs      # ModelSelection（Decision 的跨边界投影）
    ├── state_view.rs     # StateView / FeedbackStats
    ├── feedback.rs       # Feedback / Outcome
    └── error.rs          # RouterError
```

## 快速开始

### 环境要求

与仓库根目录相同：Rust `stable`。本 crate 无额外系统依赖。

### 编译

在仓库根目录：

```bash
cargo build -p openjiuwen-protocol
cargo test -p openjiuwen-protocol
```

下游 crate 在 `Cargo.toml` 中声明：

```toml
openjiuwen-protocol.workspace = true
```

`openjiuwen-runtime` 已把宿主常用类型再导出一遍，Rust 宿主也可以只依赖 runtime。

## 样例 1：构造一次路由入参

`session_id` / `agent_id` 构成 `RoutingKey`，state 按这个键做 snapshot。`exclusions` 由宿主重试逻辑填写。

```rust
use openjiuwen_protocol::{Message, RequestMetadata, RouteRequest};

let req = RouteRequest {
    messages: vec![Message {
        role: "user".into(),
        content: "What is 21 * 2?".into(),
    }],
    metadata: RequestMetadata {
        session_id: Some("sess-1".into()),
        agent_id: Some("react-agent".into()),
    },
    exclusions: vec![],
};

let key = req.routing_key();
```

`RouteHint.cache_affinity` 是每请求的 KV cache 驻留模型 hint，作为 `route` 的第二个入参，不放进 `RouteRequest`。

## 样例 2：决策、投影与反馈

算法返回 `Decision`；跨 PyO3 / rail 覆盖槽时投影为 `ModelSelection`。宿主调完模型后构造 `Feedback`。

```rust
use openjiuwen_protocol::{Decision, Feedback, ModelSelection, Outcome, RoutingKey};

let decision = Decision::answer("strong-cloud", "passthrough: first available target");
let selection = ModelSelection::from(&decision);

let feedback = Feedback {
    key: RoutingKey {
        session_id: "sess-1".into(),
        agent_id: "react-agent".into(),
    },
    selected_model_id: decision.selected_model_id.clone(),
    outcome: Outcome::Ok, // Overflow / Unavailable 会驱动排除
    latency_ms: 40,
    cache_valid: None,
};
let _ = selection;
let _ = feedback;
```

`Decision` 与 `ModelSelection` 字段相同（`selected_model_id` / `reasoning` / `is_answer_call`），差别只在「是否可执行」：前者是 runtime 内部返回值，后者是跨边界规格。runtime 北向契约 `RouterProvider::route` 返回 `ModelSelection`。

## 主要模块

### 请求（`request.rs`）

| 类型 | 作用 |
|------|------|
| `RouteRequest` | 路由入参：`messages` + `metadata` + `exclusions` |
| `Message` | 单条对话；协议层只搬运文本，不解释角色 |
| `RequestMetadata` | `session_id` / `agent_id` → `RoutingKey` |
| `RoutingKey` | state 快照的键空间 |
| `TargetSet` | 可选模型语义名；`without` 按原顺序剔除排除项 |
| `RouteHint` | 宿主每请求输入，目前含 `cache_affinity` |

### 决策与投影（`decision.rs` / `selection.rs`）

| 类型 | 作用 |
|------|------|
| `Decision` | 算法出参；`reasoning` 必填；`Decision::answer` 默认应答调用 |
| `ModelSelection` | `From<Decision>`，供宿主 / 下一级插件消费 |

### 状态快照（`state_view.rs`）

| 类型 | 作用 |
|------|------|
| `StateView` | 路由前一次性 hint：`affinity` / `exclusions` / `stats`；空视图合法 |
| `FeedbackStats` | 累计样本数等统计，字段随算法需求扩展 |

算法必须能在 `StateView::empty()` 下降级，不能把空视图当成错误。

### 反馈（`feedback.rs`）

| `Outcome` | 含义 |
|-----------|------|
| `Ok` | 调用成功，可更新亲和 |
| `Overflow` | 上下文溢出，写入排除 hint |
| `Unavailable` | 模型不可用，写入排除 hint |
| `Rejected` | 语义失败，**不**驱动排除 |

`Feedback.latency_ms` 为 `u64` 毫秒。`cache_valid` 可选，供状态层学习 KV cache 重建成本。

### 错误（`error.rs`）

| `RouterError` | 何时出现 |
|---------------|----------|
| `Config` | 装配期：未知算法名、未知 state backend、TOML 读失败 |
| `Algorithm` | 决策期：算法内部失败（含 Python 异常映射） |
| `State` | 状态层失败（远程实现超时应返回空视图，而不是本变体） |
| `NoTarget` | 剔除 exclusions 后目录为空 |

装配错误在 `from_config` 时暴露；`NoTarget` 出现在 `decide` / `route`。

### 类型在链路中的位置

```text
宿主  RouteRequest + RouteHint
        ↓
runtime snapshot(RoutingKey) → StateView
        ↓
algorithm decide → Decision
        ↓ 可选投影
      ModelSelection（跨边界）
        ↓ 宿主自己调模型
宿主  Feedback{key, selected_model_id, outcome, latency_ms}
        ↓
state 写回，下一轮 snapshot 才能看见
```

## 测试与检查

```bash
cargo fmt -p openjiuwen-protocol -- --check
cargo test -p openjiuwen-protocol
```

这些类型在 ReAct 宿主里的用法见仓库根目录：

```bash
cargo test -p openjiuwen-runtime --test react_agent -- --nocapture
```

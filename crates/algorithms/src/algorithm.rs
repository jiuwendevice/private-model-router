//! 路由算法契约。纯函数：不做 I/O，不持有可变状态。
//!
//! 同样的 `(request, ctx)` 必须返回同样的 [`Decision`]。

use openjiuwen_protocol::{Decision, RouteRequest, RouterError, StateView, TargetSet};

/// 注入给算法的只读上下文。状态是 hint，`view` 可为空。
#[derive(Clone, Debug)]
pub struct RouteContext {
    /// 本次可选目标（已剔除 exclusions）。
    pub targets: TargetSet,
    /// 状态快照（可为空，算法必须能降级）。
    pub view: StateView,
    /// 随机性显式注入，保证可重放。
    pub seed: u64,
}

/// 算法团队的唯一接入点。
pub trait Algorithm: Send + Sync {
    /// 稳定的低基数名称，用于注册表与遥测。
    fn name(&self) -> &str;

    /// 单步决策。读请求与状态快照，直接返回选中的目标。
    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError>;
}

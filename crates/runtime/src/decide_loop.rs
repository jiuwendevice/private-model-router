//! 决策循环：snapshot → 装配 RouteContext → decide → Decision。

use openjiuwen_algorithms::{AlgorithmProvider, RouteContext};
use openjiuwen_protocol::{Decision, RouteHint, RouteRequest, RouterError, TargetSet};
use openjiuwen_state::StateProvider;

/// 驱动一次纯函数决策。重试时由宿主经 `req.exclusions` 排除已败目标。
pub fn run(
    algorithm: &dyn AlgorithmProvider,    // 算法实例       
    state: &dyn StateProvider,    // 状态实例
    req: &RouteRequest,    // 路由请求
    hint: &RouteHint,    // 路由提示
    catalog: &TargetSet,    // 目标集合
    seed: u64,    // 随机种子
) -> Result<Decision, RouterError> {    // 返回的是 Result<Decision, RouterError> 类型。
    let _ = hint;
    let view = state.snapshot(&req.routing_key());
    let mut exclusions = req.exclusions.clone();
    // 排除列表扩展。
    exclusions.extend(view.exclusions.iter().cloned());
    // 目标集合过滤。
    let targets = catalog.without(&exclusions);

    let ctx: RouteContext = RouteContext {
        targets,
        view,
        seed,
    };
    algorithm.decide(req, &ctx)
}

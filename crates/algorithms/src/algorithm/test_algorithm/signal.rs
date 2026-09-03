use openjiuwen_protocol::{Decision, RouteRequest, RouterError};

use crate::algorithm::{AlgorithmProvider, RouteContext};

/// 信号驱动。骨架阶段退化为直通。
pub struct Signal;

impl AlgorithmProvider for Signal {
    fn name(&self) -> &str {
        "signal"
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        let available = ctx.targets.without(&request.exclusions);
        let model = available.first().ok_or(RouterError::NoTarget)?;
        Ok(Decision::answer(model, "signal: stub, falls back to first target"))
    }
}

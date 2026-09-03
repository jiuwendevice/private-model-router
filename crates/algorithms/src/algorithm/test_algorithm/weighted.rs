use openjiuwen_protocol::{Decision, RouteRequest, RouterError};

use crate::algorithm::{AlgorithmProvider, RouteContext};

/// 加权选择。骨架阶段退化为直通，权重表后续注入。
pub struct Weighted;

impl AlgorithmProvider for Weighted {
    fn name(&self) -> &str {
        "weighted"
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        let available = ctx.targets.without(&request.exclusions);
        let model = available.first().ok_or(RouterError::NoTarget)?;
        Ok(Decision::answer(
            model,
            "weighted: stub, falls back to first target",
        ))
    }
}

use openjiuwen_protocol::{Decision, RouteRequest, RouterError};

use crate::algorithm::{AlgorithmProvider, RouteContext};

/// 直通：选目标集中第一个未被排除的模型。
pub struct Passthrough;

impl AlgorithmProvider for Passthrough {
    fn name(&self) -> &str {
        "passthrough"
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        let available = ctx.targets.without(&request.exclusions);
        let model = available.first().ok_or(RouterError::NoTarget)?;
        Ok(Decision::answer(model, "passthrough: first available target"))
    }
}

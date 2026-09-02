use openjiuwen_protocol::{Decision, RouteRequest, RouterError};

use crate::algorithm::{Algorithm, RouteContext};

/// 规则级联。骨架阶段退化为直通。
pub struct RuleCascade;

impl Algorithm for RuleCascade {
    fn name(&self) -> &str {
        "rule_cascade"
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        let available = ctx.targets.without(&request.exclusions);
        let model = available.first().ok_or(RouterError::NoTarget)?;
        Ok(Decision::answer(
            model,
            "rule_cascade: stub, falls back to first target",
        ))
    }
}

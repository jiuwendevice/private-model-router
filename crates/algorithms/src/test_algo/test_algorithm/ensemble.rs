use openjiuwen_protocol::{Decision, RouteRequest, RouterError};

use crate::test_algo::{AlgorithmProvider, RouteContext};

/// 集成/混合。骨架阶段退化为直通。
pub struct Ensemble;

impl AlgorithmProvider for Ensemble {
    fn name(&self) -> &str {
        "ensemble"
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        let available = ctx.targets.without(&request.exclusions);
        let model = available.first().ok_or(RouterError::NoTarget)?;
        Ok(Decision::answer(
            model,
            "ensemble: stub, falls back to first target",
        ))
    }
}

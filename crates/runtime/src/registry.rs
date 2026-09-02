//! 算法池：候选目录 · 单槽选一 · 可热插拔。

use openjiuwen_algorithms::Algorithm;
use openjiuwen_protocol::RouterError;

/// 按名称从内置算法池取出一个实例。未命中则报装配错误。
pub fn create_algorithm(name: &str) -> Result<Box<dyn Algorithm>, RouterError> {
    match name {
        #[cfg(feature = "algo-passthrough")]
        "passthrough" => Ok(Box::new(openjiuwen_algorithms::passthrough::Passthrough)),
        #[cfg(feature = "algo-weighted")]
        "weighted" => Ok(Box::new(openjiuwen_algorithms::weighted::Weighted)),
        #[cfg(feature = "algo-rule_cascade")]
        "rule_cascade" => Ok(Box::new(openjiuwen_algorithms::rule_cascade::RuleCascade)),
        #[cfg(feature = "algo-signal")]
        "signal" => Ok(Box::new(openjiuwen_algorithms::signal::Signal)),
        #[cfg(feature = "algo-ensemble")]
        "ensemble" => Ok(Box::new(openjiuwen_algorithms::ensemble::Ensemble)),
        other => Err(RouterError::Config(format!(
            "unknown or disabled algorithm: {other}"
        ))),
    }
}

//! 宿主回报的一次调用结果。语义失败不在此列。

use crate::RoutingKey;

/// 溢出与不可用驱动排除逻辑；`Ok` 用于更新亲和/延迟统计。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Overflow,
    Unavailable,
    Rejected,
}

/// 反馈入参。`latency_ms` 用整数毫秒，避免协议层依赖时间库。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feedback {
    pub key: RoutingKey,        // 路由键
    pub selected_model_id: String, // 选中目标模型id
    pub outcome: Outcome,          // 结果
    pub latency_ms: u64,           // 延迟毫秒
    pub cache_valid: Option<bool>,
}

impl Feedback {
    /// 创建一个成功反馈。  key: 路由键, selected_model_id: 选中目标模型id, latency_ms: 延迟毫秒
    /// 缓存有效性为 None。
    pub fn ok(key: RoutingKey, selected_model_id: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            key,                                        // 路由键
            selected_model_id: selected_model_id.into(), // 选中目标模型id
            outcome: Outcome::Ok,                        // 结果
            latency_ms,                                  // 延迟毫秒
            cache_valid: None,                           // 缓存有效性
        }
    }
}

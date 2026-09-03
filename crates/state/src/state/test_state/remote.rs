//! gRPC 客户端（超时降级）。骨架阶段不发网络，snapshot 直接返回空视图。

use std::time::Duration;

use openjiuwen_protocol::{Feedback, RoutingKey, StateView};

use crate::state::StateProvider;

/// 云侧 state 槽实现：硬超时 → 空视图，请求不失败。
pub struct RemoteState {
    pub endpoint: String,
    pub timeout: Duration,
}

impl RemoteState {
    /// 创建一个新的远程状态实例。
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout,
        }
    }
}

impl StateProvider for RemoteState {
    /// 获取状态快照。
    fn snapshot(&self, _key: &RoutingKey) -> StateView {
        // TODO: gRPC snapshot；超时返回空视图而非 Err。
        StateView::empty()
    }

    /// 上报反馈。
    fn report(&self, _feedback: Feedback) {
        // TODO: gRPC report；fire-and-forget。
    }
}

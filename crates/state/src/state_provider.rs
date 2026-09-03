//! [`StateProvider`]：状态实现者的唯一接入点。
//!
//! 与算法侧 `AlgorithmProvider` 对位：运行期单槽选一。
//! 状态是 hint：有界、可丢失；算法从不直接调用本 trait。

use openjiuwen_protocol::{Feedback, RoutingKey, StateView};

/// CAS 写回冲突：期望版本与当前 active 不一致。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasConflict {
    pub slot: String,
    pub expected: u64,
    pub actual: u64,
}

/// 跨请求记忆的插件契约。状态是 hint：有界、可丢失。
///
/// 远程实现超时必须返回空 [`StateView`] 而非 `Err`，绝不阻塞请求。
pub trait StateProvider: Send + Sync {
    /// 路由前一次性快照。
    fn snapshot(&self, key: &RoutingKey) -> StateView;

    /// 吸收路由后反馈。异步、尽力而为；本骨架同步写入。
    fn report(&self, feedback: Feedback);

    /// 带版本原子写回（训练任务 → state）。默认实现为 no-op。
    fn publish(&self, slot: &str, artifact: &[u8], ver: u64) -> Result<(), CasConflict> {
        let _ = (slot, artifact, ver);
        Ok(())
    }
}

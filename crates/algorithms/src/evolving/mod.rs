//! 在线自演进契约。纯计算、无 I/O；拉数据 / 调度 / 写回由 runtime TrainingJob 履行。

use std::sync::Arc;

use openjiuwen_protocol::Feedback;

#[cfg(feature = "evolving-mf")]
mod mf;
#[cfg(feature = "evolving-mf")]
pub use mf::MfWeights;

/// 按 watermark 从 state 拉到的增量反馈，由 DataSelector 组装。
#[derive(Clone, Debug, Default)]
pub struct TrainingBatch {
    pub feedbacks: Vec<Feedback>,
}

/// 不可变新参数集快照，可多版本共存。
#[derive(Clone, Debug)]
pub struct Artifact {
    pub kind: String,
    pub payload: Vec<u8>,
}

/// 算法自演进的唯一接入点。
pub trait Evolving: Send + Sync {
    fn name(&self) -> &str;

    /// 单步重算。同输入必同输出；不允许 I/O。
    fn fit(&self, batch: &TrainingBatch) -> Arc<Artifact>;
}

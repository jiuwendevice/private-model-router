//! TrainingJob / DataSelector / PublishPlan。后台调度 + CAS 写回。骨架为类型占位。

use std::sync::Arc;

use openjiuwen_algorithms::evolving::{Artifact, EvolvingProvider, TrainingBatch};

/// 按 watermark 从 state 拉增量反馈。骨架返回空 batch。
pub struct DataSelector {
    pub watermark_key: String,
    pub min_samples: u64,
}

impl DataSelector {
    pub fn select(&self) -> TrainingBatch {
        let _ = self;
        TrainingBatch::default()
    }
}

/// 写回计划：目标槽 + 期望版本。
#[derive(Clone, Debug)]
pub struct PublishPlan {
    pub slot: String,
    pub expected_version: u64,
}

/// 训练任务的执行体：拉数据 → EvolvingProvider.fit → CAS 写回。
pub struct TrainingJob {
    pub name: String,
    pub selector: DataSelector,
    pub publish: PublishPlan,
}

impl TrainingJob {
    /// 骨架：选出空 batch、fit、丢弃工件。真实写回走 StateProvider::publish。
    pub fn run_once(&self, evolving: &dyn EvolvingProvider) -> Arc<Artifact> {
        let batch = self.selector.select();
        evolving.fit(&batch)
    }
}

//! MfWeights：矩阵分解示例。骨架只返回空工件。

use std::sync::Arc;

use crate::{Artifact, EvolvingProvider, TrainingBatch};

pub struct MfWeights;

impl EvolvingProvider for MfWeights {
    fn name(&self) -> &str {
        "mf-weights"
    }

    fn fit(&self, _batch: &TrainingBatch) -> Arc<Artifact> {
        Arc::new(Artifact {
            kind: "MfWeights".into(),
            payload: Vec::new(),
        })
    }
}

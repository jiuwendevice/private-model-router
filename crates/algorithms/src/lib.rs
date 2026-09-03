//! L3 算法层。测试算法及自演进实现统一放在 [`test_algo`]。

pub mod test_algo;

pub use test_algo::evolving::{Artifact, EvolvingProvider, TrainingBatch};
pub use test_algo::{AlgorithmProvider, RouteContext};

// 保留原模块路径，避免下游升级目录布局时同时发生 API 破坏。
pub use test_algo as algorithm;
pub use test_algo::evolving;

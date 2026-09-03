//! L3 算法层。契约在 [`algorithm`] / [`evolving`]；
//! 示意实现在 [`algorithm::test_algorithm`] / [`evolving::test_evolving`]。

pub mod algorithm;
pub mod evolving;

pub use algorithm::{AlgorithmProvider, RouteContext};
pub use evolving::{Artifact, EvolvingProvider, TrainingBatch};

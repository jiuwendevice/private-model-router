//! 在线自演进公共层：契约在 [`evolving_provider`]，示意实现在 [`test_evolving`]。

pub mod evolving_provider;
pub mod test_evolving;

pub use evolving_provider::{Artifact, EvolvingProvider, TrainingBatch};

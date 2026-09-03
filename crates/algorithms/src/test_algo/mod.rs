//! 测试算法集合：路由契约在 [`algorithm_provider`]，自演进代码在 [`evolving`]。

pub mod algorithm_provider;
pub mod evolving;
pub mod test_algorithm;

pub use algorithm_provider::{AlgorithmProvider, RouteContext};

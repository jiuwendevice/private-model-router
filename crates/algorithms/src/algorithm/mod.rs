//! 路由算法公共层：契约在 [`algorithm_provider`]，示意实现在 [`test_algorithm`]。

pub mod algorithm_provider;
pub mod test_algorithm;

pub use algorithm_provider::{AlgorithmProvider, RouteContext};

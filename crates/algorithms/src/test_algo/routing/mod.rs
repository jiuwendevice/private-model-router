//! 示意算法：与 Python `test_algo` 包同名同功能的骨架实现。
//!
//! 不是必须编进产物的「唯一实现」。按 `algo-*` feature 条件编译；
//! 配置选用 Python 版时关闭对应 feature，磁盘源码保留。

#[cfg(feature = "algo-ensemble")]
pub mod ensemble;
#[cfg(feature = "algo-passthrough")]
pub mod passthrough;
#[cfg(feature = "algo-rule_cascade")]
pub mod rule_cascade;
#[cfg(feature = "algo-signal")]
pub mod signal;
#[cfg(feature = "algo-weighted")]
pub mod weighted;

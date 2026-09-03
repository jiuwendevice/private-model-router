//! 示意自演进：按 `evolving-*` feature 条件编译的骨架实现。
//!
//! 不是必须编进产物的「唯一实现」。磁盘源码保留；关闭 feature 则不入产物。

#[cfg(feature = "evolving-mf")]
pub mod mf;

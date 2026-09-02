//! L3 算法层。各内置算法按 `algo-*` feature 条件编译；契约唯一在 [`algorithm::Algorithm`]。

pub mod algorithm;
pub mod evolving;

#[cfg(feature = "algo-passthrough")]
pub mod passthrough;
#[cfg(feature = "algo-weighted")]
pub mod weighted;
#[cfg(feature = "algo-rule_cascade")]
pub mod rule_cascade;
#[cfg(feature = "algo-signal")]
pub mod signal;
#[cfg(feature = "algo-ensemble")]
pub mod ensemble;

pub use algorithm::{Algorithm, RouteContext};

//! 状态公共层：契约在 [`state_provider`]，内置实现在 [`test_state`]。

pub mod state_provider;
pub mod test_state;

pub use state_provider::{CasConflict, StateProvider};

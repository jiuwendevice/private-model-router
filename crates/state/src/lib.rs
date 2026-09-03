//! L2 状态管理层。契约在 [`state`]；内置实现在 [`state::test_state`]。
//!
//! runtime 只面向 [`StateProvider`]；内存或远程由 profile 决定。

pub mod state;

#[cfg(feature = "service")]
pub mod service;

pub use state::{CasConflict, StateProvider};
pub use state::test_state::{MemoryState, RemoteState};

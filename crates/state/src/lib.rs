//! L2 状态管理层。runtime 只面向 [`StateProvider`]；内存或远程由 profile 决定。

pub mod memory;
pub mod provider;
pub mod remote;

#[cfg(feature = "service")]
pub mod service;

pub use memory::MemoryState;
pub use provider::{CasConflict, StateProvider};
pub use remote::RemoteState;

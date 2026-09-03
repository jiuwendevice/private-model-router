//! 内置 state 实现：端侧 `memory`、云侧 `remote`。
//!
//! 不是必须编进产物的「唯一实现」。profile `state.backend` 选一；
//! 云侧也可注入 PyO3 `StateClient`（内部仍是 [`remote::RemoteState`]）。

pub mod memory;
pub mod remote;

pub use memory::MemoryState;
pub use remote::RemoteState;

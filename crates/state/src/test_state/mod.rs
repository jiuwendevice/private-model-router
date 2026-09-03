//! 内置 state 实现：端侧 `memory`、云侧 `remote`。
//!
//! 不是必须编进产物的「唯一实现」。profile `state.backend` 选一
//!（`memory` / `remote`）；云侧自定义实现走 Python `StateProvider`。

pub mod memory;
pub mod remote;

pub use memory::MemoryState;
pub use remote::RemoteState;

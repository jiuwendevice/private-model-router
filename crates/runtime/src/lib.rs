//! L4 装配与运行。宿主只看 [`Router`] 一个门面。

pub mod config;
pub mod decide_loop;
pub mod registry;
pub mod router;
pub mod training;
pub mod trigger;

pub use config::RouterProfile;
pub use openjiuwen_protocol::{
    Decision, Feedback, Message, Outcome, RequestMetadata, RouteHint, RouteRequest, RouterError,
    RoutingKey,
};
pub use registry::create_algorithm;
pub use router::{KvCacheCoordinator, Router};

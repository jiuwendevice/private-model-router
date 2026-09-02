//! Router 门面：from_config / route / report。

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use openjiuwen_algorithms::Algorithm;
use openjiuwen_protocol::{Decision, Feedback, RouteHint, RouteRequest, RouterError, TargetSet};
use openjiuwen_state::{MemoryState, RemoteState, StateProvider};

use crate::config::RouterProfile;
use crate::decide_loop;
use crate::registry;

/// 模型切换时的 KV cache 协调回调。骨架仅保存，不触发。
pub trait KvCacheCoordinator: Send + Sync {
    fn on_switch(&self, from: &str, to: &str);
}

/// 已装配的路由实例。运行期算法槽与 state 槽各生效一个。
pub struct Router {
    algorithm: Box<dyn Algorithm>,
    state: Arc<dyn StateProvider>,
    targets: TargetSet,
    seed: AtomicU64,
    #[allow(dead_code)]
    kv_coordinator: Option<Box<dyn KvCacheCoordinator>>,
}

impl Router {
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self, RouterError> {
        Self::from_profile(RouterProfile::from_path(path)?)
    }

    pub fn from_toml(text: &str) -> Result<Self, RouterError> {
        Self::from_profile(RouterProfile::from_toml(text)?)
    }

    /// 从配置文件创建路由实例。返回的是 Result<Router, RouterError> 类型。
    pub fn from_profile(profile: RouterProfile) -> Result<Self, RouterError> {
        let algorithm = registry::create_algorithm(&profile.algorithm)?;
        let state: Arc<dyn StateProvider> = match profile.state.backend.as_str() {
            // 内存状态实现。
            "memory" => {
                let ttl: Duration = Duration::from_secs(profile.state.ttl_secs.unwrap_or(300));
                let cap = profile.state.max_entries.unwrap_or(1024);
                Arc::new(MemoryState::new(ttl, cap))
            }
            // 远程状态实现。必须在 profile 里给出 endpoint，不内置默认地址。
            "remote" => {
                let endpoint = profile.state.endpoint.clone().ok_or_else(|| {
                    RouterError::Config("remote state requires endpoint".into())
                })?;
                let timeout = Duration::from_millis(profile.state.timeout_ms.unwrap_or(5));
                Arc::new(RemoteState::new(endpoint, timeout))
            }
            other => {
                return Err(RouterError::Config(format!("unknown state backend: {other}")));
            }
        };
        let targets: TargetSet = TargetSet::new(profile.targets.models);
        Ok(Self {
            algorithm,
            state,
            targets,
            seed: AtomicU64::new(0),
            kv_coordinator: None,
        })
    }

    /// 驱动决策循环。`hint` 携带 cache_affinity 等每请求输入。
    /// 返回的是 Result<Decision, RouterError> 类型。
    pub fn route(&self, req: &RouteRequest, hint: &RouteHint) -> Result<Decision, RouterError> {
        let seed = self.seed.fetch_add(1, Ordering::Relaxed);
        decide_loop::run(    // 运行决策循环。
            self.algorithm.as_ref(),
            self.state.as_ref(),
            req,
            hint,
            &self.targets,
            seed,
        )
    }

    /// 转发状态层；立即返回，不等待写回完成。
    pub fn report(&self, feedback: Feedback) {
        self.state.report(feedback);
    }

    pub fn with_kv_coordinator(mut self, cb: Box<dyn KvCacheCoordinator>) -> Self {
        self.kv_coordinator = Some(cb);
        self
    }

    pub fn algorithm_name(&self) -> &str {
        self.algorithm.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openjiuwen_protocol::{RequestMetadata, RouteRequest};

    #[test]
    fn passthrough_picks_first_target() {
        let toml = r#"
algorithm = "passthrough"
[state]
backend = "memory"
[targets]
models = ["alpha", "beta"]
"#;
        let router = Router::from_toml(toml).expect("assemble");
        let req = RouteRequest {
            metadata: RequestMetadata {
                session_id: Some("s1".into()),
                agent_id: Some("a1".into()),
            },
            ..RouteRequest::default()
        };
        let d = router.route(&req, &RouteHint::default()).expect("route");
        assert_eq!(d.selected_model_id, "alpha");
        assert!(d.is_answer_call);
    }
}

//! L5 PyO3 绑定。Python 宿主经 `_openjiuwen` 调用 runtime::Router。
//!
//! 薄门面：类型转换 + 同步调用；路由逻辑全部在 Rust 内核。

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use openjiuwen_protocol::{
    Feedback, Outcome, RequestMetadata, RouteHint, RouteRequest, RoutingKey,
};
use openjiuwen_runtime::Router;

#[pyclass(name = "Router")]
pub struct PyRouter {
    inner: Router,
}

#[pymethods]
impl PyRouter {
    #[staticmethod]
    fn from_config(path: &str) -> PyResult<Self> {
        let inner = Router::from_config(path).map_err(to_py)?;
        Ok(Self { inner })
    }

    /// 同步决策。云侧门面再包一层 async。
    fn route(&self, selected_hint: Option<&str>) -> PyResult<PyDecision> {
        let req = RouteRequest {
            metadata: RequestMetadata::default(),
            ..RouteRequest::default()
        };
        let hint = RouteHint {
            cache_affinity: selected_hint.map(str::to_string),
        };
        let d = self.inner.route(&req, &hint).map_err(to_py)?;
        Ok(PyDecision {
            selected_model_id: d.selected_model_id,
            reasoning: d.reasoning,
            is_answer_call: d.is_answer_call,
        })
    }

    fn report(&self, selected_model_id: &str, latency_ms: u64) {
        self.inner.report(Feedback {
            key: RoutingKey::default(),
            selected_model_id: selected_model_id.into(),
            outcome: Outcome::Ok,
            latency_ms,
            cache_valid: None,
        });
    }

    fn algorithm_name(&self) -> &str {
        self.inner.algorithm_name()
    }
}

#[pyclass(name = "Decision")]
#[derive(Clone)]
pub struct PyDecision {
    #[pyo3(get)]
    pub selected_model_id: String,
    #[pyo3(get)]
    pub reasoning: String,
    #[pyo3(get)]
    pub is_answer_call: bool,
}

fn to_py(err: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(err.to_string())
}

#[pymodule]
fn _openjiuwen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRouter>()?;
    m.add_class::<PyDecision>()?;
    Ok(())
}

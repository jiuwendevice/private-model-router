//! L5 PyO3 绑定。Python 宿主经 `_openjiuwen` 调用 runtime::Router。
//!
//! 薄门面：类型转换 + 同步调用；路由逻辑全部在 Rust 内核。
//! 云侧 async 由 `python/openjiuwen` 再包一层。

mod adapter;
mod convert;
mod error;
mod types;

use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;

use openjiuwen_protocol::TargetSet;
use openjiuwen_runtime::{registry, KvCacheCoordinator, Router};
use openjiuwen_state::{RemoteState, StateProvider};

use crate::convert::{extract_hint, extract_request, profile_from_obj};
use crate::error::to_py;
use crate::types::{
    PyFeedback, PyFeedbackStats, PyMessage, PyModelSelection, PyRequestMetadata, PyRouteContext,
    PyRouteHint, PyRouteRequest, PyRoutingKey, PyStateView,
};

/// 云侧远程状态客户端，对应 `RemoteState`。可注入 Router 覆盖 profile。
#[pyclass(name = "StateClient")]
#[derive(Clone)]
pub struct PyStateClient {
    inner: Arc<RemoteState>,
}

#[pymethods]
impl PyStateClient {
    #[new]
    #[pyo3(signature = (endpoint, timeout_ms=5))]
    fn new(endpoint: String, timeout_ms: u64) -> PyResult<Self> {
        if endpoint.trim().is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "StateClient requires endpoint",
            ));
        }
        Ok(Self {
            inner: Arc::new(RemoteState::new(endpoint, Duration::from_millis(timeout_ms))),
        })
    }

    #[getter]
    fn endpoint(&self) -> &str {
        &self.inner.endpoint
    }

    #[getter]
    fn timeout_ms(&self) -> u64 {
        self.inner.timeout.as_millis() as u64
    }
}

struct PyKvCoordinator {
    cb: Py<PyAny>,
}

impl KvCacheCoordinator for PyKvCoordinator {
    fn on_switch(&self, from: &str, to: &str) {
        Python::with_gil(|py| {
            let _ = self.cb.bind(py).call1((from, to));
        });
    }
}

#[pyclass(name = "Router")]
pub struct PyRouter {
    inner: Router,
}

#[pymethods]
impl PyRouter {
    /// `from_config(path | dict, *, state=None)`。dict 便于配置中心注入。
    #[staticmethod]
    #[pyo3(signature = (config, *, state=None))]
    fn from_config(config: Bound<'_, PyAny>, state: Option<Bound<'_, PyAny>>) -> PyResult<Self> {
        assemble(profile_from_obj(&config)?, state)
    }

    #[staticmethod]
    fn from_toml(text: &str) -> PyResult<Self> {
        assemble(
            openjiuwen_runtime::RouterProfile::from_toml(text).map_err(to_py)?,
            None,
        )
    }

    /// 同步决策。接受 RouteRequest 或 dict；hint 可为 RouteHint / str / dict / None。
    /// 跨边界返回 ModelSelection（Decision 的投影）。
    #[pyo3(signature = (request, hint=None))]
    fn route(
        &self,
        request: Bound<'_, PyAny>,
        hint: Option<Bound<'_, PyAny>>,
    ) -> PyResult<PyModelSelection> {
        let req = extract_request(&request)?;
        let hint = extract_hint(hint.as_ref())?;
        let d = self.inner.route(&req, &hint).map_err(to_py)?;
        Ok(PyModelSelection::from_decision(&d))
    }

    fn report(&self, feedback: Bound<'_, PyAny>) -> PyResult<()> {
        let fb = if let Ok(typed) = feedback.extract::<PyRef<PyFeedback>>() {
            typed.native()?
        } else {
            extract_feedback_dict(&feedback)?
        };
        self.inner.report(fb);
        Ok(())
    }

    fn algorithm_name(&self) -> &str {
        self.inner.algorithm_name()
    }

    fn with_kv_coordinator(&mut self, cb: Bound<'_, PyAny>) {
        self.inner
            .set_kv_coordinator(Box::new(PyKvCoordinator { cb: cb.unbind() }));
    }

    fn replace_algorithm(&mut self, obj: Bound<'_, PyAny>) -> PyResult<()> {
        let algo = adapter::adapter_from_obj(obj)?;
        self.inner.replace_algorithm(algo);
        Ok(())
    }

    fn replace_state(&mut self, state: Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.replace_state(extract_state(&state)?);
        Ok(())
    }
}

fn assemble(
    profile: openjiuwen_runtime::RouterProfile,
    state: Option<Bound<'_, PyAny>>,
) -> PyResult<PyRouter> {
    let algorithm = match adapter::lookup(&profile.algorithm) {
        Some(algo) => algo,
        None => registry::create_algorithm(&profile.algorithm).map_err(to_py)?,
    };
    let state: Arc<dyn StateProvider> = match state {
        Some(obj) => extract_state(&obj)?,
        None => Router::state_from_profile(&profile).map_err(to_py)?,
    };
    Ok(PyRouter {
        inner: Router::from_parts(algorithm, state, TargetSet::new(profile.targets.models)),
    })
}

fn extract_state(obj: &Bound<'_, PyAny>) -> PyResult<Arc<dyn StateProvider>> {
    if let Ok(client) = obj.extract::<PyRef<PyStateClient>>() {
        return Ok(client.inner.clone() as Arc<dyn StateProvider>);
    }
    Err(PyTypeError::new_err("state must be openjiuwen.StateClient"))
}

fn extract_feedback_dict(obj: &Bound<'_, PyAny>) -> PyResult<openjiuwen_protocol::Feedback> {
    use pyo3::types::PyDict;
    let dict = obj
        .downcast::<PyDict>()
        .map_err(|_| pyo3::exceptions::PyValueError::new_err("feedback must be Feedback or dict"))?;
    let key = match dict.get_item("key")? {
        Some(k) if !k.is_none() => convert::extract_routing_key(&k)?,
        _ => openjiuwen_protocol::RoutingKey {
            session_id: dict_str_or_empty(dict, "session_id")?,
            agent_id: dict_str_or_empty(dict, "agent_id")?,
        },
    };
    let selected = dict
        .get_item("selected_model_id")?
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("feedback needs selected_model_id"))?
        .extract()?;
    let outcome = match dict.get_item("outcome")? {
        Some(v) if !v.is_none() => {
            let s: String = v.extract()?;
            convert::parse_outcome(&s)?
        }
        _ => openjiuwen_protocol::Outcome::Ok,
    };
    let latency_ms = match dict.get_item("latency_ms")? {
        Some(v) if !v.is_none() => v.extract()?,
        _ => 0,
    };
    let cache_valid = match dict.get_item("cache_valid")? {
        Some(v) if !v.is_none() => Some(v.extract()?),
        _ => None,
    };
    Ok(openjiuwen_protocol::Feedback {
        key,
        selected_model_id: selected,
        outcome,
        latency_ms,
        cache_valid,
    })
}

fn dict_str_or_empty(dict: &Bound<'_, pyo3::types::PyDict>, key: &str) -> PyResult<String> {
    match dict.get_item(key)? {
        Some(v) if !v.is_none() => v.extract(),
        _ => Ok(String::new()),
    }
}

#[pymodule]
fn _openjiuwen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRouter>()?;
    m.add_class::<PyModelSelection>()?;
    m.add_class::<PyRouteRequest>()?;
    m.add_class::<PyRouteHint>()?;
    m.add_class::<PyMessage>()?;
    m.add_class::<PyRequestMetadata>()?;
    m.add_class::<PyRoutingKey>()?;
    m.add_class::<PyFeedback>()?;
    m.add_class::<PyStateClient>()?;
    m.add_class::<PyStateView>()?;
    m.add_class::<PyFeedbackStats>()?;
    m.add_class::<PyRouteContext>()?;
    m.add_function(wrap_pyfunction!(adapter::register_algorithm, m)?)?;
    m.add("Decision", m.getattr("ModelSelection")?)?;
    m.add("OK", "ok")?;
    m.add("OVERFLOW", "overflow")?;
    m.add("UNAVAILABLE", "unavailable")?;
    m.add("REJECTED", "rejected")?;
    Ok(())
}

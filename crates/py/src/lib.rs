//! L5 PyO3 绑定。Python 宿主经 `_openjiuwen` 调用 runtime::Router。
//!
//! 正向边界导出 Router；反向边界把 Python 算法对象适配为 Rust
//! `Algorithm` trait。路由逻辑仍全部在 Rust runtime 内核中。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use openjiuwen_algorithms::{Algorithm, RouteContext};
use openjiuwen_protocol::{
    Decision, Feedback, Outcome, RequestMetadata, RouteHint, RouteRequest, RouterError, RoutingKey,
};
use openjiuwen_runtime::{Router, RouterProfile};
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyAnyMethods, PyDict, PyDictMethods, PyTracebackMethods};

type PythonAlgorithmRegistry = Mutex<HashMap<String, Py<PyAny>>>;

fn python_algorithms() -> &'static PythonAlgorithmRegistry {
    static REGISTRY: OnceLock<PythonAlgorithmRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[pyclass(name = "Router")]
pub struct PyRouter {
    inner: Router,
}

#[pymethods]
impl PyRouter {
    #[staticmethod]
    fn from_config(py: Python<'_>, path: &str) -> PyResult<Self> {
        let profile = RouterProfile::from_path(path).map_err(to_py)?;
        let registered = registered_algorithm(py, &profile.algorithm)?;
        let inner = match registered {
            Some(inner) => {
                let adapter = PyAlgorithmAdapter::new(profile.algorithm.clone(), inner);
                Router::from_profile_with_algorithm(profile, Box::new(adapter)).map_err(to_py)?
            }
            None => Router::from_profile(profile).map_err(to_py)?,
        };
        Ok(Self { inner })
    }

    /// 同步决策。Python 算法回调时会重新获取 GIL。
    #[pyo3(signature = (selected_hint=None))]
    fn route(&self, py: Python<'_>, selected_hint: Option<&str>) -> PyResult<PyDecision> {
        let req = RouteRequest {
            metadata: RequestMetadata::default(),
            ..RouteRequest::default()
        };
        let hint = RouteHint {
            cache_affinity: selected_hint.map(str::to_string),
        };
        let decision = py
            .allow_threads(|| self.inner.route(&req, &hint))
            .map_err(to_py)?;
        Ok(decision.into())
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

#[pyclass(name = "Decision", frozen)]
#[derive(Clone, PartialEq, Eq)]
pub struct PyDecision {
    #[pyo3(get)]
    pub selected_model_id: String,
    #[pyo3(get)]
    pub reasoning: String,
    #[pyo3(get)]
    pub is_answer_call: bool,
}

#[pymethods]
impl PyDecision {
    #[new]
    #[pyo3(signature = (selected_model_id, reasoning, is_answer_call=true))]
    fn new(selected_model_id: String, reasoning: String, is_answer_call: bool) -> Self {
        Self {
            selected_model_id,
            reasoning,
            is_answer_call,
        }
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    fn __repr__(&self) -> String {
        format!(
            "Decision(selected_model_id={:?}, reasoning={:?}, is_answer_call={})",
            self.selected_model_id, self.reasoning, self.is_answer_call
        )
    }
}

impl From<Decision> for PyDecision {
    fn from(value: Decision) -> Self {
        Self {
            selected_model_id: value.selected_model_id,
            reasoning: value.reasoning,
            is_answer_call: value.is_answer_call,
        }
    }
}

/// 传给 Python 算法的只读请求视图。
#[pyclass(name = "RouteRequest", frozen)]
pub struct PyRouteRequest {
    #[pyo3(get)]
    messages: Vec<(String, String)>,
    #[pyo3(get)]
    session_id: Option<String>,
    #[pyo3(get)]
    agent_id: Option<String>,
    #[pyo3(get)]
    exclusions: Vec<String>,
}

impl From<&RouteRequest> for PyRouteRequest {
    fn from(value: &RouteRequest) -> Self {
        Self {
            messages: value
                .messages
                .iter()
                .map(|message| (message.role.clone(), message.content.clone()))
                .collect(),
            session_id: value.metadata.session_id.clone(),
            agent_id: value.metadata.agent_id.clone(),
            exclusions: value.exclusions.clone(),
        }
    }
}

/// 传给 Python 算法的只读路由上下文视图。
#[pyclass(name = "RouteContext", frozen)]
pub struct PyRouteContext {
    #[pyo3(get)]
    targets: Vec<String>,
    #[pyo3(get)]
    affinity: Option<String>,
    #[pyo3(get)]
    state_exclusions: Vec<String>,
    #[pyo3(get)]
    sample_count: u64,
    #[pyo3(get)]
    seed: u64,
}

impl From<&RouteContext> for PyRouteContext {
    fn from(value: &RouteContext) -> Self {
        Self {
            targets: value.targets.models.clone(),
            affinity: value.view.affinity.clone(),
            state_exclusions: value.view.exclusions.clone(),
            sample_count: value.view.stats.sample_count,
            seed: value.seed,
        }
    }
}

/// Python 对象到 Rust `Algorithm` trait 的反向 PyO3 适配器。
struct PyAlgorithmAdapter {
    name: String,
    // `Py<PyAny>` 是 Send 但不是 Sync；Mutex 使适配器满足 Algorithm 的线程安全契约。
    inner: Mutex<Py<PyAny>>,
}

impl PyAlgorithmAdapter {
    fn new(name: String, inner: Py<PyAny>) -> Self {
        Self {
            name,
            inner: Mutex::new(inner),
        }
    }
}

impl Algorithm for PyAlgorithmAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&self, request: &RouteRequest, ctx: &RouteContext) -> Result<Decision, RouterError> {
        Python::with_gil(|py| {
            let request = Py::new(py, PyRouteRequest::from(request))?;
            let ctx = Py::new(py, PyRouteContext::from(ctx))?;
            let inner = self
                .inner
                .lock()
                .map_err(|_| PyRuntimeError::new_err("Python algorithm lock is poisoned"))?;
            let result = inner.call_method1(py, "decide", (request, ctx))?;
            extract_decision(result.bind(py))
        })
        .map_err(|err| {
            let details = Python::with_gil(|py| format_python_error(py, &err));
            RouterError::Algorithm(format!(
                "Python algorithm `{}` failed: {details}",
                self.name
            ))
        })
    }
}

fn extract_decision(value: &Bound<'_, PyAny>) -> PyResult<Decision> {
    let selected_model_id = extract_field::<String>(value, "selected_model_id")?;
    if selected_model_id.is_empty() {
        return Err(PyValueError::new_err(
            "Decision.selected_model_id must not be empty",
        ));
    }
    let reasoning = extract_field::<String>(value, "reasoning")?;
    if reasoning.is_empty() {
        return Err(PyValueError::new_err(
            "Decision.reasoning must not be empty",
        ));
    }
    let is_answer_call = extract_field::<bool>(value, "is_answer_call")?;
    Ok(Decision {
        selected_model_id,
        reasoning,
        is_answer_call,
    })
}

fn extract_field<'py, T>(value: &Bound<'py, PyAny>, name: &str) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    if let Ok(mapping) = value.downcast::<PyDict>() {
        return mapping
            .get_item(name)?
            .ok_or_else(|| PyTypeError::new_err(format!("Decision mapping is missing `{name}`")))?
            .extract();
    }
    value.getattr(name)?.extract()
}

#[pyfunction]
fn register_algorithm(py: Python<'_>, algorithm: Py<PyAny>) -> PyResult<String> {
    let bound = algorithm.bind(py);
    let decide = bound.getattr("decide")?;
    if !decide.is_callable() {
        return Err(PyTypeError::new_err(
            "Python algorithm must define callable decide(request, ctx)",
        ));
    }
    let name: String = bound.getattr("name")?.extract()?;
    if name.trim().is_empty() {
        return Err(PyValueError::new_err(
            "Python algorithm name must not be empty",
        ));
    }

    let mut registry = python_algorithms()
        .lock()
        .map_err(|_| PyRuntimeError::new_err("Python algorithm registry lock is poisoned"))?;
    let previous = registry.insert(name.clone(), algorithm);
    drop(registry);
    drop(previous);
    Ok(name)
}

#[pyfunction]
fn unregister_algorithm(name: &str) -> PyResult<bool> {
    let removed = {
        let mut registry = python_algorithms()
            .lock()
            .map_err(|_| PyRuntimeError::new_err("Python algorithm registry lock is poisoned"))?;
        registry.remove(name)
    };
    Ok(removed.is_some())
}

fn registered_algorithm(py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
    let registry = python_algorithms()
        .lock()
        .map_err(|_| PyRuntimeError::new_err("Python algorithm registry lock is poisoned"))?;
    Ok(registry.get(name).map(|value| value.clone_ref(py)))
}

fn format_python_error(py: Python<'_>, err: &PyErr) -> String {
    let traceback = err
        .traceback(py)
        .and_then(|traceback| traceback.format().ok())
        .unwrap_or_default();
    format!("{traceback}{err}")
}

fn to_py(err: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(err.to_string())
}

#[pymodule]
fn _openjiuwen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRouter>()?;
    m.add_class::<PyDecision>()?;
    m.add_class::<PyRouteRequest>()?;
    m.add_class::<PyRouteContext>()?;
    m.add_function(wrap_pyfunction!(register_algorithm, m)?)?;
    m.add_function(wrap_pyfunction!(unregister_algorithm, m)?)?;
    Ok(())
}

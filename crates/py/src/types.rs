//! 跨 PyO3 边界的协议类型。蓝图：RouteRequest / ModelSelection / Feedback。

use pyo3::prelude::*;
use pyo3::types::PyType;

use openjiuwen_protocol::{
    Decision, Feedback, FeedbackStats, Message, ModelSelection, RequestMetadata, RouteHint,
    RouteRequest, RoutingKey, StateView,
};

use crate::convert;

#[pyclass(name = "Message", get_all, set_all)]
#[derive(Clone, Debug)]
pub struct PyMessage {
    pub role: String,
    pub content: String,
}

#[pymethods]
impl PyMessage {
    #[new]
    #[pyo3(signature = (role, content))]
    fn new(role: String, content: String) -> Self {
        Self { role, content }
    }
}

impl From<&PyMessage> for Message {
    fn from(m: &PyMessage) -> Self {
        Self {
            role: m.role.clone(),
            content: m.content.clone(),
        }
    }
}

impl From<&Message> for PyMessage {
    fn from(m: &Message) -> Self {
        Self {
            role: m.role.clone(),
            content: m.content.clone(),
        }
    }
}

#[pyclass(name = "RequestMetadata", get_all, set_all)]
#[derive(Clone, Debug, Default)]
pub struct PyRequestMetadata {
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
}

#[pymethods]
impl PyRequestMetadata {
    #[new]
    #[pyo3(signature = (session_id=None, agent_id=None))]
    fn new(session_id: Option<String>, agent_id: Option<String>) -> Self {
        Self {
            session_id,
            agent_id,
        }
    }

    fn routing_key(&self) -> PyRoutingKey {
        PyRoutingKey::from(&self.native().routing_key())
    }
}

impl PyRequestMetadata {
    pub fn native(&self) -> RequestMetadata {
        RequestMetadata {
            session_id: self.session_id.clone(),
            agent_id: self.agent_id.clone(),
        }
    }
}

impl From<&RequestMetadata> for PyRequestMetadata {
    fn from(m: &RequestMetadata) -> Self {
        Self {
            session_id: m.session_id.clone(),
            agent_id: m.agent_id.clone(),
        }
    }
}

#[pyclass(name = "RoutingKey", get_all, set_all)]
#[derive(Clone, Debug, Default)]
pub struct PyRoutingKey {
    pub session_id: String,
    pub agent_id: String,
}

#[pymethods]
impl PyRoutingKey {
    #[new]
    #[pyo3(signature = (session_id=None, agent_id=None))]
    fn new(session_id: Option<String>, agent_id: Option<String>) -> Self {
        Self {
            session_id: session_id.unwrap_or_default(),
            agent_id: agent_id.unwrap_or_default(),
        }
    }
}

impl PyRoutingKey {
    pub fn native(&self) -> RoutingKey {
        RoutingKey {
            session_id: self.session_id.clone(),
            agent_id: self.agent_id.clone(),
        }
    }
}

impl From<&RoutingKey> for PyRoutingKey {
    fn from(k: &RoutingKey) -> Self {
        Self {
            session_id: k.session_id.clone(),
            agent_id: k.agent_id.clone(),
        }
    }
}

#[pyclass(name = "RouteHint", get_all, set_all)]
#[derive(Clone, Debug, Default)]
pub struct PyRouteHint {
    pub cache_affinity: Option<String>,
}

#[pymethods]
impl PyRouteHint {
    #[new]
    #[pyo3(signature = (cache_affinity=None))]
    fn new(cache_affinity: Option<String>) -> Self {
        Self { cache_affinity }
    }
}

impl PyRouteHint {
    pub fn native(&self) -> RouteHint {
        RouteHint {
            cache_affinity: self.cache_affinity.clone(),
        }
    }
}

#[pyclass(name = "RouteRequest")]
#[derive(Clone, Debug, Default)]
pub struct PyRouteRequest {
    #[pyo3(get, set)]
    pub messages: Vec<PyMessage>,
    #[pyo3(get, set)]
    pub metadata: PyRequestMetadata,
    #[pyo3(get, set)]
    pub exclusions: Vec<String>,
}

#[pymethods]
impl PyRouteRequest {
    #[new]
    #[pyo3(signature = (messages=None, metadata=None, exclusions=None))]
    fn new(
        messages: Option<Vec<PyMessage>>,
        metadata: Option<PyRequestMetadata>,
        exclusions: Option<Vec<String>>,
    ) -> Self {
        Self {
            messages: messages.unwrap_or_default(),
            metadata: metadata.unwrap_or_default(),
            exclusions: exclusions.unwrap_or_default(),
        }
    }

    fn routing_key(&self) -> PyRoutingKey {
        PyRoutingKey::from(&self.native().routing_key())
    }
}

impl PyRouteRequest {
    pub fn native(&self) -> RouteRequest {
        RouteRequest {
            messages: self.messages.iter().map(Message::from).collect(),
            metadata: self.metadata.native(),
            exclusions: self.exclusions.clone(),
        }
    }

    pub fn from_native(req: &RouteRequest) -> Self {
        Self {
            messages: req.messages.iter().map(PyMessage::from).collect(),
            metadata: PyRequestMetadata::from(&req.metadata),
            exclusions: req.exclusions.clone(),
        }
    }
}

#[pyclass(name = "FeedbackStats", get_all)]
#[derive(Clone, Debug, Default)]
pub struct PyFeedbackStats {
    pub sample_count: u64,
}

impl From<&FeedbackStats> for PyFeedbackStats {
    fn from(s: &FeedbackStats) -> Self {
        Self {
            sample_count: s.sample_count,
        }
    }
}

#[pyclass(name = "StateView", get_all)]
#[derive(Clone, Debug, Default)]
pub struct PyStateView {
    pub affinity: Option<String>,
    pub exclusions: Vec<String>,
    pub stats: PyFeedbackStats,
}

impl From<&StateView> for PyStateView {
    fn from(v: &StateView) -> Self {
        Self {
            affinity: v.affinity.clone(),
            exclusions: v.exclusions.clone(),
            stats: PyFeedbackStats::from(&v.stats),
        }
    }
}

#[pyclass(name = "RouteContext", get_all)]
#[derive(Clone, Debug)]
pub struct PyRouteContext {
    /// 与 Python 内置算法兼容：直接是模型名列表，不是 TargetSet 包装。
    pub targets: Vec<String>,
    pub view: PyStateView,
    pub seed: u64,
}

#[pyclass(name = "ModelSelection", get_all)]
#[derive(Clone, Debug)]
pub struct PyModelSelection {
    pub selected_model_id: String,
    pub reasoning: String,
    pub is_answer_call: bool,
}

#[pymethods]
impl PyModelSelection {
    #[new]
    #[pyo3(signature = (selected_model_id, reasoning, is_answer_call=true))]
    fn new(selected_model_id: String, reasoning: String, is_answer_call: bool) -> Self {
        Self {
            selected_model_id,
            reasoning,
            is_answer_call,
        }
    }

    /// 蓝图样例里的 `decision.target` 别名。
    #[getter]
    fn target(&self) -> &str {
        &self.selected_model_id
    }
}

impl PyModelSelection {
    pub fn from_decision(d: &Decision) -> Self {
        let sel = ModelSelection::from(d);
        Self {
            selected_model_id: sel.selected_model_id,
            reasoning: sel.reasoning,
            is_answer_call: sel.is_answer_call,
        }
    }

    pub fn to_decision(&self) -> Decision {
        Decision {
            selected_model_id: self.selected_model_id.clone(),
            reasoning: self.reasoning.clone(),
            is_answer_call: self.is_answer_call,
        }
    }
}

#[pyclass(name = "Feedback")]
#[derive(Clone, Debug)]
pub struct PyFeedback {
    #[pyo3(get, set)]
    pub key: PyRoutingKey,
    #[pyo3(get, set)]
    pub selected_model_id: String,
    #[pyo3(get, set)]
    pub outcome: String,
    #[pyo3(get, set)]
    pub latency_ms: u64,
    #[pyo3(get, set)]
    pub cache_valid: Option<bool>,
}

#[pymethods]
impl PyFeedback {
    #[new]
    #[pyo3(signature = (key, selected_model_id, outcome="ok", latency_ms=0, cache_valid=None))]
    fn new(
        key: PyRoutingKey,
        selected_model_id: String,
        outcome: &str,
        latency_ms: u64,
        cache_valid: Option<bool>,
    ) -> PyResult<Self> {
        convert::parse_outcome(outcome)?;
        Ok(Self {
            key,
            selected_model_id,
            outcome: outcome.to_ascii_lowercase(),
            latency_ms,
            cache_valid,
        })
    }

    /// `Feedback.ok(decision, latency_ms=..., key=...)`。
    #[classmethod]
    #[pyo3(signature = (decision, latency_ms, *, key=None, session_id=None, agent_id=None, selected_model_id=None, cache_valid=None, outcome="ok"))]
    fn ok(
        _cls: &Bound<'_, PyType>,
        decision: Bound<'_, PyAny>,
        latency_ms: u64,
        key: Option<Bound<'_, PyAny>>,
        session_id: Option<String>,
        agent_id: Option<String>,
        selected_model_id: Option<String>,
        cache_valid: Option<bool>,
        outcome: &str,
    ) -> PyResult<Self> {
        let model = match selected_model_id {
            Some(id) => id,
            None => convert::selection_model_id(&decision)?,
        };
        let routing_key = match key {
            Some(k) => convert::extract_routing_key(&k)?,
            None => RoutingKey {
                session_id: session_id.unwrap_or_default(),
                agent_id: agent_id.unwrap_or_default(),
            },
        };
        convert::parse_outcome(outcome)?;
        Ok(Self {
            key: PyRoutingKey::from(&routing_key),
            selected_model_id: model,
            outcome: outcome.to_ascii_lowercase(),
            latency_ms,
            cache_valid,
        })
    }
}

impl PyFeedback {
    pub fn native(&self) -> PyResult<Feedback> {
        Ok(Feedback {
            key: self.key.native(),
            selected_model_id: self.selected_model_id.clone(),
            outcome: convert::parse_outcome(&self.outcome)?,
            latency_ms: self.latency_ms,
            cache_valid: self.cache_valid,
        })
    }
}

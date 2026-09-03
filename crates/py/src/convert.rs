//! Python 对象 → 协议类型。

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySequence, PyString, PyType};

use openjiuwen_algorithms::RouteContext;
use openjiuwen_protocol::{
    Message, Outcome, RequestMetadata, RouteHint, RouteRequest, RouterError, RoutingKey,
};
use openjiuwen_runtime::config::{RouterProfile, StateConfig, TargetsConfig};

use crate::types::{
    PyMessage, PyModelSelection, PyRequestMetadata, PyRouteContext, PyRouteHint, PyRouteRequest,
    PyRoutingKey,
};

pub fn parse_outcome(raw: &str) -> PyResult<Outcome> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ok" => Ok(Outcome::Ok),
        "overflow" => Ok(Outcome::Overflow),
        "unavailable" => Ok(Outcome::Unavailable),
        "rejected" => Ok(Outcome::Rejected),
        other => Err(PyValueError::new_err(format!(
            "unknown outcome: {other} (ok|overflow|unavailable|rejected)"
        ))),
    }
}

pub fn extract_routing_key(obj: &Bound<'_, PyAny>) -> PyResult<RoutingKey> {
    if obj.is_none() {
        return Ok(RoutingKey::default());
    }
    if let Ok(key) = obj.extract::<PyRef<PyRoutingKey>>() {
        return Ok(key.native());
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        return Ok(RoutingKey {
            session_id: opt_dict_str(dict, "session_id")?.unwrap_or_default(),
            agent_id: opt_dict_str(dict, "agent_id")?.unwrap_or_default(),
        });
    }
    Err(PyValueError::new_err(
        "routing key must be RoutingKey or dict with session_id/agent_id",
    ))
}

pub fn selection_model_id(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(sel) = obj.extract::<PyRef<PyModelSelection>>() {
        return Ok(sel.selected_model_id.clone());
    }
    if obj.hasattr("selected_model_id")? {
        return obj.getattr("selected_model_id")?.extract();
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        if let Some(v) = dict.get_item("selected_model_id")? {
            return v.extract();
        }
        if let Some(v) = dict.get_item("target")? {
            return v.extract();
        }
    }
    Err(PyValueError::new_err(
        "decision must expose selected_model_id (ModelSelection, dict, or attribute)",
    ))
}

pub fn extract_message(obj: &Bound<'_, PyAny>) -> PyResult<Message> {
    if let Ok(msg) = obj.extract::<PyRef<PyMessage>>() {
        return Ok(Message::from(&*msg));
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        return Ok(Message {
            role: dict_str(dict, "role")?.unwrap_or_default(),
            content: dict_str(dict, "content")?.unwrap_or_default(),
        });
    }
    if let Ok(seq) = obj.downcast::<PySequence>() {
        if seq.len()? >= 2 {
            return Ok(Message {
                role: seq.get_item(0)?.extract()?,
                content: seq.get_item(1)?.extract()?,
            });
        }
    }
    Err(PyValueError::new_err(
        "message must be Message, dict{role,content}, or (role, content)",
    ))
}

pub fn extract_metadata(obj: &Bound<'_, PyAny>) -> PyResult<RequestMetadata> {
    if obj.is_none() {
        return Ok(RequestMetadata::default());
    }
    if let Ok(meta) = obj.extract::<PyRef<PyRequestMetadata>>() {
        return Ok(meta.native());
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        return Ok(RequestMetadata {
            session_id: opt_dict_str(dict, "session_id")?,
            agent_id: opt_dict_str(dict, "agent_id")?,
        });
    }
    Err(PyValueError::new_err(
        "metadata must be RequestMetadata or dict",
    ))
}

pub fn extract_request(obj: &Bound<'_, PyAny>) -> PyResult<RouteRequest> {
    if let Ok(req) = obj.extract::<PyRef<PyRouteRequest>>() {
        return Ok(req.native());
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut messages = Vec::new();
        if let Some(raw) = dict.get_item("messages")? {
            if !raw.is_none() {
                for item in raw.try_iter()? {
                    messages.push(extract_message(&item?)?);
                }
            }
        }
        let metadata = match dict.get_item("metadata")? {
            Some(m) if !m.is_none() => extract_metadata(&m)?,
            _ => RequestMetadata {
                session_id: opt_dict_str(dict, "session_id")?,
                agent_id: opt_dict_str(dict, "agent_id")?,
            },
        };
        let exclusions = match dict.get_item("exclusions")? {
            Some(v) if !v.is_none() => v.extract()?,
            _ => Vec::new(),
        };
        return Ok(RouteRequest {
            messages,
            metadata,
            exclusions,
        });
    }
    Err(PyValueError::new_err(
        "request must be RouteRequest or dict with messages/metadata/exclusions",
    ))
}

pub fn extract_hint(obj: Option<&Bound<'_, PyAny>>) -> PyResult<RouteHint> {
    let Some(obj) = obj else {
        return Ok(RouteHint::default());
    };
    if obj.is_none() {
        return Ok(RouteHint::default());
    }
    if let Ok(hint) = obj.extract::<PyRef<PyRouteHint>>() {
        return Ok(hint.native());
    }
    if obj.downcast::<PyString>().is_ok() {
        return Ok(RouteHint {
            cache_affinity: Some(obj.extract()?),
        });
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        return Ok(RouteHint {
            cache_affinity: opt_dict_str(dict, "cache_affinity")?,
        });
    }
    Err(PyValueError::new_err(
        "hint must be RouteHint, str, dict, or None",
    ))
}

pub fn extract_decision(obj: &Bound<'_, PyAny>) -> PyResult<openjiuwen_protocol::Decision> {
    if let Ok(sel) = obj.extract::<PyRef<PyModelSelection>>() {
        return Ok(sel.to_decision());
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let selected = dict
            .get_item("selected_model_id")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .or_else(|| {
                dict.get_item("target")
                    .ok()
                    .flatten()
                    .and_then(|v| v.extract::<String>().ok())
            })
            .ok_or_else(|| PyValueError::new_err("decision dict needs selected_model_id"))?;
        let reasoning = dict
            .get_item("reasoning")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .unwrap_or_default();
        let is_answer_call = dict
            .get_item("is_answer_call")?
            .map(|v| v.extract::<bool>())
            .transpose()?
            .unwrap_or(true);
        return Ok(openjiuwen_protocol::Decision {
            selected_model_id: selected,
            reasoning,
            is_answer_call,
        });
    }
    if obj.hasattr("selected_model_id")? {
        return Ok(openjiuwen_protocol::Decision {
            selected_model_id: obj.getattr("selected_model_id")?.extract()?,
            reasoning: if obj.hasattr("reasoning")? {
                obj.getattr("reasoning")?.extract()?
            } else {
                String::new()
            },
            is_answer_call: if obj.hasattr("is_answer_call")? {
                obj.getattr("is_answer_call")?.extract()?
            } else {
                true
            },
        });
    }
    Err(PyValueError::new_err(
        "decide() must return ModelSelection or dict{selected_model_id, reasoning}",
    ))
}

pub fn py_route_context(py: Python<'_>, ctx: &RouteContext) -> PyResult<Py<PyRouteContext>> {
    Bound::new(
        py,
        PyRouteContext {
            targets: ctx.targets.models.clone(),
            view: crate::types::PyStateView::from(&ctx.view),
            seed: ctx.seed,
        },
    )
    .map(|b| b.unbind())
}

pub fn py_route_request(py: Python<'_>, req: &RouteRequest) -> PyResult<Py<PyRouteRequest>> {
    Bound::new(py, PyRouteRequest::from_native(req)).map(|b| b.unbind())
}

pub fn profile_from_obj(obj: &Bound<'_, PyAny>) -> PyResult<RouterProfile> {
    if let Ok(path) = obj.extract::<String>() {
        return RouterProfile::from_path(path).map_err(|e| match e {
            RouterError::Config(msg) => PyValueError::new_err(format!("config: {msg}")),
            other => PyValueError::new_err(other.to_string()),
        });
    }
    let dict = obj
        .downcast::<PyDict>()
        .map_err(|_| PyValueError::new_err("from_config expects a path string or dict"))?;
    profile_from_dict(dict)
}

pub fn profile_from_dict(dict: &Bound<'_, PyDict>) -> PyResult<RouterProfile> {
    let algorithm = dict
        .get_item("algorithm")?
        .ok_or_else(|| PyValueError::new_err("profile requires algorithm"))?
        .extract::<String>()?;
    let state = match dict.get_item("state")? {
        Some(s) if !s.is_none() => {
            let sd = s
                .downcast::<PyDict>()
                .map_err(|_| PyValueError::new_err("state must be a dict"))?;
            StateConfig {
                backend: dict_str(sd, "backend")?
                    .ok_or_else(|| PyValueError::new_err("state.backend is required"))?,
                ttl_secs: opt_dict_u64(sd, "ttl_secs")?,
                max_entries: opt_dict_usize(sd, "max_entries")?,
                endpoint: opt_dict_str(sd, "endpoint")?,
                timeout_ms: opt_dict_u64(sd, "timeout_ms")?,
            }
        }
        _ => {
            return Err(PyValueError::new_err("profile requires state"));
        }
    };
    let models = match dict.get_item("targets")? {
        Some(t) if !t.is_none() => extract_models(&t)?,
        _ => Vec::new(),
    };
    Ok(RouterProfile {
        algorithm,
        state,
        targets: TargetsConfig { models },
        evolving: Vec::new(),
    })
}

fn extract_models(obj: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        if let Some(m) = dict.get_item("models")? {
            return m.extract();
        }
        return Ok(Vec::new());
    }
    if obj.downcast::<PyList>().is_ok() {
        return obj.extract();
    }
    Err(PyValueError::new_err(
        "targets must be a list of models or dict{models}",
    ))
}

fn dict_str(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    match dict.get_item(key)? {
        Some(v) if !v.is_none() => Ok(Some(v.extract()?)),
        _ => Ok(None),
    }
}

fn opt_dict_str(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    dict_str(dict, key)
}

fn opt_dict_u64(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<u64>> {
    match dict.get_item(key)? {
        Some(v) if !v.is_none() => Ok(Some(v.extract()?)),
        _ => Ok(None),
    }
}

fn opt_dict_usize(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<usize>> {
    match dict.get_item(key)? {
        Some(v) if !v.is_none() => Ok(Some(v.extract()?)),
        _ => Ok(None),
    }
}

/// 类则 call0 实例化；实例则原样返回。
pub fn as_instance<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if obj.downcast::<PyType>().is_ok() {
        obj.call0()
    } else {
        Ok(obj.clone())
    }
}

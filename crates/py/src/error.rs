//! Rust [`RouterError`] → Python 异常。

use pyo3::exceptions::{PyLookupError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use openjiuwen_protocol::RouterError;

pub fn to_py(err: RouterError) -> PyErr {
    match err {
        RouterError::Config(msg) => PyValueError::new_err(format!("config: {msg}")),
        RouterError::Algorithm(msg) => PyRuntimeError::new_err(format!("algorithm: {msg}")),
        RouterError::State(msg) => PyRuntimeError::new_err(format!("state: {msg}")),
        RouterError::NoTarget => PyLookupError::new_err(err.to_string()),
    }
}

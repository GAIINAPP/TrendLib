#![forbid(unsafe_code)]

mod generated;
mod shared;

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Warm-up rows for an indicator, by name. The generated code holds the table.
#[pyfunction]
#[pyo3(signature = (name, **params))]
fn lookback(py: Python<'_>, name: &str, params: Option<&Bound<'_, PyDict>>) -> PyResult<usize> {
    generated::lookback_of(py, name, params)
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", trendlib::VERSION)?;
    m.add_function(wrap_pyfunction!(lookback, m)?)?;
    generated::register(m)
}

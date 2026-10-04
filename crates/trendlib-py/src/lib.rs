#![forbid(unsafe_code)]

use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", trendlib::VERSION)?;
    Ok(())
}

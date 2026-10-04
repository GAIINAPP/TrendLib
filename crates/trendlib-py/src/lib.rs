#![forbid(unsafe_code)]

mod handwritten;

use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", trendlib::VERSION)?;
    handwritten::register(m)
}

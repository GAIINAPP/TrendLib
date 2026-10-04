//! Helpers the generated bindings call. Hand-written and small on purpose: the
//! generator should emit calls, not logic.

use numpy::PyReadonlyArray1;
use pyo3::prelude::*;
use trendlib::TlError;
use trendlib::core::math::MaType;

/// Map a core error onto the Python class `docs/PYTHON_API.md` promises. The
/// module is imported on the error path only, so the happy path pays nothing
/// and no exception type is cached anywhere (D9).
pub fn to_py_err(py: Python<'_>, error: &TlError) -> PyErr {
    let (class, message) = match error {
        TlError::InvalidInput(message) => ("InvalidInput", message),
        TlError::InsufficientHistory(message) => ("InsufficientHistory", message),
    };
    match py
        .import("trendlib.errors")
        .and_then(|module| module.getattr(class))
        .and_then(|class| class.call1((message.as_str(),)))
    {
        Ok(instance) => PyErr::from_value(instance),
        Err(err) => err,
    }
}

/// Borrow an array as a slice. `_convert.py` hands over C-contiguous float64,
/// so the error only fires if something bypassed the Python layer.
pub fn as_slice<'a>(array: &'a PyReadonlyArray1<'a, f64>, name: &str) -> PyResult<&'a [f64]> {
    array.as_slice().map_err(|_| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "{name} must be a C-contiguous float64 array; pass it through trendlib's Python layer"
        ))
    })
}

/// Narrow a signed parameter from Python, reporting the value the caller
/// actually passed rather than whatever it clamped to.
pub fn int_param(
    indicator: &str,
    name: &str,
    value: i64,
    min: i64,
    max: i64,
) -> Result<usize, TlError> {
    if value < min || value > max {
        return Err(TlError::param_out_of_range(
            indicator, name, value, min, max,
        ));
    }
    usize::try_from(value)
        .map_err(|_| TlError::param_out_of_range(indicator, name, value, min, max))
}

// Called by generated code as soon as an indicator has a float parameter;
// `bbands` is the first (nbdev_up, nbdev_dn). Kept beside int_param so the two
// report an out-of-range value the same way.
#[allow(dead_code)]
pub fn float_param(
    indicator: &str,
    name: &str,
    value: f64,
    min: f64,
    max: f64,
) -> Result<f64, TlError> {
    if !value.is_finite() || value < min || value > max {
        return Err(TlError::param_out_of_range(
            indicator, name, value, min, max,
        ));
    }
    Ok(value)
}

/// Resolve a moving-average name, listing what is accepted when it is not one.
///
/// An average whose indicator has not shipped yet is rejected rather than
/// quietly standing in for another one (`docs/INDICATORS.md` section 1).
pub fn ma_type_param(indicator: &str, name: &str, value: &str) -> Result<MaType, TlError> {
    MaType::parse(indicator, name, value)
}

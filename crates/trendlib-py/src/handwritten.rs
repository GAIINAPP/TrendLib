//! Hand-written bindings for the M1 indicators. `cargo xtask generate` writes
//! this file as `generated.rs` in M2 and these go away.

use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use trendlib::TlError;
use trendlib::core::traits::{Indicator, Stream};
use trendlib::indicators::{ema, rsi, sma};

/// Map a core error onto the Python class that `docs/PYTHON_API.md` promises.
/// The module is imported on the error path only, so the happy path pays
/// nothing and no exception type is cached anywhere (D9).
fn to_py_err(py: Python<'_>, error: &TlError) -> PyErr {
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

/// Borrow the array as a slice. `_convert.py` hands over C-contiguous float64,
/// so the fallback only fires if something bypassed the Python layer.
fn as_slice<'a>(array: &'a PyReadonlyArray1<'a, f64>) -> PyResult<&'a [f64]> {
    array.as_slice().map_err(|_| {
        pyo3::exceptions::PyValueError::new_err(
            "input array must be C-contiguous float64; pass it through trendlib's Python layer",
        )
    })
}

/// Turn a signed period from Python into the unsigned one the core wants,
/// reporting the value the caller actually passed.
fn period_of(name: &str, period: i64, min: usize, max: usize) -> Result<usize, TlError> {
    usize::try_from(period)
        .ok()
        .filter(|value| (min..=max).contains(value))
        .ok_or_else(|| TlError::param_out_of_range(name, "period", period, min, max))
}

macro_rules! bind_single_series {
    (
        $binding:ident,
        $module:ident,
        $indicator:ident,
        $name:literal,
        $stream_class:literal
    ) => {
        pub mod $binding {
            use super::*;

            type Indicated = $module::$indicator;
            type Inner = <Indicated as Indicator>::Stream;

            pub const NAME: &str = $name;
            pub const DEFAULT: usize = $module::PERIOD_DEFAULT;
            pub const MIN: usize = $module::PERIOD_MIN;
            pub const MAX: usize = $module::PERIOD_MAX;

            fn params(period: i64) -> Result<$module::Params, TlError> {
                Ok($module::Params {
                    period: period_of(NAME, period, MIN, MAX)?,
                })
            }

            pub fn lookback(period: i64) -> Result<usize, TlError> {
                Ok(<Indicated as Indicator>::lookback(&params(period)?))
            }

            #[pyfunction]
            #[pyo3(name = $name, signature = (source, *, period))]
            pub fn batch<'py>(
                py: Python<'py>,
                source: PyReadonlyArray1<'py, f64>,
                period: i64,
            ) -> PyResult<Bound<'py, PyArray1<f64>>> {
                let values = as_slice(&source)?;
                let params = params(period).map_err(|e| to_py_err(py, &e))?;
                let out = py
                    .detach(|| <Indicated as Indicator>::batch(values, &params))
                    .map_err(|e| to_py_err(py, &e))?;
                Ok(out.into_pyarray(py))
            }

            #[pyclass(module = "trendlib._core", name = $stream_class, skip_from_py_object)]
            #[derive(Clone, Debug)]
            pub struct PyStream {
                inner: Inner,
            }

            #[pymethods]
            impl PyStream {
                #[staticmethod]
                #[pyo3(signature = (history, *, period))]
                fn open<'py>(
                    py: Python<'py>,
                    history: PyReadonlyArray1<'py, f64>,
                    period: i64,
                ) -> PyResult<Self> {
                    let values = as_slice(&history)?;
                    let params = params(period).map_err(|e| to_py_err(py, &e))?;
                    let inner = py
                        .detach(|| <Inner as Stream>::open(values, &params))
                        .map_err(|e| to_py_err(py, &e))?;
                    Ok(Self { inner })
                }

                #[staticmethod]
                #[pyo3(signature = (history, *, period))]
                fn open_and_fill<'py>(
                    py: Python<'py>,
                    history: PyReadonlyArray1<'py, f64>,
                    period: i64,
                ) -> PyResult<(Self, Bound<'py, PyArray1<f64>>)> {
                    let values = as_slice(&history)?;
                    let params = params(period).map_err(|e| to_py_err(py, &e))?;
                    let (inner, out) = py
                        .detach(|| <Indicated as Indicator>::open_and_fill(values, &params))
                        .map_err(|e| to_py_err(py, &e))?;
                    Ok((Self { inner }, out.into_pyarray(py)))
                }

                fn update(&mut self, py: Python<'_>, bar: f64) -> PyResult<f64> {
                    self.inner.update(bar).map_err(|e| to_py_err(py, &e))
                }

                fn peek(&self, py: Python<'_>, bar: f64) -> PyResult<f64> {
                    self.inner.peek(bar).map_err(|e| to_py_err(py, &e))
                }

                fn copy(&self) -> Self {
                    self.clone()
                }

                #[getter]
                fn value(&self) -> Option<f64> {
                    self.inner.value()
                }

                #[getter]
                fn bars_seen(&self) -> u64 {
                    self.inner.bars_seen()
                }

                #[getter]
                fn name(&self) -> &'static str {
                    NAME
                }

                fn __repr__(&self) -> String {
                    match self.inner.value() {
                        Some(value) => format!(
                            "<trendlib stream {NAME} bars_seen={} value={value}>",
                            self.inner.bars_seen()
                        ),
                        None => format!(
                            "<trendlib stream {NAME} bars_seen={} value=None>",
                            self.inner.bars_seen()
                        ),
                    }
                }
            }
        }
    };
}

bind_single_series!(sma_binding, sma, Sma, "sma", "SmaStream");
bind_single_series!(ema_binding, ema, Ema, "ema", "EmaStream");
bind_single_series!(rsi_binding, rsi, Rsi, "rsi", "RsiStream");

#[pyfunction]
#[pyo3(signature = (name, *, period=None))]
fn lookback(py: Python<'_>, name: &str, period: Option<i64>) -> PyResult<usize> {
    let result = match name {
        sma_binding::NAME => sma_binding::lookback(period.unwrap_or(sma_binding::DEFAULT as i64)),
        ema_binding::NAME => ema_binding::lookback(period.unwrap_or(ema_binding::DEFAULT as i64)),
        rsi_binding::NAME => rsi_binding::lookback(period.unwrap_or(rsi_binding::DEFAULT as i64)),
        other => {
            return Err(to_py_err(
                py,
                &TlError::InvalidInput(format!("lookback: no indicator named {other:?}")),
            ));
        }
    };
    result.map_err(|e| to_py_err(py, &e))
}

/// Parameter metadata, so the Python layer never repeats a default or a range.
/// The registry replaces this in M2.
fn params_table<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    let table = PyDict::new(py);
    for (name, default, min, max) in [
        (
            sma_binding::NAME,
            sma_binding::DEFAULT,
            sma_binding::MIN,
            sma_binding::MAX,
        ),
        (
            ema_binding::NAME,
            ema_binding::DEFAULT,
            ema_binding::MIN,
            ema_binding::MAX,
        ),
        (
            rsi_binding::NAME,
            rsi_binding::DEFAULT,
            rsi_binding::MIN,
            rsi_binding::MAX,
        ),
    ] {
        let period = PyDict::new(py);
        period.set_item("default", default)?;
        period.set_item("min", min)?;
        period.set_item("max", max)?;
        let params = PyDict::new(py);
        params.set_item("period", period)?;
        table.set_item(name, params)?;
    }
    Ok(table)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sma_binding::batch, m)?)?;
    m.add_function(wrap_pyfunction!(ema_binding::batch, m)?)?;
    m.add_function(wrap_pyfunction!(rsi_binding::batch, m)?)?;
    m.add_class::<sma_binding::PyStream>()?;
    m.add_class::<ema_binding::PyStream>()?;
    m.add_class::<rsi_binding::PyStream>()?;
    m.add_function(wrap_pyfunction!(lookback, m)?)?;
    m.add("PARAMS", params_table(m.py())?)?;
    Ok(())
}

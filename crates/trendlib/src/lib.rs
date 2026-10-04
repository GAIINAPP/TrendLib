#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub mod core;
pub mod indicators;

pub use crate::core::error::TlError;
pub use crate::core::traits::{Indicator, SeriesStep, Stream};

/// The version of this crate, shared across the whole workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Hand-written for M1. From M2 this section is written by
// `cargo xtask generate` from every indicator's spec.yaml.

/// Simple moving average of `source`.
pub fn sma(source: &[f64], params: &indicators::sma::Params) -> Result<Vec<f64>, TlError> {
    indicators::sma::Sma::batch(source, params)
}

/// Exponential moving average of `source`.
pub fn ema(source: &[f64], params: &indicators::ema::Params) -> Result<Vec<f64>, TlError> {
    indicators::ema::Ema::batch(source, params)
}

/// Relative strength index of `source`.
pub fn rsi(source: &[f64], params: &indicators::rsi::Params) -> Result<Vec<f64>, TlError> {
    indicators::rsi::Rsi::batch(source, params)
}

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_three_numeric_components() {
        let core: &str = VERSION.split(['-', '+']).next().unwrap();
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "version {VERSION} is not major.minor.patch");
        assert!(parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())));
    }
}

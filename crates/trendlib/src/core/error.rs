use std::fmt;

/// Every error the core crate produces.
///
/// The two variants map one to one onto the Python exceptions in
/// `docs/PYTHON_API.md` section 1, and the message is the whole payload: it
/// names the parameter or input, the offending value and the allowed range, so
/// a caller never has to inspect the variant to write a useful log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlError {
    /// A parameter is out of range, or an input violates the frame contract.
    /// Maps to `trendlib.InvalidInput`.
    InvalidInput(String),
    /// A stream was opened with fewer valid bars than its lookback needs.
    /// Maps to `trendlib.InsufficientHistory`.
    InsufficientHistory(String),
}

impl TlError {
    pub fn param_out_of_range(
        indicator: &str,
        param: &str,
        value: impl fmt::Display,
        min: impl fmt::Display,
        max: impl fmt::Display,
    ) -> Self {
        Self::InvalidInput(format!(
            "{indicator}: {param}={value} is out of range [{min}, {max}]"
        ))
    }

    pub fn unequal_lengths(
        indicator: &str,
        first_input: &str,
        first_len: usize,
        input: &str,
        len: usize,
    ) -> Self {
        Self::InvalidInput(format!(
            "{indicator}: inputs must have equal length; {first_input} has {first_len} rows, \
             {input} has {len}"
        ))
    }

    pub fn non_finite(indicator: &str, input: &str, row: usize, value: f64) -> Self {
        Self::InvalidInput(format!(
            "{indicator}: {input} is {value} at row {row}; every input must be finite after the \
             first valid bar"
        ))
    }

    pub fn insufficient_history(indicator: &str, needed: usize, got: usize) -> Self {
        Self::InsufficientHistory(format!(
            "{indicator}: opening a stream needs at least {needed} valid bars, got {got}"
        ))
    }

    pub fn message(&self) -> &str {
        match self {
            Self::InvalidInput(message) | Self::InsufficientHistory(message) => message,
        }
    }
}

impl fmt::Display for TlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for TlError {}

#[cfg(test)]
mod tests {
    use super::TlError;

    #[test]
    fn param_message_matches_the_python_api_contract() {
        let error = TlError::param_out_of_range("rsi", "period", 1, 2, 100_000);
        assert_eq!(
            error.to_string(),
            "rsi: period=1 is out of range [2, 100000]"
        );
        assert!(matches!(error, TlError::InvalidInput(_)));
    }

    #[test]
    fn history_message_states_how_many_bars_are_needed() {
        let error = TlError::insufficient_history("ema", 30, 12);
        assert_eq!(
            error.to_string(),
            "ema: opening a stream needs at least 30 valid bars, got 12"
        );
    }

    #[test]
    fn non_finite_message_names_the_input_and_the_row() {
        let error = TlError::non_finite("sma", "source", 7, f64::NAN);
        assert!(
            error
                .to_string()
                .starts_with("sma: source is NaN at row 7;")
        );
    }
}

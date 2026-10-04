use crate::core::error::TlError;

/// One named series handed to an indicator.
#[derive(Clone, Copy, Debug)]
pub struct SeriesInput<'a> {
    pub name: &'static str,
    pub values: &'a [f64],
}

impl<'a> SeriesInput<'a> {
    pub fn new(name: &'static str, values: &'a [f64]) -> Self {
        Self { name, values }
    }
}

/// What validation learned about a set of equally long inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prepared {
    pub len: usize,
    /// Index of the first bar where every input is finite, or `len` when there
    /// is none. Rows before it are warm-up rows (`docs/CONVENTIONS.md` § 3).
    pub first_valid: usize,
}

impl Prepared {
    pub fn valid_len(&self) -> usize {
        self.len - self.first_valid
    }
}

/// Check the frame contract and locate the first valid bar.
///
/// Leading non-finite rows are warm-up and are skipped. A NaN or infinity after
/// the first valid bar is an error naming the input and the row (D8), because a
/// loud error beats a plausible wrong number.
pub fn prepare(indicator: &str, inputs: &[SeriesInput<'_>]) -> Result<Prepared, TlError> {
    let Some(first) = inputs.first() else {
        return Ok(Prepared {
            len: 0,
            first_valid: 0,
        });
    };
    let len = first.values.len();
    for input in &inputs[1..] {
        if input.values.len() != len {
            return Err(TlError::unequal_lengths(
                indicator,
                first.name,
                len,
                input.name,
                input.values.len(),
            ));
        }
    }

    let row_is_valid = |row: usize| inputs.iter().all(|i| i.values[row].is_finite());
    let first_valid = (0..len).find(|&row| row_is_valid(row)).unwrap_or(len);

    for row in first_valid + 1..len {
        for input in inputs {
            let value = input.values[row];
            if !value.is_finite() {
                return Err(TlError::non_finite(indicator, input.name, row, value));
            }
        }
    }

    Ok(Prepared { len, first_valid })
}

/// Reject a negative volume, which is never a valid bar.
pub fn check_volume(indicator: &str, volume: &[f64], from: usize) -> Result<(), TlError> {
    for (row, &value) in volume.iter().enumerate().skip(from) {
        if value < 0.0 {
            return Err(TlError::InvalidInput(format!(
                "{indicator}: volume is {value} at row {row}; volume may be zero but not negative"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Prepared, SeriesInput, prepare};
    use crate::core::error::TlError;

    fn one(values: &[f64]) -> Result<Prepared, TlError> {
        prepare("test", &[SeriesInput::new("source", values)])
    }

    #[test]
    fn empty_input_is_not_an_error() {
        assert_eq!(
            one(&[]).unwrap(),
            Prepared {
                len: 0,
                first_valid: 0
            }
        );
    }

    #[test]
    fn leading_non_finite_rows_are_skipped() {
        let prepared = one(&[f64::NAN, f64::NEG_INFINITY, 1.0, 2.0]).unwrap();
        assert_eq!(prepared.first_valid, 2);
        assert_eq!(prepared.valid_len(), 2);
    }

    #[test]
    fn an_all_invalid_series_has_no_valid_bar() {
        let prepared = one(&[f64::NAN, f64::NAN]).unwrap();
        assert_eq!(prepared.first_valid, 2);
        assert_eq!(prepared.valid_len(), 0);
    }

    #[test]
    fn non_finite_after_the_first_valid_bar_is_rejected() {
        let error = one(&[1.0, 2.0, f64::INFINITY]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "test: source is inf at row 2; every input must be finite after the first valid bar"
        );
    }

    #[test]
    fn the_first_valid_bar_needs_every_input_finite() {
        let a = [f64::NAN, 1.0, 2.0];
        let b = [0.0, f64::NAN, 3.0];
        let prepared = prepare(
            "test",
            &[SeriesInput::new("high", &a), SeriesInput::new("low", &b)],
        )
        .unwrap();
        assert_eq!(prepared.first_valid, 2);
    }

    #[test]
    fn unequal_lengths_are_rejected() {
        let a = [1.0, 2.0];
        let b = [1.0];
        let error = prepare(
            "test",
            &[SeriesInput::new("high", &a), SeriesInput::new("low", &b)],
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "test: inputs must have equal length; high has 2 rows, low has 1"
        );
    }
}

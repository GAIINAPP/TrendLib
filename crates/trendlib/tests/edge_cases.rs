//! The shared edge-case suite from `docs/TESTING.md` section 5. Hand-written
//! for M1; generated for every indicator in M2.

mod support;

use support::{bitwise_equal, daily_closes, registered};
use trendlib::TlError;

#[test]
fn empty_input_returns_empty_output() {
    for indicator in registered() {
        let out = (indicator.batch)(&[], indicator.default_period).unwrap();
        assert!(out.is_empty(), "{}", indicator.name);
    }
}

#[test]
fn short_input_is_all_warm_up_until_one_bar_past_the_lookback() {
    let closes = daily_closes();
    for indicator in registered() {
        let period = indicator.default_period;
        let lookback = (indicator.lookback)(period);

        for len in [lookback.saturating_sub(1), lookback] {
            let out = (indicator.batch)(&closes[..len], period).unwrap();
            assert_eq!(out.len(), len, "{}", indicator.name);
            assert!(
                out.iter().all(|v| v.is_nan()),
                "{} produced a value with only {len} bars",
                indicator.name
            );
        }

        let out = (indicator.batch)(&closes[..lookback + 1], period).unwrap();
        assert_eq!(out.len(), lookback + 1);
        assert!(
            out[..lookback].iter().all(|v| v.is_nan()),
            "{}",
            indicator.name
        );
        assert!(
            out[lookback].is_finite(),
            "{} has no value at row {lookback}",
            indicator.name
        );
    }
}

#[test]
fn a_constant_series_is_defined_everywhere() {
    let flat = vec![5.0; 200];
    for indicator in registered() {
        let period = indicator.default_period;
        let lookback = (indicator.lookback)(period);
        let out = (indicator.batch)(&flat, period).unwrap();
        // A flat series has no movement at all, so the moving averages return
        // the level and RSI reports 0, which is what TA-Lib returns too (the
        // Python parity suite checks that against the oracle directly).
        let expected = if indicator.name == "rsi" { 0.0 } else { 5.0 };
        for (row, value) in out.iter().enumerate().skip(lookback) {
            assert_eq!(*value, expected, "{} at row {row}", indicator.name);
        }
    }
}

#[test]
fn leading_warm_up_rows_are_skipped() {
    let closes = daily_closes();
    for indicator in registered() {
        let period = indicator.default_period;
        let trimmed = (indicator.batch)(&closes, period).unwrap();
        for leading in [1usize, 5, 37] {
            let mut padded = vec![f64::NAN; leading];
            padded.extend_from_slice(&closes);
            let shifted = (indicator.batch)(&padded, period).unwrap();
            assert!(shifted[..leading].iter().all(|v| v.is_nan()));
            assert!(
                bitwise_equal(&shifted[leading..], &trimmed),
                "{} with {leading} leading NaN rows",
                indicator.name
            );
        }
    }
}

#[test]
fn a_non_finite_bar_after_the_first_valid_bar_is_rejected() {
    let closes = daily_closes();
    for indicator in registered() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut series = closes[..100].to_vec();
            series[42] = bad;
            let error = (indicator.batch)(&series, indicator.default_period).unwrap_err();
            let message = error.to_string();
            assert!(
                matches!(error, TlError::InvalidInput(_)),
                "{}",
                indicator.name
            );
            assert!(
                message.contains("source") && message.contains("row 42"),
                "{}: message does not name the input and row: {message}",
                indicator.name
            );
        }
    }
}

#[test]
fn extreme_magnitudes_do_not_panic() {
    for scale in [1.0e-300f64, 1.0e300] {
        let series: Vec<f64> = (0..120)
            .map(|i| scale * (1.0 + (i % 7) as f64 / 10.0))
            .collect();
        for indicator in registered() {
            let out = (indicator.batch)(&series, indicator.default_period).unwrap();
            assert_eq!(out.len(), series.len(), "{}", indicator.name);
            let lookback = (indicator.lookback)(indicator.default_period);
            assert!(
                out[lookback..].iter().all(|v| v.is_finite()),
                "{} produced a non-finite value at scale {scale}",
                indicator.name
            );
        }
    }
}

#[test]
fn parameters_at_the_edges_of_their_range_are_accepted() {
    let closes = daily_closes();
    for indicator in registered() {
        for period in [indicator.min_period, indicator.max_period] {
            let out = (indicator.batch)(&closes, period)
                .unwrap_or_else(|e| panic!("{} period={period}: {e}", indicator.name));
            assert_eq!(out.len(), closes.len());
        }
    }
}

#[test]
fn parameters_outside_their_range_name_the_range() {
    let closes = daily_closes();
    for indicator in registered() {
        for period in [indicator.min_period - 1, indicator.max_period + 1] {
            let error = (indicator.batch)(&closes, period).unwrap_err();
            let message = error.to_string();
            assert!(
                matches!(error, TlError::InvalidInput(_)),
                "{}",
                indicator.name
            );
            let expected = format!(
                "{}: period={period} is out of range [{}, {}]",
                indicator.name, indicator.min_period, indicator.max_period
            );
            assert_eq!(message, expected, "{}", indicator.name);
        }
    }
}

#[test]
fn a_stream_opened_with_exactly_the_lookback_is_refused() {
    let closes = daily_closes();
    for indicator in registered() {
        let period = indicator.default_period;
        let lookback = (indicator.lookback)(period);

        let error = (indicator.open)(&closes[..lookback], period).unwrap_err();
        let message = error.to_string();
        assert!(
            matches!(error, TlError::InsufficientHistory(_)),
            "{}",
            indicator.name
        );
        assert!(
            message.contains(&format!("at least {} valid bars", lookback + 1)),
            "{}: {message}",
            indicator.name
        );

        (indicator.open)(&closes[..lookback + 1], period)
            .unwrap_or_else(|e| panic!("{} should open on lookback + 1 bars: {e}", indicator.name));
    }
}

#[test]
fn a_rejected_bar_leaves_the_stream_untouched() {
    let closes = daily_closes();
    for indicator in registered() {
        let period = indicator.default_period;
        let split = (indicator.lookback)(period) + 1;

        let mut clean = (indicator.open)(&closes[..split], period).unwrap();
        let mut poisoned = (indicator.open)(&closes[..split], period).unwrap();

        for bad in [f64::NAN, f64::INFINITY] {
            let error = poisoned.update(bad).unwrap_err();
            assert!(
                matches!(error, TlError::InvalidInput(_)),
                "{}",
                indicator.name
            );
            assert!(poisoned.peek(bad).is_err(), "{}", indicator.name);
        }
        assert_eq!(
            poisoned.bars_seen(),
            clean.bars_seen(),
            "{}",
            indicator.name
        );

        for &bar in &closes[split..split + 20] {
            let expected = clean.update(bar).unwrap();
            let got = poisoned.update(bar).unwrap();
            assert_eq!(
                expected.to_bits(),
                got.to_bits(),
                "{}: a rejected bar changed later values",
                indicator.name
            );
        }
    }
}

//! The shared edge-case suite from `docs/TESTING.md` section 5, run against
//! every registered indicator whatever its shape. Generated for every indicator
//! in M2.

mod support;

use support::{Registered, as_slices, bars_from, bitwise_equal, daily_inputs, head, registered};
use trendlib::TlError;

fn empty_like(indicator: &Registered) -> Vec<Vec<f64>> {
    indicator.inputs.iter().map(|_| Vec::new()).collect()
}

#[test]
fn empty_input_returns_empty_output() {
    for indicator in registered() {
        let columns = empty_like(&indicator);
        let out = (indicator.batch)(&as_slices(&columns), &indicator.defaults()).unwrap();
        assert_eq!(out.len(), indicator.outputs.len(), "{}", indicator.name);
        assert!(out.iter().all(Vec::is_empty), "{}", indicator.name);
    }
}

#[test]
fn short_input_is_all_warm_up_until_one_bar_past_the_lookback() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        let lookback = (indicator.lookback)(&indicator.defaults());

        for len in [lookback.saturating_sub(1), lookback] {
            let short = head(&columns, len);
            let out = (indicator.batch)(&as_slices(&short), &indicator.defaults()).unwrap();
            for column in &out {
                assert_eq!(column.len(), len, "{}", indicator.name);
                assert!(
                    column.iter().all(|v| v.is_nan()),
                    "{} produced a value with only {len} bars",
                    indicator.name
                );
            }
        }

        let exact = head(&columns, lookback + 1);
        let out = (indicator.batch)(&as_slices(&exact), &indicator.defaults()).unwrap();
        for (index, column) in out.iter().enumerate() {
            assert!(
                column[..lookback].iter().all(|v| v.is_nan()),
                "{}",
                indicator.name
            );
            assert!(
                !column[lookback].is_nan() || indicator.may_be_non_finite,
                "{} has no {} at row {lookback}",
                indicator.name,
                indicator.outputs[index]
            );
        }
    }
}

#[test]
fn a_constant_series_gives_a_constant_result() {
    for indicator in registered() {
        // A running total keeps climbing on a flat series, which is the whole
        // point of it, so the law applies to the rest.
        if indicator.path_dependent {
            continue;
        }
        let base = vec![5.0; 200];
        let columns = bars_from(&base, indicator.inputs);
        let lookback = (indicator.lookback)(&indicator.defaults());
        let out = (indicator.batch)(&as_slices(&columns), &indicator.defaults()).unwrap();
        for (index, column) in out.iter().enumerate() {
            let settled = column[lookback];
            if !settled.is_finite() {
                assert!(
                    indicator.may_be_non_finite,
                    "{} {} is not finite on a constant series",
                    indicator.name, indicator.outputs[index]
                );
                continue;
            }
            for (row, value) in column.iter().enumerate().skip(lookback) {
                assert_eq!(
                    *value, settled,
                    "{} {} drifted at row {row} on constant input",
                    indicator.name, indicator.outputs[index]
                );
            }
        }
        // RSI's documented answer on a flat series, which TA-Lib also returns.
        if indicator.name == "rsi" {
            assert_eq!(out[0][lookback], 0.0);
        }
    }
}

#[test]
fn leading_warm_up_rows_are_skipped() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        let trimmed = (indicator.batch)(&as_slices(&columns), &indicator.defaults()).unwrap();
        for leading in [1usize, 5, 37] {
            let padded: Vec<Vec<f64>> = columns
                .iter()
                .map(|column| {
                    let mut padded = vec![f64::NAN; leading];
                    padded.extend_from_slice(column);
                    padded
                })
                .collect();
            let shifted = (indicator.batch)(&as_slices(&padded), &indicator.defaults()).unwrap();
            for (index, column) in shifted.iter().enumerate() {
                assert!(column[..leading].iter().all(|v| v.is_nan()));
                assert!(
                    bitwise_equal(&column[leading..], &trimmed[index]),
                    "{} with {leading} leading NaN rows",
                    indicator.name
                );
            }
        }
    }
}

#[test]
fn a_non_finite_bar_after_the_first_valid_bar_is_rejected() {
    for indicator in registered() {
        let columns = head(&daily_inputs(&indicator), 100);
        for (which, name) in indicator.inputs.iter().enumerate() {
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut poisoned = columns.clone();
                poisoned[which][42] = bad;
                let error =
                    (indicator.batch)(&as_slices(&poisoned), &indicator.defaults()).unwrap_err();
                let message = error.to_string();
                assert!(
                    matches!(error, TlError::InvalidInput(_)),
                    "{}",
                    indicator.name
                );
                assert!(
                    message.contains(name) && message.contains("row 42"),
                    "{}: message does not name the input and row: {message}",
                    indicator.name
                );
            }
        }
    }
}

#[test]
fn extreme_magnitudes_do_not_panic() {
    for scale in [1.0e-300f64, 1.0e300] {
        for indicator in registered() {
            let base: Vec<f64> = (0..120)
                .map(|i| scale * (1.0 + (i % 7) as f64 / 10.0))
                .collect();
            let columns = bars_from(&base, indicator.inputs);
            let out = (indicator.batch)(&as_slices(&columns), &indicator.defaults()).unwrap();
            let lookback = (indicator.lookback)(&indicator.defaults());
            for (index, column) in out.iter().enumerate() {
                assert_eq!(column.len(), base.len(), "{}", indicator.name);
                // An indicator flagged nan_inf_output may legitimately answer
                // infinity or NaN here; what matters is that it answered at all
                // rather than panicking or dropping rows.
                assert!(
                    indicator.may_be_non_finite || column[lookback..].iter().all(|v| v.is_finite()),
                    "{} {} produced a non-finite value at scale {scale}",
                    indicator.name,
                    indicator.outputs[index]
                );
            }
        }
    }
}

#[test]
fn parameters_at_the_edges_of_their_range_are_accepted() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        for param in indicator.params {
            for edge in [param.min, param.max.min(100_000.0)] {
                let values = indicator.with(param.name, edge);
                let out = (indicator.batch)(&as_slices(&columns), &values)
                    .unwrap_or_else(|e| panic!("{} {}={edge}: {e}", indicator.name, param.name));
                assert_eq!(out[0].len(), columns[0].len());
            }
        }
    }
}

#[test]
fn parameters_outside_their_range_name_the_range() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        for param in indicator.params {
            if !param.integral {
                continue;
            }
            for bad in [param.min - 1.0, param.max + 1.0] {
                let values = indicator.with(param.name, bad);
                let error = (indicator.batch)(&as_slices(&columns), &values).unwrap_err();
                assert_eq!(
                    error.to_string(),
                    format!(
                        "{}: {}={} is out of range [{}, {}]",
                        indicator.name, param.name, bad as i64, param.min as i64, param.max as i64
                    ),
                    "{} {}",
                    indicator.name,
                    param.name
                );
            }
        }
    }
}

#[test]
fn a_stream_opened_with_exactly_the_lookback_is_refused() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        let lookback = (indicator.lookback)(&indicator.defaults());

        let too_short = head(&columns, lookback);
        let error = indicator
            .open(&as_slices(&too_short), &indicator.defaults())
            .unwrap_err();
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

        let enough = head(&columns, lookback + 1);
        indicator
            .open(&as_slices(&enough), &indicator.defaults())
            .unwrap_or_else(|e| panic!("{} should open on lookback + 1 bars: {e}", indicator.name));
    }
}

#[test]
fn a_rejected_bar_leaves_the_stream_untouched() {
    for indicator in registered() {
        let columns = daily_inputs(&indicator);
        let split = (indicator.lookback)(&indicator.defaults()) + 1;
        let history = head(&columns, split);

        let mut clean = indicator
            .open(&as_slices(&history), &indicator.defaults())
            .unwrap();
        let mut poisoned = indicator
            .open(&as_slices(&history), &indicator.defaults())
            .unwrap();

        for bad in [f64::NAN, f64::INFINITY] {
            let bar: Vec<f64> = indicator.inputs.iter().map(|_| bad).collect();
            let error = poisoned.update(&bar).unwrap_err();
            assert!(
                matches!(error, TlError::InvalidInput(_)),
                "{}",
                indicator.name
            );
            assert!(poisoned.peek(&bar).is_err(), "{}", indicator.name);
        }
        assert_eq!(
            poisoned.bars_seen(),
            clean.bars_seen(),
            "{}",
            indicator.name
        );

        for row in split..split + 20 {
            let bar: Vec<f64> = columns.iter().map(|c| c[row]).collect();
            let expected = clean.update(&bar).unwrap();
            let got = poisoned.update(&bar).unwrap();
            assert!(
                bitwise_equal(&expected, &got),
                "{}: a rejected bar changed later values",
                indicator.name
            );
        }
    }
}

/// Deviation 6: `natr` normalises at every period, including the one where
/// TA-Lib does not, so the oracle cannot pin this row and a test must.
#[test]
fn natr_normalises_even_at_period_one() {
    let natr = support::find("natr").expect("natr is registered");
    let trange = support::find("trange").expect("trange is registered");
    let columns = head(&daily_inputs(&natr), 50);
    let inputs = as_slices(&columns);

    let normalised = (natr.batch)(&inputs, &natr.with("period", 1.0)).unwrap();
    let spans = (trange.batch)(&inputs, &trange.defaults()).unwrap();
    let close = &columns[2];

    for row in 1..columns[0].len() {
        let expected = 100.0 * spans[0][row] / close[row];
        assert_eq!(
            normalised[0][row].to_bits(),
            expected.to_bits(),
            "natr(period=1) at row {row} is not 100 * trange / close"
        );
    }
}

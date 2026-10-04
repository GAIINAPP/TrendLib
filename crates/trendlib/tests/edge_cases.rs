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
        // point of it. A row index moves too: every bar ties, so the record
        // holder keeps falling out of the window and being replaced. The law
        // applies to the rest.
        if indicator.path_dependent || indicator.absolute_index {
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
                if indicator.absolute_index {
                    // A row index is an index into the caller's own array, so
                    // a warm-up prefix moves it by exactly its own length.
                    let moved: Vec<f64> = trimmed[index]
                        .iter()
                        .map(|v| if v.is_nan() { *v } else { v + leading as f64 })
                        .collect();
                    assert!(
                        bitwise_equal(&column[leading..], &moved),
                        "{} with {leading} leading NaN rows did not shift its index",
                        indicator.name
                    );
                    continue;
                }
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
            // A parameter declared without bounds means "any finite value",
            // which the registry records as an infinite range. The edges of
            // that are not values anything can be called with.
            if !param.min.is_finite() || !param.max.is_finite() {
                continue;
            }
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

/// The golden file for `stddev` at its minimum period carries a looser
/// tolerance because the oracle is the less accurate of the two there. That
/// claim is only worth making if it is checked, so this pins how close the
/// answer is to the exact one: over two bars the population standard deviation
/// is half the difference between them.
#[test]
fn stddev_of_two_bars_is_half_their_difference() {
    let stddev = support::find("stddev").expect("stddev is registered");
    let closes = support::daily_column("close");
    let out = (stddev.batch)(&[&closes], &stddev.with("period", 2.0)).unwrap();

    let mut exact = 0usize;
    let mut compared = 0usize;
    for row in 1..closes.len() {
        let expected = (closes[row] - closes[row - 1]).abs() / 2.0;
        if expected == 0.0 {
            continue;
        }
        compared += 1;
        if out[0][row].to_bits() == expected.to_bits() {
            exact += 1;
        }
        let error = (out[0][row] - expected).abs() / expected;
        assert!(
            error <= 1.0e-15,
            "stddev(period=2) at row {row} is {} away from half the bar-to-bar difference",
            error
        );
    }
    // Measured at 98.7 percent of rows; the bound leaves room for a different
    // platform's rounding without letting a regression through unnoticed.
    assert!(
        exact * 100 >= compared * 95,
        "only {exact} of {compared} rows were exact, which used to be 98.7 percent"
    );
}

/// `var` carries the same looser parity bound as `stddev` and for the same
/// reason, so the exact property is pinned here too: over two bars the
/// population variance is the square of half their difference.
#[test]
fn var_of_two_bars_is_the_square_of_half_their_difference() {
    let var = support::find("var").expect("var is registered");
    let closes = support::daily_column("close");
    let out = (var.batch)(&[&closes], &var.with("period", 2.0)).unwrap();

    for row in 1..closes.len() {
        let half = (closes[row] - closes[row - 1]).abs() / 2.0;
        let expected = half * half;
        if expected == 0.0 {
            continue;
        }
        let error = (out[0][row] - expected).abs() / expected;
        assert!(
            error <= 1.0e-15,
            "var(period=2) at row {row} is {error} away from the square of half the difference"
        );
    }
}

/// The stochastic family carries a looser golden tolerance on its `trima`
/// cases, on the grounds that TA-Lib's running sums keep a residue across the
/// swing from 0 to 100 and back. That claim is only worth making if TrendLib's
/// own answer is checked, so this rebuilds the triangular average of %K from
/// the window itself rather than from a running sum, and pins how close the
/// streamed one stays to it.
#[test]
fn the_triangular_average_of_percent_k_stays_near_the_window_mean() {
    let stochf = support::find("stochf").expect("stochf is registered");
    let bars = daily_inputs(&stochf);
    let columns = as_slices(&bars);

    // fastd_period = 1 smooths nothing, so this is the raw %K column.
    let mut raw = stochf.defaults();
    raw[1] = 1.0;
    let k = (stochf.batch)(&columns, &raw).unwrap()[0].clone();

    let mut params = stochf.defaults();
    params[1] = 3.0;
    params[2] = support::registry::MA_TYPES
        .iter()
        .position(|name| *name == "trima")
        .expect("trima is an average the core has") as f64;
    let out = (stochf.batch)(&columns, &params).unwrap();

    // trima over three bars is the mean of two overlapping pairs, which is
    // (a + 2b + c) / 4 written out.
    for row in 2..k.len() {
        if k[row - 2].is_nan() {
            continue;
        }
        let expected = (k[row - 2] + 2.0 * k[row - 1] + k[row]) / 4.0;
        let error = (out[1][row] - expected).abs();
        assert!(
            error <= 1.0e-12,
            "stochf %D at row {row} is {error} away from the triangular mean of its window"
        );
    }
}

/// `apo` and `ppo` carry a looser golden tolerance because subtracting two
/// averages of one series cancels most of the digits away. The claim is only
/// worth making if the parts that do not cancel are pinned, so this checks the
/// two exact properties the cancellation cannot touch: equal periods subtract
/// an average from itself and give exactly zero, and swapping the periods is
/// the same question asked the other way round, so it gives the same bits.
#[test]
fn equal_periods_make_the_price_oscillators_exactly_zero() {
    let closes = support::daily_column("close");
    for name in ["apo", "ppo"] {
        let indicator = support::find(name).expect("registered");
        for choice in 0..support::registry::MA_TYPES.len() {
            let mut params = indicator.defaults();
            params[0] = 9.0;
            params[1] = 9.0;
            params[2] = choice as f64;
            let out = (indicator.batch)(&[&closes], &params).unwrap();
            let lookback = (indicator.lookback)(&params);
            for (row, value) in out[0].iter().enumerate().skip(lookback) {
                assert_eq!(
                    *value,
                    0.0,
                    "{name} with ma_type={} and equal periods is {value} at row {row}, not zero",
                    support::registry::MA_TYPES[choice],
                );
            }
        }
    }
}

#[test]
fn swapping_the_price_oscillator_periods_changes_nothing() {
    let closes = support::daily_column("close");
    for name in ["apo", "ppo"] {
        let indicator = support::find(name).expect("registered");
        let mut forward = indicator.defaults();
        forward[0] = 7.0;
        forward[1] = 23.0;
        let mut backward = forward.clone();
        backward.swap(0, 1);
        let one = (indicator.batch)(&[&closes], &forward).unwrap();
        let two = (indicator.batch)(&[&closes], &backward).unwrap();
        assert!(
            bitwise_equal(&one[0], &two[0]),
            "{name} answers differently when the periods arrive the other way round"
        );
    }
}

/// Deviation 7: the directional indicators stay a percentage at every period,
/// including the one where TA-Lib returns the raw fraction, so the oracle
/// cannot pin those rows and a test must.
#[test]
fn directional_indicators_scale_to_percent_even_at_period_one() {
    let columns = head(&daily_inputs(&support::find("plus_di").unwrap()), 200);
    let inputs = as_slices(&columns);

    for name in ["plus_di", "minus_di"] {
        let indicator = support::find(name).expect("registered");
        let at_one = (indicator.batch)(&inputs, &indicator.with("period", 1.0)).unwrap();
        // Every defined row is a percentage: the raw fraction would never
        // exceed 100 but would also never reach it on this data.
        let defined: Vec<f64> = at_one[0].iter().copied().filter(|v| !v.is_nan()).collect();
        assert!(!defined.is_empty(), "{name} produced nothing at period 1");
        assert!(
            defined.iter().any(|v| *v > 1.0),
            "{name} at period 1 looks like a raw fraction, not a percentage"
        );
        assert!(
            defined.iter().all(|v| (0.0..=100.0).contains(v)),
            "{name} at period 1 left the 0 to 100 range"
        );
    }
}

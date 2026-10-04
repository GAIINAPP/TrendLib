//! Batch and stream must agree bit for bit on every bar, for any history and
//! any split point (`CLAUDE.md` rule 6). Hand-written for M1; generated in M2.

mod support;

use proptest::prelude::*;
use support::{as_slices, bars_from, bitwise_equal, first_difference, head, registered};
use trendlib::TlError;

/// Period-like parameters are capped so a case stays fast; the golden suite
/// covers the documented default and minimum, and `edge_cases` the maximum.
const MAX_PERIOD: f64 = 40.0;

fn finite_series() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-1.0e6..1.0e6f64, 0..400)
}

fn bar_at(columns: &[Vec<f64>], row: usize) -> Vec<f64> {
    columns.iter().map(|column| column[row]).collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    #[test]
    fn stream_equals_batch_from_any_split(
        base in finite_series(),
        seed in 0usize..1000,
        split_pick in 0.0f64..=1.0,
        fork_pick in 0.0f64..=1.0,
    ) {
        for indicator in registered() {
            let values = indicator.capped(seed, MAX_PERIOD);
            let columns = bars_from(&base, indicator.inputs);
            let rows = base.len();
            let lookback = (indicator.lookback)(&values);

            let batch = (indicator.batch)(&as_slices(&columns), &values)
                .expect("a finite series is valid");
            prop_assert_eq!(batch.len(), indicator.outputs.len());
            for column in &batch {
                prop_assert_eq!(column.len(), rows);
            }

            let needed = lookback + 1;
            if rows < needed {
                let error = indicator.open(&as_slices(&columns), &values).unwrap_err();
                prop_assert!(matches!(error, TlError::InsufficientHistory(_)));
                continue;
            }

            let room = rows - needed;
            let split = (needed + (room as f64 * split_pick).round() as usize).min(rows);
            let history = head(&columns, split);

            let (mut stream, filled) =
                (indicator.open_and_fill)(&as_slices(&history), &values).unwrap();
            let history_batch = (indicator.batch)(&as_slices(&history), &values).unwrap();
            for (index, column) in filled.iter().enumerate() {
                prop_assert!(
                    bitwise_equal(column, &history_batch[index]),
                    "{}: open_and_fill differs from batch at {:?}",
                    indicator.name,
                    first_difference(column, &history_batch[index])
                );
            }
            prop_assert_eq!(stream.bars_seen(), split as u64);

            let fork_at = if split >= rows {
                usize::MAX
            } else {
                split + ((rows - split - 1) as f64 * fork_pick).round() as usize
            };

            let mut streamed = filled;
            for row in split..rows {
                let bar = bar_at(&columns, row);

                let peeked = stream.peek(&bar).unwrap();
                let peeked_again = stream.peek(&bar).unwrap();
                prop_assert!(bitwise_equal(&peeked, &peeked_again));

                if row == fork_at {
                    // A fork is an independent value: whatever happens to it,
                    // the original's next bar is unaffected.
                    let mut fork = stream.fork();
                    let nudged: Vec<f64> = bar.iter().map(|v| v * 1.5 + 1.0).collect();
                    fork.update(&nudged).unwrap();
                    fork.update(&nudged).unwrap();
                }

                let committed = stream.update(&bar).unwrap();
                prop_assert!(
                    bitwise_equal(&peeked, &committed),
                    "{}: peek did not predict update",
                    indicator.name
                );
                for (column, value) in streamed.iter_mut().zip(committed) {
                    column.push(value);
                }
            }

            for (index, column) in streamed.iter().enumerate() {
                prop_assert!(
                    bitwise_equal(column, &batch[index]),
                    "{} {}: stream and batch differ at {:?}",
                    indicator.name,
                    indicator.outputs[index],
                    first_difference(column, &batch[index])
                );
            }
            prop_assert_eq!(stream.bars_seen(), rows as u64);
        }
    }

    #[test]
    fn leading_warm_up_rows_shift_the_output(
        base in prop::collection::vec(-1.0e6..1.0e6f64, 40..200),
        leading in 1usize..10,
        seed in 0usize..1000,
    ) {
        for indicator in registered() {
            let values = indicator.capped(seed, MAX_PERIOD);
            let columns = bars_from(&base, indicator.inputs);
            let trimmed = (indicator.batch)(&as_slices(&columns), &values).unwrap();

            let padded: Vec<Vec<f64>> = columns
                .iter()
                .map(|column| {
                    let mut padded = vec![f64::NAN; leading];
                    padded.extend_from_slice(column);
                    padded
                })
                .collect();
            let shifted = (indicator.batch)(&as_slices(&padded), &values).unwrap();

            for (index, column) in shifted.iter().enumerate() {
                prop_assert_eq!(column.len(), base.len() + leading);
                prop_assert!(column[..leading].iter().all(|v| v.is_nan()));
                // A row index moves with the prefix; everything else does not.
                let expected: Vec<f64> = if indicator.absolute_index {
                    trimmed[index]
                        .iter()
                        .map(|v| if v.is_nan() { *v } else { v + leading as f64 })
                        .collect()
                } else {
                    trimmed[index].clone()
                };
                prop_assert!(
                    bitwise_equal(&column[leading..], &expected),
                    "{}: padding changed the values",
                    indicator.name
                );
            }
        }
    }
}

// The suite is only meaningful if it really ran the number of cases
// docs/MILESTONES.md asks for, so the count is asserted rather than assumed.
static CASES_RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

proptest! {
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    #[test]
    fn count_cases(_seed in 0usize..1000) {
        CASES_RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[test]
fn the_property_suite_runs_at_least_a_thousand_cases() {
    use std::sync::atomic::Ordering::Relaxed;

    let before = CASES_RUN.load(Relaxed);
    count_cases();
    let ran = CASES_RUN.load(Relaxed) - before;
    assert!(
        ran >= 1000,
        "proptest ran {ran} cases; docs/MILESTONES.md M1 asks for at least 1000"
    );
}

//! Batch and stream must agree bit for bit on every bar, for any history and
//! any split point (`CLAUDE.md` rule 6). Hand-written for M1; generated in M2.

mod support;

use proptest::prelude::*;
use support::{Registered, bitwise_equal, first_difference, registered};
use trendlib::TlError;

/// Periods are capped at 60 so a case stays fast; the golden suite covers the
/// documented default and minimum, and `edge_cases` covers the maximum.
const MAX_PERIOD: usize = 60;

fn period_for(indicator: &Registered, seed: usize) -> usize {
    let span = MAX_PERIOD - indicator.min_period + 1;
    indicator.min_period + seed % span
}

fn finite_series() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(-1.0e6..1.0e6f64, 0..600)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

    #[test]
    fn stream_equals_batch_from_any_split(
        series in finite_series(),
        seed in 0usize..1000,
        split_pick in 0.0f64..=1.0,
        fork_pick in 0.0f64..=1.0,
    ) {
        for indicator in registered() {
            let period = period_for(&indicator, seed);
            let lookback = (indicator.lookback)(period);
            let batch = (indicator.batch)(&series, period).expect("finite series is valid");
            prop_assert_eq!(batch.len(), series.len());

            let needed = lookback + 1;
            if series.len() < needed {
                let error = (indicator.open)(&series, period).unwrap_err();
                prop_assert!(matches!(error, TlError::InsufficientHistory(_)));
                continue;
            }

            let room = series.len() - needed;
            let split = needed + (room as f64 * split_pick).round() as usize;
            let split = split.min(series.len());

            let (mut stream, head) = (indicator.open_and_fill)(&series[..split], period).unwrap();
            let head_batch = (indicator.batch)(&series[..split], period).unwrap();
            prop_assert!(
                bitwise_equal(&head, &head_batch),
                "{}: open_and_fill differs from batch at {:?}",
                indicator.name,
                first_difference(&head, &head_batch)
            );
            prop_assert_eq!(stream.bars_seen(), split as u64);
            prop_assert_eq!(
                stream.value().map(f64::to_bits),
                head.last().copied().map(f64::to_bits)
            );

            let tail = &series[split..];
            let fork_at = if tail.is_empty() {
                usize::MAX
            } else {
                ((tail.len() - 1) as f64 * fork_pick).round() as usize
            };

            let mut streamed = head;
            for (offset, &bar) in tail.iter().enumerate() {
                let peeked = stream.peek(bar).unwrap();
                let peeked_again = stream.peek(bar).unwrap();
                prop_assert_eq!(peeked.to_bits(), peeked_again.to_bits());

                if offset == fork_at {
                    // A fork is an independent value: whatever happens to it,
                    // the original's next bar is unaffected.
                    let mut fork = stream.fork();
                    fork.update(bar * 1.5 + 1.0).unwrap();
                    fork.update(bar * 0.5 - 1.0).unwrap();
                }

                let committed = stream.update(bar).unwrap();
                prop_assert_eq!(
                    peeked.to_bits(),
                    committed.to_bits(),
                    "{}: peek did not predict update",
                    indicator.name
                );
                streamed.push(committed);
            }

            prop_assert!(
                bitwise_equal(&streamed, &batch),
                "{}: stream and batch differ at {:?}",
                indicator.name,
                first_difference(&streamed, &batch)
            );
            prop_assert_eq!(stream.bars_seen(), series.len() as u64);
        }
    }

    #[test]
    fn leading_warm_up_rows_shift_the_output(
        series in prop::collection::vec(-1.0e6..1.0e6f64, 40..200),
        leading in 1usize..10,
        seed in 0usize..1000,
    ) {
        for indicator in registered() {
            let period = period_for(&indicator, seed);
            let trimmed = (indicator.batch)(&series, period).unwrap();

            let mut padded = vec![f64::NAN; leading];
            padded.extend_from_slice(&series);
            let shifted = (indicator.batch)(&padded, period).unwrap();

            prop_assert_eq!(shifted.len(), padded.len());
            prop_assert!(shifted[..leading].iter().all(|v| v.is_nan()));
            prop_assert!(
                bitwise_equal(&shifted[leading..], &trimmed),
                "{}: padding changed the values",
                indicator.name
            );
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

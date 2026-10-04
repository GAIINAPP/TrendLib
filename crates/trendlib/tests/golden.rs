//! Every golden file is checked against the implementation, and every golden
//! file is also checked against the stream, because a value that batch gets
//! right and the stream gets wrong is still a wrong number on a live screen.

mod support;

use support::{bitwise_equal, find, first_difference, golden_files, read_golden};

#[test]
fn every_golden_file_matches_the_implementation() {
    let files = golden_files();
    assert!(
        !files.is_empty(),
        "no golden files found under crates/trendlib/src/indicators/*/golden/"
    );

    for path in files {
        let golden = read_golden(&path);
        let shown = path.display();
        let indicator = find(&golden.indicator)
            .unwrap_or_else(|| panic!("{shown}: no indicator named {}", golden.indicator));

        let source = golden.column("source");
        let outputs: Vec<String> = golden
            .columns
            .iter()
            .filter(|column| *column != "source")
            .cloned()
            .collect();
        assert_eq!(outputs.len(), 1, "{shown}: M1 indicators have one output");

        let expected = golden.column(&outputs[0]);
        let period = golden.period();
        let actual = (indicator.batch)(&source, period)
            .unwrap_or_else(|e| panic!("{shown}: batch failed: {e}"));

        assert_eq!(actual.len(), expected.len(), "{shown}: output length");

        let mut skipped = 0usize;
        let mut compared = 0usize;
        for (row, (&got, &want)) in actual.iter().zip(&expected).enumerate() {
            if golden.is_excluded(row) {
                skipped += 1;
                continue;
            }
            assert_eq!(
                got.is_nan(),
                want.is_nan(),
                "{shown}: row {row} is {got} but the oracle says {want}"
            );
            if want.is_nan() {
                continue;
            }
            let difference = (got - want).abs();
            assert!(
                difference <= golden.abs || difference <= golden.rel * want.abs(),
                "{shown}: row {row} is {got}, oracle {want}, off by {difference} \
                 (tolerance rel={} abs={})",
                golden.rel,
                golden.abs
            );
            compared += 1;
        }
        assert!(compared > 0, "{shown}: nothing was compared");
        println!("{shown}: {compared} rows compared, {skipped} excluded");

        // The same file also pins the stream: open on the shortest history that
        // can produce a value, feed the rest, and demand the batch output back
        // bar for bar.
        let lookback = (indicator.lookback)(period);
        let split = lookback + 1;
        assert!(
            split < source.len(),
            "{shown}: dataset is too short to stream"
        );
        let (mut stream, head) = (indicator.open_and_fill)(&source[..split], period)
            .unwrap_or_else(|e| panic!("{shown}: open_and_fill failed: {e}"));
        let mut streamed = head;
        for &bar in &source[split..] {
            streamed.push(
                stream
                    .update(bar)
                    .unwrap_or_else(|e| panic!("{shown}: update failed: {e}")),
            );
        }
        assert!(
            bitwise_equal(&streamed, &actual),
            "{shown}: stream and batch differ first at row {:?}",
            first_difference(&streamed, &actual)
        );
        assert_eq!(
            stream.bars_seen(),
            source.len() as u64,
            "{shown}: bars_seen"
        );
        assert_eq!(
            stream.value().map(f64::to_bits),
            actual.last().copied().map(f64::to_bits),
            "{shown}: stream value is not the last batch row"
        );
    }
}

#[test]
fn every_indicator_has_a_default_and_a_min_period_golden() {
    for indicator in support::registered() {
        for case in ["default", "min_period"] {
            let path = support::indicators_dir()
                .join(indicator.name)
                .join("golden")
                .join(format!("{case}.csv"));
            assert!(path.exists(), "{} has no golden/{case}.csv", indicator.name);
        }
    }
}

#[test]
fn golden_params_are_inside_the_documented_range() {
    for path in golden_files() {
        let golden = read_golden(&path);
        let indicator = find(&golden.indicator).expect("known indicator");
        let period = golden.period();
        assert!(
            (indicator.min_period..=indicator.max_period).contains(&period),
            "{}: period={period} is outside [{}, {}]",
            path.display(),
            indicator.min_period,
            indicator.max_period
        );
        if golden.case == "default" {
            assert_eq!(period, indicator.default_period, "{}", path.display());
        }
        if golden.case == "min_period" {
            assert_eq!(period, indicator.min_period, "{}", path.display());
        }
    }
}

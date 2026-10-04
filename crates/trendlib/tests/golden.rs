//! Every golden file is checked against the implementation, and against the
//! stream too, because a value batch gets right and the stream gets wrong is
//! still a wrong number on a live screen.

mod support;

use support::{as_slices, bitwise_equal, find, first_difference, golden_files, read_golden};

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

        let columns: Vec<Vec<f64>> = indicator
            .inputs
            .iter()
            .map(|name| golden.column(name))
            .collect();
        let inputs = as_slices(&columns);
        let period = golden.period();

        let actual = (indicator.batch)(&inputs, period)
            .unwrap_or_else(|e| panic!("{shown}: batch failed: {e}"));
        assert_eq!(
            actual.len(),
            indicator.outputs.len(),
            "{shown}: output count"
        );

        let mut compared = 0usize;
        let mut skipped = 0usize;
        for (index, name) in indicator.outputs.iter().enumerate() {
            let expected = golden.column(name);
            let got = &actual[index];
            assert_eq!(got.len(), expected.len(), "{shown}: {name} length");
            for (row, (&value, &want)) in got.iter().zip(&expected).enumerate() {
                if golden.is_excluded(row) {
                    skipped += 1;
                    continue;
                }
                assert_eq!(
                    value.is_nan(),
                    want.is_nan(),
                    "{shown}: {name} row {row} is {value} but the oracle says {want}"
                );
                if want.is_nan() {
                    continue;
                }
                let difference = (value - want).abs();
                assert!(
                    difference <= golden.abs || difference <= golden.rel * want.abs(),
                    "{shown}: {name} row {row} is {value}, oracle {want}, off by {difference} \
                     (tolerance rel={} abs={})",
                    golden.rel,
                    golden.abs
                );
                compared += 1;
            }
        }
        assert!(compared > 0, "{shown}: nothing was compared");
        println!("{shown}: {compared} values compared, {skipped} excluded");

        // The same file also pins the stream: open on the shortest history that
        // can produce a value, feed the rest, and demand the batch output back.
        let lookback = (indicator.lookback)(period);
        let split = lookback + 1;
        let rows = columns[0].len();
        assert!(split < rows, "{shown}: dataset is too short to stream");

        let history = support::head(&columns, split);
        let (mut stream, filled) = (indicator.open_and_fill)(&as_slices(&history), period)
            .unwrap_or_else(|e| panic!("{shown}: open_and_fill failed: {e}"));
        let mut streamed = filled;
        for row in split..rows {
            let bar: Vec<f64> = columns.iter().map(|c| c[row]).collect();
            let values = stream
                .update(&bar)
                .unwrap_or_else(|e| panic!("{shown}: update failed at row {row}: {e}"));
            for (column, value) in streamed.iter_mut().zip(values) {
                column.push(value);
            }
        }
        for (index, name) in indicator.outputs.iter().enumerate() {
            assert!(
                bitwise_equal(&streamed[index], &actual[index]),
                "{shown}: {name} stream and batch differ first at row {:?}",
                first_difference(&streamed[index], &actual[index])
            );
        }
        assert_eq!(stream.bars_seen(), rows as u64, "{shown}: bars_seen");
    }
}

#[test]
fn every_indicator_has_the_golden_cases_its_parameters_call_for() {
    for indicator in support::registered() {
        let folder = support::indicators_dir()
            .join(indicator.name)
            .join("golden");
        assert!(
            folder.join("default.csv").exists(),
            "{} has no golden/default.csv",
            indicator.name
        );
        // A boundary case is only meaningful for an indicator that has a
        // boundary; `trange` takes no parameters at all.
        if indicator.period.is_some() {
            assert!(
                folder.join("min_period.csv").exists(),
                "{} has a period but no golden/min_period.csv",
                indicator.name
            );
        }
    }
}

#[test]
fn golden_params_are_inside_the_documented_range() {
    for path in golden_files() {
        let golden = read_golden(&path);
        let indicator = find(&golden.indicator).expect("known indicator");
        let Some(period) = golden.period() else {
            assert!(
                indicator.period.is_none(),
                "{}: {} takes a period but the header has none",
                path.display(),
                indicator.name
            );
            continue;
        };
        let (default, min, max) = indicator.period.expect("a period in the header");
        assert!(
            (min..=max).contains(&period),
            "{}: period={period} is outside [{min}, {max}]",
            path.display()
        );
        if golden.case == "default" {
            assert_eq!(period, default, "{}", path.display());
        }
        if golden.case == "min_period" && period != min {
            assert!(
                golden
                    .note()
                    .is_some_and(|note| note.contains("CONVENTIONS.md")),
                "{}: the boundary case uses period={period} instead of {min} without a \
                 `# note:` header pointing at the deviation that explains it",
                path.display()
            );
        }
    }
}

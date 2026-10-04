//! Every golden file is checked against the implementation, and against the
//! stream too, because a value batch gets right and the stream gets wrong is
//! still a wrong number on a live screen.

mod support;

use support::{as_slices, bitwise_equal, find, first_difference, golden_files, read_golden};
use trendlib::core::math::{MaType, VwapAnchor};

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
        let values = golden.values(indicator.params);

        let actual = (indicator.batch)(&inputs, &values)
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
            // An integer output's warm-up rows are 0, not NaN, which is what
            // the oracle wrote and what callers are handed.
            let narrowed: Vec<f64>;
            let got = if indicator
                .integer_outputs
                .get(index)
                .copied()
                .unwrap_or(false)
            {
                narrowed = actual[index]
                    .iter()
                    .map(|v| if v.is_nan() { 0.0 } else { *v })
                    .collect();
                &narrowed
            } else {
                &actual[index]
            };
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
        let lookback = (indicator.lookback)(&values);
        let split = lookback + 1;
        let rows = columns[0].len();
        assert!(split < rows, "{shown}: dataset is too short to stream");

        let history = support::head(&columns, split);
        let (mut stream, filled) = (indicator.open_and_fill)(&as_slices(&history), &values)
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
        if indicator.params.iter().any(|p| p.integral) {
            assert!(
                folder.join("min_period.csv").exists(),
                "{} has a period but no golden/min_period.csv",
                indicator.name
            );
        }
    }
}

/// The generator decides which averages `ma_type` accepts by looking for an
/// indicator of that name; `MaType` decides by what it has a variant for. They
/// are two readings of the same rule, so a new average that reaches only one of
/// them would silently change what callers can ask for.
#[test]
fn the_registry_lists_every_average_the_core_has() {
    let core: Vec<&str> = MaType::ALL.iter().map(|(name, _)| *name).collect();
    assert_eq!(support::registry::MA_TYPES, core.as_slice());
    let anchors: Vec<&str> = VwapAnchor::ALL.iter().map(|(name, _)| *name).collect();
    assert_eq!(support::registry::VWAP_ANCHORS, anchors.as_slice());
    for pending in MaType::PENDING {
        assert!(
            !core.contains(pending),
            "{pending} is implemented; take it out of MaType::PENDING"
        );
    }
}

#[test]
fn golden_params_are_inside_the_documented_range() {
    for path in golden_files() {
        let golden = read_golden(&path);
        let indicator = find(&golden.indicator).expect("known indicator");
        let values = golden.values(indicator.params);
        for (param, value) in indicator.params.iter().zip(&values) {
            assert!(
                (param.min..=param.max).contains(value),
                "{}: {}={value} is outside [{}, {}]",
                path.display(),
                param.name,
                param.min,
                param.max
            );
            if golden.case == "default" {
                assert_eq!(*value, param.default, "{}: {}", path.display(), param.name);
            }
            if golden.case == "min_period" && param.integral && *value != param.min {
                assert!(
                    golden
                        .note()
                        .is_some_and(|note| note.contains("CONVENTIONS.md")),
                    "{}: the boundary case uses {}={value} instead of {} without a \
                     `# note:` header pointing at the deviation that explains it",
                    path.display(),
                    param.name,
                    param.min
                );
            }
        }
    }
}

//! Every indicator over a million bars, so the Rust core can be timed apart
//! from the Python layer that wraps it (`docs/TESTING.md` section 8).
//!
//! The registry the test suites use is reused here rather than copied: it is
//! generated from the same specs, so a new indicator is benchmarked the moment
//! its folder exists.

#[path = "../tests/support/mod.rs"]
mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use support::{Registered, as_slices, registered};

/// The size `docs/TESTING.md` section 8 specifies.
const BARS: usize = 1_000_000;

/// A deterministic walk, built without a random-number dependency so the core
/// crate's rule about dependencies is not bent for a benchmark.
fn walk(bars: usize) -> Vec<f64> {
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut price = 1000.0_f64;
    (0..bars)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let unit = (state >> 11) as f64 / (1_u64 << 53) as f64;
            price *= 1.0 + (unit - 0.5) * 0.02;
            price
        })
        .collect()
}

fn column(name: &str, base: &[f64]) -> Vec<f64> {
    match name {
        "high" => base.iter().map(|v| v + v.abs() * 0.005 + 0.5).collect(),
        "low" => base.iter().map(|v| v - v.abs() * 0.005 - 0.5).collect(),
        "volume" => base.iter().map(|v| v.abs() * 10.0).collect(),
        "open" | "source1" => base.iter().map(|v| v * 0.75 + 1.0).collect(),
        "periods" => (0..base.len()).map(|row| 2.0 + (row % 29) as f64).collect(),
        "timestamps" => (0..base.len())
            .map(|row| 1.767_225_6e18 + row as f64 * 8.64e13)
            .collect(),
        _ => base.to_vec(),
    }
}

fn inputs_for(indicator: &Registered, base: &[f64]) -> Vec<Vec<f64>> {
    indicator
        .inputs
        .iter()
        .map(|name| column(name, base))
        .collect()
}

fn batch(criterion: &mut Criterion) {
    let base = walk(BARS);
    let mut group = criterion.benchmark_group("batch");
    group.sample_size(10);
    for indicator in registered() {
        let columns = inputs_for(&indicator, &base);
        let values = indicator.defaults();
        group.bench_function(indicator.name, |bencher| {
            bencher.iter(|| {
                let slices = as_slices(&columns);
                black_box((indicator.batch)(&slices, &values).expect("batch"))
            })
        });
    }
    group.finish();
}

criterion_group!(benches, batch);
criterion_main!(benches);

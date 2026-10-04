//! Shared scaffolding for the M1 test suites.
//!
//! The adapters below exist only because M1 has no registry yet: each one
//! erases an indicator's concrete types so a suite can loop over all of them.
//! `cargo xtask generate` replaces this file in M2.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use trendlib::TlError;
use trendlib::core::traits::{Indicator, Stream};
use trendlib::indicators::{ema, rsi, sma};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate lives at <repo>/crates/trendlib")
        .to_path_buf()
}

pub fn indicators_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/indicators")
}

/// A stream with its concrete type erased, so suites can treat every
/// single-series indicator the same way.
pub trait AnyStream: std::fmt::Debug {
    fn update(&mut self, bar: f64) -> Result<f64, TlError>;
    fn peek(&self, bar: f64) -> Result<f64, TlError>;
    fn value(&self) -> Option<f64>;
    fn bars_seen(&self) -> u64;
    fn fork(&self) -> Box<dyn AnyStream>;
}

impl<S> AnyStream for S
where
    S: Stream<Bar = f64, Value = f64> + std::fmt::Debug + 'static,
{
    fn update(&mut self, bar: f64) -> Result<f64, TlError> {
        Stream::update(self, bar)
    }

    fn peek(&self, bar: f64) -> Result<f64, TlError> {
        Stream::peek(self, bar)
    }

    fn value(&self) -> Option<f64> {
        Stream::value(self)
    }

    fn bars_seen(&self) -> u64 {
        Stream::bars_seen(self)
    }

    fn fork(&self) -> Box<dyn AnyStream> {
        Box::new(self.clone())
    }
}

pub type BatchFn = fn(&[f64], usize) -> Result<Vec<f64>, TlError>;
pub type LookbackFn = fn(usize) -> usize;
pub type OpenFn = fn(&[f64], usize) -> Result<Box<dyn AnyStream>, TlError>;
pub type OpenAndFillFn = fn(&[f64], usize) -> Result<(Box<dyn AnyStream>, Vec<f64>), TlError>;

pub struct Registered {
    pub name: &'static str,
    pub default_period: usize,
    pub min_period: usize,
    pub max_period: usize,
    pub batch: BatchFn,
    pub lookback: LookbackFn,
    pub open: OpenFn,
    pub open_and_fill: OpenAndFillFn,
}

macro_rules! adapter {
    ($adapter:ident, $module:ident, $type:ident, $name:literal, $default:literal, $min:literal, $max:literal) => {
        mod $adapter {
            use super::*;

            type Indicated = $module::$type;

            fn params(period: usize) -> $module::Params {
                $module::Params { period }
            }

            fn batch(source: &[f64], period: usize) -> Result<Vec<f64>, TlError> {
                <Indicated as Indicator>::batch(source, &params(period))
            }

            fn lookback(period: usize) -> usize {
                <Indicated as Indicator>::lookback(&params(period))
            }

            fn open(source: &[f64], period: usize) -> Result<Box<dyn AnyStream>, TlError> {
                <<Indicated as Indicator>::Stream as Stream>::open(source, &params(period))
                    .map(|stream| Box::new(stream) as Box<dyn AnyStream>)
            }

            fn open_and_fill(
                source: &[f64],
                period: usize,
            ) -> Result<(Box<dyn AnyStream>, Vec<f64>), TlError> {
                <Indicated as Indicator>::open_and_fill(source, &params(period))
                    .map(|(stream, out)| (Box::new(stream) as Box<dyn AnyStream>, out))
            }

            pub fn registered() -> Registered {
                Registered {
                    name: $name,
                    default_period: $default,
                    min_period: $min,
                    max_period: $max,
                    batch,
                    lookback,
                    open,
                    open_and_fill,
                }
            }
        }
    };
}

adapter!(sma_adapter, sma, Sma, "sma", 30, 1, 100_000);
adapter!(ema_adapter, ema, Ema, "ema", 30, 1, 100_000);
adapter!(rsi_adapter, rsi, Rsi, "rsi", 14, 2, 100_000);

pub fn registered() -> Vec<Registered> {
    vec![
        sma_adapter::registered(),
        ema_adapter::registered(),
        rsi_adapter::registered(),
    ]
}

pub fn find(name: &str) -> Option<Registered> {
    registered().into_iter().find(|i| i.name == name)
}

/// One parsed golden file: the header from `docs/SPEC_FORMAT.md` section 4,
/// plus the input and output columns.
pub struct Golden {
    pub path: PathBuf,
    pub indicator: String,
    pub case: String,
    pub params: BTreeMap<String, String>,
    pub rel: f64,
    pub abs: f64,
    pub excluded: Vec<(usize, usize)>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<f64>>,
}

impl Golden {
    pub fn column(&self, name: &str) -> Vec<f64> {
        let index = self
            .columns
            .iter()
            .position(|c| c == name)
            .unwrap_or_else(|| panic!("{}: no column {name}", self.path.display()));
        self.rows.iter().map(|row| row[index]).collect()
    }

    pub fn period(&self) -> usize {
        self.params
            .get("period")
            .unwrap_or_else(|| panic!("{}: no period in the params header", self.path.display()))
            .parse()
            .expect("period is an integer")
    }

    pub fn is_excluded(&self, row: usize) -> bool {
        self.excluded
            .iter()
            .any(|&(start, end)| row >= start && row <= end)
    }
}

fn parse_tolerance(value: &str, path: &Path) -> (f64, f64) {
    let mut rel = None;
    let mut abs = None;
    for field in value.split_whitespace() {
        let (key, number) = field
            .split_once('=')
            .unwrap_or_else(|| panic!("{}: bad tolerance field {field}", path.display()));
        let parsed: f64 = number
            .parse()
            .unwrap_or_else(|_| panic!("{}: bad tolerance {number}", path.display()));
        match key {
            "rel" => rel = Some(parsed),
            "abs" => abs = Some(parsed),
            other => panic!("{}: unknown tolerance key {other}", path.display()),
        }
    }
    (
        rel.unwrap_or_else(|| panic!("{}: tolerance has no rel", path.display())),
        abs.unwrap_or_else(|| panic!("{}: tolerance has no abs", path.display())),
    )
}

fn parse_excluded(value: &str, path: &Path) -> Vec<(usize, usize)> {
    if value == "none" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|part| {
            let range = part.trim().split('(').next().unwrap().trim();
            let (start, end) = range
                .split_once('-')
                .unwrap_or_else(|| panic!("{}: bad excluded range {range}", path.display()));
            (
                start.trim().parse().expect("row index"),
                end.trim().parse().expect("row index"),
            )
        })
        .collect()
}

pub fn read_golden(path: &Path) -> Golden {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut header = BTreeMap::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.peek() {
        let Some(rest) = line.strip_prefix("# ") else {
            break;
        };
        let (key, value) = rest
            .split_once(": ")
            .unwrap_or_else(|| panic!("{}: bad header line {line}", path.display()));
        header.insert(key.to_string(), value.to_string());
        lines.next();
    }

    for required in [
        "indicator",
        "case",
        "params",
        "oracle",
        "produced_by",
        "input",
        "tolerance",
        "excluded_rows",
        "date",
    ] {
        assert!(
            header.contains_key(required),
            "{}: header is missing `{required}` (docs/SPEC_FORMAT.md § 4)",
            path.display()
        );
    }

    let produced_by = &header["produced_by"];
    assert!(
        produced_by.contains("scripts/oracle/") || produced_by == "manual",
        "{}: produced_by must name an oracle script or be `manual`, got {produced_by}",
        path.display()
    );

    let params = if header["params"] == "none" {
        BTreeMap::new()
    } else {
        header["params"]
            .split(", ")
            .map(|field| {
                let (key, value) = field
                    .split_once('=')
                    .unwrap_or_else(|| panic!("{}: bad param {field}", path.display()));
                (key.to_string(), value.to_string())
            })
            .collect()
    };

    let (rel, abs) = parse_tolerance(&header["tolerance"], path);
    let excluded = parse_excluded(&header["excluded_rows"], path);

    let columns: Vec<String> = lines
        .next()
        .unwrap_or_else(|| panic!("{}: no column header", path.display()))
        .split(',')
        .map(str::to_string)
        .collect();

    let rows: Vec<Vec<f64>> = lines
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split(',')
                .map(|cell| {
                    cell.parse::<f64>()
                        .unwrap_or_else(|_| panic!("{}: bad number {cell}", path.display()))
                })
                .collect()
        })
        .collect();

    Golden {
        path: path.to_path_buf(),
        indicator: header["indicator"].clone(),
        case: header["case"].clone(),
        params,
        rel,
        abs,
        excluded,
        columns,
        rows,
    }
}

pub fn golden_files() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let root = indicators_dir();
    for entry in fs::read_dir(&root).expect("indicators folder") {
        let folder = entry.expect("dir entry").path();
        let golden = folder.join("golden");
        if !golden.is_dir() {
            continue;
        }
        for file in fs::read_dir(&golden).expect("golden folder") {
            let path = file.expect("dir entry").path();
            if path.extension().is_some_and(|e| e == "csv") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// Close prices from the committed daily dataset.
pub fn daily_closes() -> Vec<f64> {
    let path = repo_root().join("testdata/daily_2000.csv");
    let text = fs::read_to_string(&path).expect("testdata/daily_2000.csv");
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().expect("header").split(',').collect();
    let close = header
        .iter()
        .position(|c| *c == "close")
        .expect("close column");
    lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split(',').nth(close).unwrap().parse().unwrap())
        .collect()
}

/// `true` when two outputs are identical bar for bar, counting NaN as equal to
/// NaN. Bitwise, so a value that merely rounds to the same decimal fails.
pub fn bitwise_equal(left: &[f64], right: &[f64]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits())
}

pub fn first_difference(left: &[f64], right: &[f64]) -> Option<usize> {
    left.iter()
        .zip(right)
        .position(|(a, b)| !((a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()))
}

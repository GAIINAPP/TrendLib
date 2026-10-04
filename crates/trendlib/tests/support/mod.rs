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
use trendlib::core::kernel::{BarStream, Kernel};
use trendlib::core::traits::Stream;

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

/// A stream with its concrete type erased, so a suite can treat every
/// indicator the same way regardless of how many inputs or outputs it has.
pub trait AnyStream: std::fmt::Debug {
    fn update(&mut self, bar: &[f64]) -> Result<Vec<f64>, TlError>;
    fn peek(&self, bar: &[f64]) -> Result<Vec<f64>, TlError>;
    fn value(&self) -> Option<Vec<f64>>;
    fn bars_seen(&self) -> u64;
    fn fork(&self) -> Box<dyn AnyStream>;
}

impl<K, const I: usize, const O: usize> AnyStream for BarStream<K, I, O>
where
    K: Kernel<I, O> + std::fmt::Debug + 'static,
    K::Params: std::fmt::Debug,
    K::State: std::fmt::Debug,
{
    fn update(&mut self, bar: &[f64]) -> Result<Vec<f64>, TlError> {
        let fixed: [f64; I] = std::array::from_fn(|i| bar[i]);
        Stream::update(self, fixed).map(|row| row.to_vec())
    }

    fn peek(&self, bar: &[f64]) -> Result<Vec<f64>, TlError> {
        let fixed: [f64; I] = std::array::from_fn(|i| bar[i]);
        Stream::peek(self, fixed).map(|row| row.to_vec())
    }

    fn value(&self) -> Option<Vec<f64>> {
        Stream::value(self).map(|row| row.to_vec())
    }

    fn bars_seen(&self) -> u64 {
        Stream::bars_seen(self)
    }

    fn fork(&self) -> Box<dyn AnyStream> {
        Box::new(self.clone())
    }
}

/// One parameter, as the spec declares it.
#[derive(Clone, Copy, Debug)]
pub struct ParamSpec {
    pub name: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub integral: bool,
}

pub type Columns = Vec<Vec<f64>>;
pub type OpenAndFillResult = Result<(Box<dyn AnyStream>, Columns), TlError>;

pub type BatchFn = fn(&[&[f64]], &[f64]) -> Result<Columns, TlError>;
pub type LookbackFn = fn(&[f64]) -> usize;
pub type OpenAndFillFn = fn(&[&[f64]], &[f64]) -> OpenAndFillResult;

/// An indicator with its types erased. Parameters travel as `f64` in spec
/// order, so one shape covers an indicator with none and one with three.
pub struct Registered {
    pub name: &'static str,
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub params: &'static [ParamSpec],
    pub batch: BatchFn,
    pub lookback: LookbackFn,
    pub open_and_fill: OpenAndFillFn,
}

impl Registered {
    pub fn defaults(&self) -> Vec<f64> {
        self.params.iter().map(|p| p.default).collect()
    }

    /// Defaults with one parameter replaced, for the boundary tests.
    pub fn with(&self, name: &str, value: f64) -> Vec<f64> {
        self.params
            .iter()
            .map(|p| if p.name == name { value } else { p.default })
            .collect()
    }

    /// A period-like parameter capped, so a property case stays quick.
    pub fn capped(&self, seed: usize, cap: f64) -> Vec<f64> {
        self.params
            .iter()
            .map(|p| {
                if !p.integral {
                    return p.default;
                }
                let high = p.max.min(cap);
                if high <= p.min {
                    return p.min;
                }
                let span = (high - p.min) as usize + 1;
                p.min + (seed % span) as f64
            })
            .collect()
    }

    pub fn open(&self, inputs: &[&[f64]], values: &[f64]) -> Result<Box<dyn AnyStream>, TlError> {
        (self.open_and_fill)(inputs, values).map(|(stream, _)| stream)
    }
}

mod registry;

pub use registry::registered;

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
    pub note: Option<String>,
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

    /// Parameter values for this case, in the order the spec declares them.
    pub fn values(&self, params: &[ParamSpec]) -> Vec<f64> {
        params
            .iter()
            .map(|param| {
                self.params
                    .get(param.name)
                    .map(|value| {
                        value.parse().unwrap_or_else(|_| {
                            panic!("{}: {} is not a number", self.path.display(), param.name)
                        })
                    })
                    .unwrap_or(param.default)
            })
            .collect()
    }

    /// Set when the case could not use the documented boundary, with the reason.
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
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
        note: header.get("note").cloned(),
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

/// One column of the committed daily dataset.
pub fn daily_column(name: &str) -> Vec<f64> {
    let path = repo_root().join("testdata/daily_2000.csv");
    let text = fs::read_to_string(&path).expect("testdata/daily_2000.csv");
    let mut lines = text.lines();
    let wanted = if name == "source" { "close" } else { name };
    let header: Vec<&str> = lines.next().expect("header").split(',').collect();
    let column = header
        .iter()
        .position(|c| *c == wanted)
        .unwrap_or_else(|| panic!("daily_2000.csv has no column {wanted}"));
    lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split(',').nth(column).unwrap().parse().unwrap())
        .collect()
}

/// The columns an indicator asks for, taken from the committed daily dataset.
pub fn daily_inputs(indicator: &Registered) -> Vec<Vec<f64>> {
    indicator
        .inputs
        .iter()
        .map(|name| daily_column(name))
        .collect()
}

pub fn as_slices(columns: &[Vec<f64>]) -> Vec<&[f64]> {
    columns.iter().map(Vec::as_slice).collect()
}

/// Truncate every column to the same length.
pub fn head(columns: &[Vec<f64>], len: usize) -> Vec<Vec<f64>> {
    columns
        .iter()
        .map(|c| c[..len.min(c.len())].to_vec())
        .collect()
}

/// A plausible bar from one base series, so multi-input indicators can be fed
/// random data without violating `high >= low`.
pub fn bars_from(base: &[f64], names: &[&str]) -> Vec<Vec<f64>> {
    names
        .iter()
        .map(|name| match *name {
            "high" => base.iter().map(|v| v + v.abs() * 0.005 + 0.5).collect(),
            "low" => base.iter().map(|v| v - v.abs() * 0.005 - 0.5).collect(),
            "volume" => base.iter().map(|v| v.abs() * 10.0).collect(),
            _ => base.to_vec(),
        })
        .collect()
}

use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pivots_camarilla";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    previous: Option<[f64; 3]>,
}

/// The divisors Nick Scott's equation spaces the levels by.
const SPACING: [f64; 4] = [12.0, 6.0, 4.0, 2.0];
const FACTOR: f64 = 1.1;

fn levels(bar: [f64; 3]) -> [f64; 8] {
    let [high, low, close] = bar;
    let range = high - low;
    let step = |divisor: f64| FACTOR * range / divisor;
    [
        close + step(SPACING[0]),
        close + step(SPACING[1]),
        close + step(SPACING[2]),
        close + step(SPACING[3]),
        close - step(SPACING[0]),
        close - step(SPACING[1]),
        close - step(SPACING[2]),
        close - step(SPACING[3]),
    ]
}

impl Step<3, 8> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 8]> {
        self.previous.replace(bar).map(levels)
    }

    fn preview(&self, _bar: [f64; 3]) -> Option<[f64; 8]> {
        self.previous.map(levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PivotsCamarilla;

pub type PivotsCamarillaStream = BarStream<PivotsCamarilla, 3, 8>;

impl Kernel<3, 8> for PivotsCamarilla {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 8] = [
        "camarilla_r1",
        "camarilla_r2",
        "camarilla_r3",
        "camarilla_r4",
        "camarilla_s1",
        "camarilla_s2",
        "camarilla_s3",
        "camarilla_s4",
    ];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(_params: &Params) -> State {
        State { previous: None }
    }
}

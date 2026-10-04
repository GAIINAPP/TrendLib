use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pivots_traditional";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug, Default)]
pub struct State {
    previous: Option<[f64; 3]>,
}

fn levels(bar: [f64; 3]) -> [f64; 7] {
    let [high, low, close] = bar;
    let pivot = (high + low + close) / 3.0;
    let range = high - low;
    [
        pivot,
        2.0 * pivot - low,
        pivot + range,
        high + 2.0 * (pivot - low),
        2.0 * pivot - high,
        pivot - range,
        low - 2.0 * (high - pivot),
    ]
}

impl Step<3, 7> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 7]> {
        self.previous.replace(bar).map(levels)
    }

    fn preview(&self, _bar: [f64; 3]) -> Option<[f64; 7]> {
        self.previous.map(levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PivotsTraditional;

pub type PivotsTraditionalStream = BarStream<PivotsTraditional, 3, 7>;

impl Kernel<3, 7> for PivotsTraditional {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 7] = [
        "pivots_traditional_pp",
        "pivots_traditional_r1",
        "pivots_traditional_r2",
        "pivots_traditional_r3",
        "pivots_traditional_s1",
        "pivots_traditional_s2",
        "pivots_traditional_s3",
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

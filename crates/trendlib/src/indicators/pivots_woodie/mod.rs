use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pivots_woodie";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<[f64; 3]>,
}

impl State {
    fn levels([high, low, close]: [f64; 3]) -> [f64; 5] {
        let pivot = (high + low + 2.0 * close) / 4.0;
        let range = high - low;
        [
            pivot,
            2.0 * pivot - low,
            2.0 * pivot - high,
            pivot + range,
            pivot - range,
        ]
    }
}

impl Step<3, 5> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 5]> {
        let levels = self.previous.map(Self::levels);
        self.previous = Some(bar);
        levels
    }

    fn preview(&self, _bar: [f64; 3]) -> Option<[f64; 5]> {
        self.previous.map(Self::levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PivotsWoodie;

pub type PivotsWoodieStream = BarStream<PivotsWoodie, 3, 5>;

impl Kernel<3, 5> for PivotsWoodie {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 5] = [
        "woodie_pp",
        "woodie_r1",
        "woodie_s1",
        "woodie_r2",
        "woodie_s2",
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

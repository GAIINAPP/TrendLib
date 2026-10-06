use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pivots_demark";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<[f64; 4]>,
}

impl State {
    fn levels([open, high, low, close]: [f64; 4]) -> [f64; 3] {
        let sum = if close < open {
            high + 2.0 * low + close
        } else if close > open {
            2.0 * high + low + close
        } else {
            high + low + 2.0 * close
        };
        [sum / 4.0, sum / 2.0 - low, sum / 2.0 - high]
    }
}

impl Step<4, 3> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 3]> {
        let levels = self.previous.map(Self::levels);
        self.previous = Some(bar);
        levels
    }

    fn preview(&self, _bar: [f64; 4]) -> Option<[f64; 3]> {
        self.previous.map(Self::levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PivotsDemark;

pub type PivotsDemarkStream = BarStream<PivotsDemark, 4, 3>;

impl Kernel<4, 3> for PivotsDemark {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 3] = ["demark_pp", "demark_r1", "demark_s1"];

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

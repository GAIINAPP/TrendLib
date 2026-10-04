use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "marketfi";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Copy, Debug, Default)]
pub struct State;

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.preview(bar)
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        // No volume means no trading to divide the range among, and TA-Lib
        // answers zero rather than an infinity. The test is exact.
        let volume = bar[2];
        Some([if volume == 0.0 {
            0.0
        } else {
            (bar[0] - bar[1]) / volume
        }])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Marketfi;

pub type MarketfiStream = BarStream<Marketfi, 3, 1>;

impl Kernel<3, 1> for Marketfi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "volume"];
    const OUTPUTS: [&'static str; 1] = ["marketfi"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(_params: &Params) -> State {
        State
    }
}

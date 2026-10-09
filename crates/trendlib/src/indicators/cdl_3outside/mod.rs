use crate::core::candles::PatternState;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3outside";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// An engulfing pattern carried on by a third bar closing further in the same direction
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    let swallowed_up = middle.is_white()
        && first.is_black()
        && middle.close > first.open
        && middle.open < first.close
        && bar.close > middle.close;
    let swallowed_down = middle.is_black()
        && first.is_white()
        && middle.open > first.close
        && middle.close < first.open
        && bar.close < middle.close;
    if swallowed_up || swallowed_down {
        middle.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3outside;

pub type Cdl3outsideStream = BarStream<Cdl3outside, 4, 1>;

impl Kernel<4, 1> for Cdl3outside {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3outside"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        let _ = params;
        PatternState::new(LOOKBACK, 0.0, detect)
    }
}

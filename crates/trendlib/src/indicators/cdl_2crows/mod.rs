use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_2crows";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long white bar, a black one gapping above it, then a black one closing back inside it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    if first.is_white()
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && middle.is_black()
        && middle.body_gaps_above(first)
        && bar.is_black()
        && bar.open < middle.open
        && bar.open > middle.close
        && bar.close > first.open
        && bar.close < first.close
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl2crows;

pub type Cdl2crowsStream = BarStream<Cdl2crows, 4, 1>;

impl Kernel<4, 1> for Cdl2crows {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_2crows"];

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

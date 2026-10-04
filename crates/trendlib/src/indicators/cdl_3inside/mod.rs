use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3inside";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A harami carried on by a third bar closing past the first bar's open
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    let inside = middle.body_bottom() > first.body_bottom() && middle.body_top() < first.body_top();
    let confirmed = (first.is_white() && bar.is_black() && bar.close < first.open)
        || (first.is_black() && bar.is_white() && bar.close > first.open);
    if first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && middle.body() <= pattern.average(CandleSetting::BODY_SHORT, 1)
        && inside
        && confirmed
    {
        -first.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3inside;

pub type Cdl3insideStream = BarStream<Cdl3inside, 4, 1>;

impl Kernel<4, 1> for Cdl3inside {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3inside"];

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

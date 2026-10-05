use crate::core::candles::PatternState;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_engulfing";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A body that covers the whole of the previous one, in the other colour.
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let previous = pattern.back(1);
    let swallows_black = bar.is_white()
        && previous.is_black()
        && ((bar.close >= previous.open && bar.open < previous.close)
            || (bar.close > previous.open && bar.open <= previous.close));
    let swallows_white = bar.is_black()
        && previous.is_white()
        && ((bar.open >= previous.close && bar.close < previous.open)
            || (bar.open > previous.close && bar.close <= previous.open));
    if swallows_black || swallows_white {
        // One end of the body may sit exactly where the earlier one's did. The
        // bar still swallows the other, so it is reported, but at four fifths:
        // the oracle grades the two cases apart and a reader comparing against
        // it would see the difference.
        let touching = bar.open == previous.close || bar.close == previous.open;
        bar.colour() * if touching { 80.0 } else { 100.0 }
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlEngulfing;

pub type CdlEngulfingStream = BarStream<CdlEngulfing, 4, 1>;

impl Kernel<4, 1> for CdlEngulfing {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_engulfing"];

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

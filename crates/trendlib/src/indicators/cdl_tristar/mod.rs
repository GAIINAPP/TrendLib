use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_tristar";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three doji in a row, the middle one gapping clear and the third coming back
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    // One average serves all three bars, the one trailing the first of them.
    // TA-Lib keeps a single running total here where its other three-bar
    // patterns keep one per position.
    let doji = pattern.average(CandleSetting::BODY_DOJI, 2);
    if first.body() > doji || middle.body() > doji || bar.body() > doji {
        return 0.0;
    }
    if middle.body_gaps_above(first) && bar.body_top() < middle.body_top() {
        -100.0
    } else if middle.body_gaps_below(first) && bar.body_bottom() > middle.body_bottom() {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlTristar;

pub type CdlTristarStream = BarStream<CdlTristar, 4, 1>;

impl Kernel<4, 1> for CdlTristar {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_tristar"];

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

use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_unique3river";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long black bar, a smaller black one making a new low, then a short white one above it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    if first.is_black()
        && second.is_black()
        && bar.is_white()
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && second.open > first.close
        && second.open <= first.open
        && second.close > first.close
        && second.low < first.low
        && bar.body() < pattern.average(CandleSetting::BODY_SHORT, 0)
        && bar.open > second.low
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlUnique3river;

pub type CdlUnique3riverStream = BarStream<CdlUnique3river, 4, 1>;

impl Kernel<4, 1> for CdlUnique3river {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_unique3river"];

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

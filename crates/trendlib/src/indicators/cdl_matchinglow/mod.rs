use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_matchinglow";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Two black bars that closed at the same price
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let equal = pattern.average(CandleSetting::EQUAL, 1);
    if prior.is_black()
        && bar.is_black()
        && bar.close <= prior.close + equal
        && bar.close >= prior.close - equal
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlMatchinglow;

pub type CdlMatchinglowStream = BarStream<CdlMatchinglow, 4, 1>;

impl Kernel<4, 1> for CdlMatchinglow {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_matchinglow"];

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

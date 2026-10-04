use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_stalledpattern";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Two long white bars then a short one riding on the second's shoulder
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    if first.is_white()
        && second.is_white()
        && bar.is_white()
        && bar.close > second.close
        && second.close > first.close
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && second.body() > pattern.average(CandleSetting::BODY_LONG, 1)
        && second.upper_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, 1)
        && bar.body() < pattern.average(CandleSetting::BODY_SHORT, 0)
        && second.open > first.open
        && second.open <= first.close + pattern.average(CandleSetting::NEAR, 2)
        && bar.open > second.close - bar.body()
        && bar.open <= second.close + pattern.average(CandleSetting::NEAR, 1)
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlStalledpattern;

pub type CdlStalledpatternStream = BarStream<CdlStalledpattern, 4, 1>;

impl Kernel<4, 1> for CdlStalledpattern {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_stalledpattern"];

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

use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_marubozu";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long body with almost no wick at either end
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    if bar.body() > pattern.average(CandleSetting::BODY_LONG, 0)
        && bar.upper_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, 0)
        && bar.lower_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, 0)
    {
        bar.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlMarubozu;

pub type CdlMarubozuStream = BarStream<CdlMarubozu, 4, 1>;

impl Kernel<4, 1> for CdlMarubozu {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_marubozu"];

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

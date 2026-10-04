use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_rickshawman";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A doji with a long wick on each side and its body near the middle of the range
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let near = pattern.average(CandleSetting::NEAR, 0);
    let middle = bar.low + bar.range() / 2.0;
    let long_shadow = pattern.average(CandleSetting::SHADOW_LONG, 0);
    if bar.body() <= pattern.average(CandleSetting::BODY_DOJI, 0)
        && bar.upper_shadow() > long_shadow
        && bar.lower_shadow() > long_shadow
        && bar.body_bottom() <= middle + near
        && bar.body_top() >= middle - near
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlRickshawman;

pub type CdlRickshawmanStream = BarStream<CdlRickshawman, 4, 1>;

impl Kernel<4, 1> for CdlRickshawman {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_rickshawman"];

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

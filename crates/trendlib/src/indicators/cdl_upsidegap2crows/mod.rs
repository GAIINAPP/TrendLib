use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_upsidegap2crows";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long white bar, then two black ones above it, the second swallowing the first without closing the gap
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    if first.is_white()
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && middle.is_black()
        && middle.body() <= pattern.average(CandleSetting::BODY_SHORT, 1)
        && middle.body_gaps_above(first)
        && bar.is_black()
        && bar.open > middle.open
        && bar.close < middle.close
        && bar.close > first.close
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlUpsidegap2crows;

pub type CdlUpsidegap2crowsStream = BarStream<CdlUpsidegap2crows, 4, 1>;

impl Kernel<4, 1> for CdlUpsidegap2crows {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_upsidegap2crows"];

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

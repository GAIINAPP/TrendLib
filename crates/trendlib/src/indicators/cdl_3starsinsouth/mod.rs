use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3starsinsouth";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three black bars, each smaller than the last and giving back less ground
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    if first.is_black()
        && middle.is_black()
        && bar.is_black()
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && first.lower_shadow() > pattern.average(CandleSetting::SHADOW_LONG, 2)
        && middle.body() < first.body()
        && middle.open > first.close
        && middle.open <= first.high
        && middle.low < first.close
        && middle.low >= first.low
        && middle.lower_shadow() > pattern.average(CandleSetting::SHADOW_VERY_SHORT, 1)
        && bar.body() < pattern.average(CandleSetting::BODY_SHORT, 0)
        && bar.lower_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, 0)
        && bar.upper_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, 0)
        && bar.low > middle.low
        && bar.high < middle.high
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3starsinsouth;

pub type Cdl3starsinsouthStream = BarStream<Cdl3starsinsouth, 4, 1>;

impl Kernel<4, 1> for Cdl3starsinsouth {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3starsinsouth"];

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

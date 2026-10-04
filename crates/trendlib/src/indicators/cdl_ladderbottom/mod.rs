use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_ladderbottom";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 14;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three black bars stepping down, a fourth with a wick above, then a white one clearing it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let fourth = pattern.back(1);
    let third = pattern.back(2);
    let second = pattern.back(3);
    let first = pattern.back(4);
    if first.is_black()
        && second.is_black()
        && third.is_black()
        && first.open > second.open
        && second.open > third.open
        && first.close > second.close
        && second.close > third.close
        && fourth.is_black()
        && fourth.upper_shadow() > pattern.average(CandleSetting::SHADOW_VERY_SHORT, 1)
        && bar.is_white()
        && bar.open > fourth.open
        && bar.close > fourth.high
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlLadderbottom;

pub type CdlLadderbottomStream = BarStream<CdlLadderbottom, 4, 1>;

impl Kernel<4, 1> for CdlLadderbottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_ladderbottom"];

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

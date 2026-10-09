use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_identical3crows";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three black bars each opening where the last one closed and closing lower
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    let closes_low = |candle: Candle, back: usize| {
        candle.is_black()
            && candle.lower_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, back)
    };
    let opens_at = |open: f64, close: f64, back: usize| {
        let equal = pattern.average(CandleSetting::EQUAL, back);
        open <= close + equal && open >= close - equal
    };
    if closes_low(first, 2)
        && closes_low(second, 1)
        && closes_low(bar, 0)
        && first.close > second.close
        && second.close > bar.close
        && opens_at(second.open, first.close, 2)
        && opens_at(bar.open, second.close, 1)
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlIdentical3crows;

pub type CdlIdentical3crowsStream = BarStream<CdlIdentical3crows, 4, 1>;

impl Kernel<4, 1> for CdlIdentical3crows {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_identical3crows"];

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

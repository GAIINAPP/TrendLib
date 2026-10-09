use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3blackcrows";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 13;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three black bars each opening inside the last body and closing lower, after a white one
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    let before = pattern.back(3);
    let closes_low = |candle: Candle, back: usize| {
        candle.is_black()
            && candle.lower_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, back)
    };
    if before.is_white()
        && closes_low(first, 2)
        && closes_low(second, 1)
        && closes_low(bar, 0)
        && second.open < first.open
        && second.open > first.close
        && bar.open < second.open
        && bar.open > second.close
        && before.high > first.close
        && first.close > second.close
        && second.close > bar.close
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3blackcrows;

pub type Cdl3blackcrowsStream = BarStream<Cdl3blackcrows, 4, 1>;

impl Kernel<4, 1> for Cdl3blackcrows {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3blackcrows"];

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

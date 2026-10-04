use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3whitesoldiers";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three white bars each opening inside the last body and closing higher without shortening
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    let closes_high = |candle: Candle, back: usize| {
        candle.is_white()
            && candle.upper_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, back)
    };
    let opens_within = |open: f64, previous: Candle, back: usize| {
        open > previous.open && open <= previous.close + pattern.average(CandleSetting::NEAR, back)
    };
    let keeps_up = |candle: Candle, previous: Candle, back: usize| {
        candle.body() > previous.body() - pattern.average(CandleSetting::FAR, back)
    };
    if closes_high(first, 2)
        && closes_high(second, 1)
        && closes_high(bar, 0)
        && bar.close > second.close
        && second.close > first.close
        && opens_within(second.open, first, 2)
        && opens_within(bar.open, second, 1)
        && keeps_up(second, first, 2)
        && keeps_up(bar, second, 1)
        && bar.body() > pattern.average(CandleSetting::BODY_SHORT, 0)
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3whitesoldiers;

pub type Cdl3whitesoldiersStream = BarStream<Cdl3whitesoldiers, 4, 1>;

impl Kernel<4, 1> for Cdl3whitesoldiers {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3whitesoldiers"];

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

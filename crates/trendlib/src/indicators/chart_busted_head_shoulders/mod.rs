use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::BustState;
use crate::indicators::chart_head_shoulders::{ChartHeadShoulders, Params as Base};

pub const NAME: &str = "chart_busted_head_shoulders";

pub const REVERSAL_BARS_DEFAULT: usize = 10;
pub const REVERSAL_BARS_MIN: usize = 1;
pub const REVERSAL_BARS_MAX: usize = 100_000;
pub const REVERSAL_PCT_DEFAULT: f64 = 0.05;
pub const REVERSAL_PCT_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub reversal_bars: usize,
    pub reversal_pct: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            reversal_bars: REVERSAL_BARS_DEFAULT,
            reversal_pct: REVERSAL_PCT_DEFAULT,
        }
    }
}

pub type State = BustState<<ChartHeadShoulders as Kernel<3, 1>>::State>;

#[derive(Clone, Copy, Debug)]
pub struct ChartBustedHeadShoulders;

pub type ChartBustedHeadShouldersStream = BarStream<ChartBustedHeadShoulders, 3, 1>;

impl Kernel<3, 1> for ChartBustedHeadShoulders {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_busted_head_shoulders"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "reversal_bars",
            params.reversal_bars,
            REVERSAL_BARS_MIN,
            REVERSAL_BARS_MAX,
        )?;
        at_least(NAME, "reversal_pct", params.reversal_pct, REVERSAL_PCT_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        <ChartHeadShoulders as Kernel<3, 1>>::lookback(&Base::default())
    }

    fn state(params: &Params) -> State {
        BustState::new(
            <ChartHeadShoulders as Kernel<3, 1>>::state(&Base::default()),
            false,
            params.reversal_bars,
            params.reversal_pct,
            <ChartHeadShoulders as Kernel<3, 1>>::lookback(&Base::default()),
        )
    }
}

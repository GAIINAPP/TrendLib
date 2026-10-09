use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "darvas_box";

pub const CONFIRM_BARS_DEFAULT: usize = 3;
pub const CONFIRM_BARS_MIN: usize = 1;
pub const CONFIRM_BARS_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub confirm_bars: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            confirm_bars: CONFIRM_BARS_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    confirm: usize,
    lookback: usize,
    bars: usize,
    top: f64,
    bottom: Option<(f64, usize)>,
    /// The box in force until price leaves it.
    boxed: Option<[f64; 2]>,
    shown: [f64; 2],
}

impl State {
    fn restart(&mut self, high: f64) {
        self.top = high;
        self.bottom = None;
        self.boxed = None;
    }

    fn advance(&mut self, [high, low]: [f64; 2]) -> Option<[f64; 2]> {
        let now = self.bars;
        self.bars += 1;
        match self.boxed {
            Some([top, bottom]) => {
                if high > top || low < bottom {
                    self.restart(high);
                }
            }
            None => {
                if high > self.top {
                    self.restart(high);
                } else if self.bottom.is_none_or(|(bottom, _)| low < bottom) {
                    self.bottom = Some((low, now));
                }
                if let Some((bottom, at)) = self.bottom
                    && now - at >= self.confirm
                {
                    self.boxed = Some([self.top, bottom]);
                    self.shown = [self.top, bottom];
                }
            }
        }
        (now >= self.lookback).then_some(self.shown)
    }
}

impl Step<2, 2> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DarvasBox;

pub type DarvasBoxStream = BarStream<DarvasBox, 2, 2>;

impl Kernel<2, 2> for DarvasBox {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 2] = ["darvas_top", "darvas_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "confirm_bars",
            params.confirm_bars,
            CONFIRM_BARS_MIN,
            CONFIRM_BARS_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.confirm_bars + 1
    }

    fn state(params: &Params) -> State {
        State {
            confirm: params.confirm_bars,
            lookback: params.confirm_bars + 1,
            bars: 0,
            top: f64::NEG_INFINITY,
            bottom: None,
            boxed: None,
            shown: [f64::NAN; 2],
        }
    }
}

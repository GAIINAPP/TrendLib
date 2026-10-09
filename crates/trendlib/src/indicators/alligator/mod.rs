use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "alligator";

pub const JAW_PERIOD_DEFAULT: usize = 13;
pub const JAW_PERIOD_MIN: usize = 1;
pub const JAW_PERIOD_MAX: usize = 100_000;
pub const JAW_SHIFT_DEFAULT: usize = 8;
pub const JAW_SHIFT_MIN: usize = 1;
pub const JAW_SHIFT_MAX: usize = 100_000;
pub const TEETH_PERIOD_DEFAULT: usize = 8;
pub const TEETH_PERIOD_MIN: usize = 1;
pub const TEETH_PERIOD_MAX: usize = 100_000;
pub const TEETH_SHIFT_DEFAULT: usize = 5;
pub const TEETH_SHIFT_MIN: usize = 1;
pub const TEETH_SHIFT_MAX: usize = 100_000;
pub const LIPS_PERIOD_DEFAULT: usize = 5;
pub const LIPS_PERIOD_MIN: usize = 1;
pub const LIPS_PERIOD_MAX: usize = 100_000;
pub const LIPS_SHIFT_DEFAULT: usize = 3;
pub const LIPS_SHIFT_MIN: usize = 1;
pub const LIPS_SHIFT_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub jaw_period: usize,
    pub jaw_shift: usize,
    pub teeth_period: usize,
    pub teeth_shift: usize,
    pub lips_period: usize,
    pub lips_shift: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            jaw_period: JAW_PERIOD_DEFAULT,
            jaw_shift: JAW_SHIFT_DEFAULT,
            teeth_period: TEETH_PERIOD_DEFAULT,
            teeth_shift: TEETH_SHIFT_DEFAULT,
            lips_period: LIPS_PERIOD_DEFAULT,
            lips_shift: LIPS_SHIFT_DEFAULT,
        }
    }
}

/// A smoothed median drawn `shift` bars ahead of the bar it was computed on.
#[derive(Clone, Debug)]
struct Line {
    average: Ema,
    ahead: Box<[f64]>,
    head: usize,
}

impl Line {
    fn new(period: usize, shift: usize) -> Self {
        Self {
            // Wilder's smoothing written as an exponential step with k = 1/n,
            // which leaves a flat median exactly flat.
            average: Ema::with_smoothing(period, 1.0 / period as f64),
            ahead: vec![f64::NAN; shift + 1].into_boxed_slice(),
            head: 0,
        }
    }

    fn push(&mut self, median: f64) -> f64 {
        self.ahead[self.head] = self.average.push(median).unwrap_or(f64::NAN);
        self.head = (self.head + 1) % self.ahead.len();
        // The slot about to be overwritten is the one written `shift` bars ago.
        self.ahead[self.head]
    }
}

#[derive(Clone, Debug)]
pub struct State {
    jaw: Line,
    teeth: Line,
    lips: Line,
    lookback: usize,
    bars: usize,
}

impl State {
    pub(crate) fn new(params: &Params) -> Self {
        let [jaw, teeth, lips] = smoothed_lines(params);
        Self {
            jaw: Line::new(jaw.0, jaw.1),
            teeth: Line::new(teeth.0, teeth.1),
            lips: Line::new(lips.0, lips.1),
            lookback: Alligator::lookback(params),
            bars: 0,
        }
    }

    pub(crate) fn advance(&mut self, bar: [f64; 2]) -> Option<[f64; 3]> {
        let median = (bar[0] + bar[1]) / 2.0;
        let lines = [
            self.jaw.push(median),
            self.teeth.push(median),
            self.lips.push(median),
        ];
        let now = self.bars;
        self.bars += 1;
        (now >= self.lookback).then_some(lines)
    }
}

impl Step<2, 3> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 3]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 3]> {
        self.clone().advance(bar)
    }
}

fn smoothed_lines(params: &Params) -> [(usize, usize); 3] {
    [
        (params.jaw_period, params.jaw_shift),
        (params.teeth_period, params.teeth_shift),
        (params.lips_period, params.lips_shift),
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct Alligator;

pub type AlligatorStream = BarStream<Alligator, 2, 3>;

impl Kernel<2, 3> for Alligator {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 3] = ["alligator_jaw", "alligator_teeth", "alligator_lips"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "jaw_period",
            params.jaw_period,
            JAW_PERIOD_MIN,
            JAW_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "jaw_shift",
            params.jaw_shift,
            JAW_SHIFT_MIN,
            JAW_SHIFT_MAX,
        )?;
        whole(
            NAME,
            "teeth_period",
            params.teeth_period,
            TEETH_PERIOD_MIN,
            TEETH_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "teeth_shift",
            params.teeth_shift,
            TEETH_SHIFT_MIN,
            TEETH_SHIFT_MAX,
        )?;
        whole(
            NAME,
            "lips_period",
            params.lips_period,
            LIPS_PERIOD_MIN,
            LIPS_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "lips_shift",
            params.lips_shift,
            LIPS_SHIFT_MIN,
            LIPS_SHIFT_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        smoothed_lines(params)
            .iter()
            .map(|(period, shift)| period - 1 + shift)
            .max()
            .unwrap_or(0)
    }

    fn state(params: &Params) -> State {
        State::new(params)
    }
}

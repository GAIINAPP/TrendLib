use crate::core::error::TlError;
use crate::core::hilbert::{DominantPhase, Hilbert, PRIMED_LATE, Recent};
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "ht_trendmode";

/// Taken from the oracle.
pub const LOOKBACK: usize = 63;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    cycle: Hilbert,
    phase: DominantPhase,
    prices: Recent,
    averaged: [f64; 3],
    previous_degrees: f64,
    waves: (f64, f64),
    bars_since_crossing: f64,
    seen: usize,
}

const DEGREE: f64 = std::f64::consts::PI / 180.0;
const LEAD: f64 = 45.0;

/// How far the series may sit from its own trendline before the reading is
/// called a trend whatever else says otherwise.
const AWAY: f64 = 0.015;

impl State {
    fn blend(&self, mean: f64) -> f64 {
        let inner = 4.0f64.mul_add(mean, 3.0 * self.averaged[0]);
        (2.0f64.mul_add(self.averaged[1], inner) + self.averaged[2]) / 10.0
    }

    fn judge(&mut self, degrees: f64, smooth_period: f64, trendline: f64, smoothed: f64) -> f64 {
        let (previous_sine, previous_lead) = self.waves;
        let sine = (degrees * DEGREE).sin();
        let lead = ((degrees + LEAD) * DEGREE).sin();
        self.waves = (sine, lead);
        let crossed = (sine > lead && previous_sine <= previous_lead)
            || (sine < lead && previous_sine >= previous_lead);
        let mut trending = true;
        if crossed {
            self.bars_since_crossing = 0.0;
            trending = false;
        }
        self.bars_since_crossing += 1.0;
        if self.bars_since_crossing < 0.5 * smooth_period {
            trending = false;
        }
        // A phase advancing at about one cycle's worth per bar is the cycle
        // itself, not a trend.
        let turned = degrees - self.previous_degrees;
        let per_bar = 360.0 / smooth_period;
        if smooth_period != 0.0 && turned > 0.67 * per_bar && turned < 1.5 * per_bar {
            trending = false;
        }
        if trendline != 0.0 && ((smoothed - trendline) / trendline).abs() >= AWAY {
            trending = true;
        }
        if trending { 1.0 } else { 0.0 }
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.prices.push(bar[0]);
        let reading = self.cycle.push(bar[0]);
        self.seen += 1;
        let reading = reading?;
        let previous_degrees = self.previous_degrees;
        let degrees = self.phase.push(&self.cycle, reading.smooth_period);
        self.previous_degrees = degrees;
        let mean = self.prices.mean((reading.smooth_period + 0.5) as usize);
        let trendline = self.blend(mean);
        self.averaged.rotate_right(1);
        self.averaged[0] = mean;
        let smoothed = self.cycle.history().back(0);
        self.previous_degrees = previous_degrees;
        let value = self.judge(degrees, reading.smooth_period, trendline, smoothed);
        self.previous_degrees = degrees;
        (self.seen > LOOKBACK).then_some([value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HtTrendmode;

pub type HtTrendmodeStream = BarStream<HtTrendmode, 1, 1>;

impl Kernel<1, 1> for HtTrendmode {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ht_trendmode"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(_params: &Params) -> State {
        State {
            cycle: Hilbert::new(PRIMED_LATE),
            phase: DominantPhase::default(),
            prices: Recent::default(),
            averaged: [0.0; 3],
            previous_degrees: 0.0,
            waves: (0.0, 0.0),
            bars_since_crossing: 0.0,
            seen: 0,
        }
    }
}

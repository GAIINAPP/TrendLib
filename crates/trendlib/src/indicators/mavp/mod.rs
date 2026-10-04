use std::collections::BTreeMap;

use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage};

pub const NAME: &str = "mavp";

pub const MIN_PERIOD_DEFAULT: usize = 2;
pub const MIN_PERIOD_MIN: usize = 1;
pub const MIN_PERIOD_MAX: usize = 30;
pub const MAX_PERIOD_DEFAULT: usize = 30;
pub const MAX_PERIOD_MIN: usize = 2;
pub const MAX_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub min_period: usize,
    pub max_period: usize,
    pub ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            min_period: MIN_PERIOD_DEFAULT,
            max_period: MAX_PERIOD_DEFAULT,
            ma_type: MaType::Sma,
        }
    }
}

/// One average, and the value it produced on the newest bar.
#[derive(Clone, Debug)]
struct Tracked {
    average: MovingAverage,
    latest: Option<f64>,
}

/// The whole series so far.
///
/// This is the one indicator that keeps it. A period asked for the first time
/// at bar five thousand still has to be the average over the bars before it,
/// so the bars have to still be there; TA-Lib reaches back into its input
/// array for the same reason.
#[derive(Clone, Debug)]
pub struct State {
    seen_values: Vec<f64>,
    tracked: BTreeMap<usize, Tracked>,
    min_period: usize,
    max_period: usize,
    ma_type: MaType,
    warm_up: usize,
}

impl State {
    /// The period this bar asks for, held inside the documented range. The
    /// fraction is dropped rather than rounded, as TA-Lib drops it.
    fn wanted(&self, asked: f64) -> usize {
        let minimum = self.min_period as f64;
        let maximum = self.max_period as f64;
        let held = if asked < minimum {
            self.min_period
        } else if asked > maximum {
            self.max_period
        } else {
            asked as usize
        };
        // `min_period` may be set above `max_period`, which would ask for an
        // average longer than the warm-up covers; the longest is the most
        // this indicator can answer with.
        held.max(1).min(self.max_period)
    }

    /// The average for `period`, started where TA-Lib starts it: far enough
    /// back to have its own warm-up behind this indicator's first row, and no
    /// further, which is what a recursive average's seed depends on.
    fn tracked(&mut self, period: usize) -> &Tracked {
        if !self.tracked.contains_key(&period) {
            let begins = self.warm_up - self.ma_type.lookback(period);
            let mut average = self.ma_type.state(period);
            let mut latest = None;
            for value in &self.seen_values[begins..] {
                latest = average.push(*value);
            }
            self.tracked.insert(period, Tracked { average, latest });
        }
        &self.tracked[&period]
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.seen_values.push(bar[0]);
        // Every average already running has to take the bar, whether or not
        // this one is asked for.
        for tracked in self.tracked.values_mut() {
            tracked.latest = tracked.average.push(bar[0]);
        }
        if self.seen_values.len() <= self.warm_up {
            return None;
        }
        let period = self.wanted(bar[1]);
        self.tracked(period).latest.map(|value| [value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mavp;

pub type MavpStream = BarStream<Mavp, 2, 1>;

impl Kernel<2, 1> for Mavp {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "periods"];
    const OUTPUTS: [&'static str; 1] = ["mavp"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(MIN_PERIOD_MIN..=MIN_PERIOD_MAX).contains(&params.min_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "min_period",
                params.min_period,
                MIN_PERIOD_MIN,
                MIN_PERIOD_MAX,
            ));
        }
        if !(MAX_PERIOD_MIN..=MAX_PERIOD_MAX).contains(&params.max_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "max_period",
                params.max_period,
                MAX_PERIOD_MIN,
                MAX_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    // The longest average anyone could ask for decides the first row.
    fn lookback(params: &Params) -> usize {
        params.ma_type.lookback(params.max_period)
    }

    fn state(params: &Params) -> State {
        State {
            seen_values: Vec::new(),
            tracked: BTreeMap::new(),
            min_period: params.min_period,
            max_period: params.max_period,
            ma_type: params.ma_type,
            warm_up: Self::lookback(params),
        }
    }
}

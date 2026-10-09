use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage, RollingWindow};

pub const NAME: &str = "bbands";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const NBDEV_DEFAULT: f64 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub nbdev_up: f64,
    pub nbdev_dn: f64,
    pub ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            nbdev_up: NBDEV_DEFAULT,
            nbdev_dn: NBDEV_DEFAULT,
            ma_type: MaType::Sma,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    middle: MovingAverage,
    window: RollingWindow,
    nbdev_up: f64,
    nbdev_dn: f64,
}

/// Population standard deviation about the window's own simple mean, which is
/// what TA-Lib's STDDEV computes whatever `ma_type` the middle band uses. The
/// width therefore does not follow the middle band when the band is, say, an
/// EMA; `doc.md` says so.
fn deviation(window: &RollingWindow, mean: f64, values: impl Iterator<Item = f64>) -> f64 {
    let period = window.period() as f64;
    let variance = values
        .map(|value| {
            let distance = value - mean;
            distance * distance
        })
        .sum::<f64>()
        / period;
    variance.sqrt()
}

fn bands(middle: f64, sigma: f64, nbdev_up: f64, nbdev_dn: f64) -> [f64; 3] {
    [middle + sigma * nbdev_up, middle, middle - sigma * nbdev_dn]
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        // The deviation window fills first for every average but `sma`, and it
        // has to keep taking bars while the middle band is still warming up.
        let full = self.window.push(bar[0]);
        let middle = self.middle.push(bar[0])?;
        if !full {
            return None;
        }
        let mean = self.window.exact_mean();
        let sigma = deviation(&self.window, mean, self.window.values());
        Some(bands(middle, sigma, self.nbdev_up, self.nbdev_dn))
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 3]> {
        let middle = self.middle.preview(bar[0])?;
        if !self.window.preview_is_full() {
            return None;
        }
        let mean = self.window.preview_mean(bar[0]);
        let sigma = deviation(&self.window, mean, self.window.preview_values(bar[0]));
        Some(bands(middle, sigma, self.nbdev_up, self.nbdev_dn))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bbands;

pub type BbandsStream = BarStream<Bbands, 1, 3>;

impl Kernel<1, 3> for Bbands {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = ["bbands_upper", "bbands_middle", "bbands_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        for (name, value) in [("nbdev_up", params.nbdev_up), ("nbdev_dn", params.nbdev_dn)] {
            if !value.is_finite() {
                return Err(TlError::param_out_of_range(
                    NAME,
                    name,
                    value,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                ));
            }
        }
        Ok(())
    }

    // The deviation is ready after period - 1 bars, which no moving average
    // beats, so the middle band alone decides when the first row appears.
    fn lookback(params: &Params) -> usize {
        params.ma_type.lookback(params.period)
    }

    fn state(params: &Params) -> State {
        State {
            middle: params.ma_type.state(params.period),
            window: RollingWindow::new(params.period),
            nbdev_up: params.nbdev_up,
            nbdev_dn: params.nbdev_dn,
        }
    }
}

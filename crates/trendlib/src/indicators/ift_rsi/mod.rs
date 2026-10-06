use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::PandasEwm;

pub const NAME: &str = "ift_rsi";

pub const RSI_PERIOD_DEFAULT: usize = 5;
pub const RSI_PERIOD_MIN: usize = 2;
pub const RSI_PERIOD_MAX: usize = 100_000;
pub const WMA_PERIOD_DEFAULT: usize = 9;
pub const WMA_PERIOD_MIN: usize = 2;
pub const WMA_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub rsi_period: usize,
    pub wma_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            rsi_period: RSI_PERIOD_DEFAULT,
            wma_period: WMA_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    gain: PandasEwm,
    loss: PandasEwm,
    previous: Option<f64>,
    /// The last `wma_period` rescaled RSI values, oldest first after `head`.
    scaled: Box<[f64]>,
    head: usize,
    bars: usize,
}

impl State {
    fn advance(&mut self, close: f64) -> Option<f64> {
        let change = self.previous.map_or(f64::NAN, |previous| close - previous);
        self.previous = Some(close);
        let gain = self.gain.push(if change.is_nan() {
            f64::NAN
        } else {
            change.max(0.0)
        });
        let loss = self.loss.push(if change.is_nan() {
            f64::NAN
        } else {
            (-change).max(0.0)
        });
        let rsi = 100.0 - 100.0 / (1.0 + gain / loss);
        let len = self.scaled.len();
        self.scaled[self.head] = 0.1 * (rsi - 50.0);
        self.head = (self.head + 1) % len;
        self.bars += 1;
        if self.bars <= len {
            return None;
        }
        // Weights 1 for the oldest to `len` for the newest, over their sum.
        let weighted: f64 = (0..len)
            .map(|k| (k + 1) as f64 * self.scaled[(self.head + k) % len])
            .sum();
        let average = weighted / ((len * (len + 1)) as f64 / 2.0);
        let squared = average * average;
        Some((squared - 1.0) / (squared + 1.0))
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.advance(bar[0]).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.clone().advance(bar[0]).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IftRsi;

pub type IftRsiStream = BarStream<IftRsi, 1, 1>;

impl Kernel<1, 1> for IftRsi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["ift_rsi"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "rsi_period",
            params.rsi_period,
            RSI_PERIOD_MIN,
            RSI_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "wma_period",
            params.wma_period,
            WMA_PERIOD_MIN,
            WMA_PERIOD_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.wma_period
    }

    fn state(params: &Params) -> State {
        State {
            gain: PandasEwm::with_alpha(1.0 / params.rsi_period as f64),
            loss: PandasEwm::with_alpha(1.0 / params.rsi_period as f64),
            previous: None,
            scaled: vec![f64::NAN; params.wma_period].into_boxed_slice(),
            head: 0,
            bars: 0,
        }
    }
}

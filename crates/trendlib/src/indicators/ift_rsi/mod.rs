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
        // The inverse Fisher transform, (e^2x - 1) / (e^2x + 1), is tanh(x).
        // It keeps the sign of the RSI's swing about 50, so an overbought and
        // an oversold reading come out on opposite sides of zero, and it
        // saturates at +/-1 where the exponential form would overflow.
        Some(average.tanh())
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

#[cfg(test)]
mod tests {
    use super::{IftRsi, Params};
    use crate::core::kernel::Kernel;

    fn ift(close: &[f64]) -> Vec<f64> {
        let [out] = IftRsi::batch([close], &Params::default()).unwrap();
        out
    }

    fn wavy(start: f64, step: f64) -> Vec<f64> {
        (0..80)
            .map(|i| start + step * i as f64 + (i as f64 * 0.7).sin() * 0.4)
            .collect()
    }

    #[test]
    fn a_rise_reads_near_plus_one_and_a_fall_near_minus_one() {
        let rise = ift(&wavy(100.0, 1.0));
        let fall = ift(&wavy(200.0, -1.0));
        assert!(*rise.last().unwrap() > 0.99, "rise: {:?}", rise.last());
        assert!(*fall.last().unwrap() < -0.99, "fall: {:?}", fall.last());
    }

    #[test]
    fn mirrored_prices_give_the_negated_reading() {
        // Reflecting prices turns RSI r into 100 - r, so the transform, an odd
        // function of the rescaled RSI, must flip sign bar for bar.
        let close = wavy(150.0, 0.2);
        let mirror: Vec<f64> = close.iter().map(|c| 300.0 - c).collect();
        let (a, b) = (ift(&close), ift(&mirror));
        for (row, (x, y)) in a.iter().zip(&b).enumerate() {
            if x.is_nan() {
                assert!(y.is_nan(), "row {row}");
                continue;
            }
            assert!((x + y).abs() < 1e-12, "row {row}: {x} vs {y}");
            assert!(x.abs() < 1.0);
        }
    }
}

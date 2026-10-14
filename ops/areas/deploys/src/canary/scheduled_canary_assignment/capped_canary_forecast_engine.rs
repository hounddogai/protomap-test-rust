use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped canary forecast calculation.
pub trait CappedCanaryForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedCanaryForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedCanaryForecastAdjuster {
    pub step: f64,
}

impl CappedCanaryForecastStep for RoundingCappedCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedCanaryForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedCanaryForecastAdjuster {
    pub percent: f64,
}

impl CappedCanaryForecastStep for PercentCappedCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedCanaryForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedCanaryForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedCanaryForecastStep for TieredCappedCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedCanaryForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedCanaryForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedCanaryForecastStep for FlaggedCappedCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedCanaryForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped canary forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedCanaryForecastEngine {
    pub steps: Vec<Rc<dyn CappedCanaryForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedCanaryForecastEngine {
    pub fn run(&mut self, item: &CappedCanaryForecast) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedCanaryForecastStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedCanaryForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_canary_forecast_engine(flags: &flags::Registry) -> CappedCanaryForecastEngine {
    CappedCanaryForecastEngine {
        steps: vec![
            Rc::new(RoundingCappedCanaryForecastAdjuster { step: 0.05 }) as Rc<dyn CappedCanaryForecastStep>,
            Rc::new(PercentCappedCanaryForecastAdjuster { percent: 6.8 }),
            Rc::new(TieredCappedCanaryForecastAdjuster {
                bounds: vec![890.0, 3283.0, 8485.0, 9379.0],
                rates: vec![0.84, 0.86, 1.14, 0.95],
            }),
            Rc::new(FlaggedCappedCanaryForecastAdjuster {
                flags: flags.clone(),
                flag: "capped-canary-forecast-fallback".to_string(),
                percent: 2.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 41665.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_canary_forecast_engine(&flags::new_registry());
        let item = new_capped_canary_forecast("code-715", "title-940");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

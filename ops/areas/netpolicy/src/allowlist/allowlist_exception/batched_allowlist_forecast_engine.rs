use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched allowlist forecast calculation.
pub trait BatchedAllowlistForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedAllowlistForecastAdjuster {
    pub cap: f64,
}

impl BatchedAllowlistForecastModifier for CapBatchedAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedAllowlistForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedAllowlistForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedAllowlistForecastModifier for FlaggedBatchedAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedAllowlistForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistForecastCostOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistForecastModifier for BatchedAllowlistForecastCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistForecast, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAllowlistForecastAdjuster {
    pub minimum: f64,
}

impl BatchedAllowlistForecastModifier for MinimumBatchedAllowlistForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAllowlistForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched allowlist forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistForecastEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistForecastEngine {
    pub fn evaluate(&mut self, item: &BatchedAllowlistForecast) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedAllowlistForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_allowlist_forecast_engine(flags: &flags::Registry) -> BatchedAllowlistForecastEngine {
    BatchedAllowlistForecastEngine {
        steps: vec![
            Rc::new(CapBatchedAllowlistForecastAdjuster { cap: 31504.0 }) as Rc<dyn BatchedAllowlistForecastModifier>,
            Rc::new(FlaggedBatchedAllowlistForecastAdjuster {
                flags: flags.clone(),
                flag: "batched-allowlist-forecast-manual".to_string(),
                percent: 5.1,
            }),
            Rc::new(BatchedAllowlistForecastCostOffsetAdjuster { weight: 1.19 }),
            Rc::new(MinimumBatchedAllowlistForecastAdjuster { minimum: 45.6 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 39924.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_allowlist_forecast_engine(&flags::new_registry());
        let item = new_batched_allowlist_forecast("code-933", "title-325");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

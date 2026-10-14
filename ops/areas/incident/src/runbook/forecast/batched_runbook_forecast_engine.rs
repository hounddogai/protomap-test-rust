use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched runbook forecast calculation.
pub trait BatchedRunbookForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRunbookForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRunbookForecastAdjuster {
    pub cap: f64,
}

impl BatchedRunbookForecastStep for CapBatchedRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRunbookForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRunbookForecastAdjuster {
    pub minimum: f64,
}

impl BatchedRunbookForecastStep for MinimumBatchedRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRunbookForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRunbookForecastStep for BatchedRunbookForecastGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRunbookForecast, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the batched runbook forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRunbookForecastEngine {
    pub steps: Vec<Rc<dyn BatchedRunbookForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRunbookForecastEngine {
    pub fn price(&mut self, item: &BatchedRunbookForecast) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRunbookForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRunbookForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_runbook_forecast_engine() -> BatchedRunbookForecastEngine {
    BatchedRunbookForecastEngine {
        steps: vec![
            Rc::new(CapBatchedRunbookForecastAdjuster { cap: 14873.0 }) as Rc<dyn BatchedRunbookForecastStep>,
            Rc::new(MinimumBatchedRunbookForecastAdjuster { minimum: 2.8 }),
            Rc::new(BatchedRunbookForecastGraceDaysOffsetAdjuster { weight: 1.15 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 11300.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_runbook_forecast_engine();
        let item = new_batched_runbook_forecast("code-642", "label-938");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

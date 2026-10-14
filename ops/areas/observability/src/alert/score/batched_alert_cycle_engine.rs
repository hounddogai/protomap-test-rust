use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched alert cycle calculation.
pub trait BatchedAlertCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAlertCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedAlertCycleAdjuster {
    pub step: f64,
}

impl BatchedAlertCycleFactor for RoundingBatchedAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedAlertCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAlertCycleFactor for BatchedAlertCycleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAlertCycle, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAlertCycleAdjuster {
    pub minimum: f64,
}

impl BatchedAlertCycleFactor for MinimumBatchedAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAlertCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedAlertCycleAdjuster {
    pub percent: f64,
}

impl BatchedAlertCycleFactor for PercentBatchedAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedAlertCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched alert cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAlertCycleEngine {
    pub steps: Vec<Rc<dyn BatchedAlertCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAlertCycleEngine {
    pub fn evaluate(&mut self, item: &BatchedAlertCycle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[BatchedAlertCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_alert_cycle_engine() -> BatchedAlertCycleEngine {
    BatchedAlertCycleEngine {
        steps: vec![
            Rc::new(RoundingBatchedAlertCycleAdjuster { step: 5.0 }) as Rc<dyn BatchedAlertCycleFactor>,
            Rc::new(BatchedAlertCycleStartsAtOffsetAdjuster { weight: 0.15 }),
            Rc::new(MinimumBatchedAlertCycleAdjuster { minimum: 5.5 }),
            Rc::new(PercentBatchedAlertCycleAdjuster { percent: 10.3 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 23397.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_alert_cycle_engine();
        let item = new_batched_alert_cycle("code-905", "label-944");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

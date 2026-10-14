use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted rollout record calculation.
pub trait WeightedRolloutRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRolloutRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRolloutRecordAdjuster {
    pub percent: f64,
}

impl WeightedRolloutRecordStep for PercentWeightedRolloutRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRolloutRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRolloutRecordAdjuster {
    pub cap: f64,
}

impl WeightedRolloutRecordStep for CapWeightedRolloutRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRolloutRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedRolloutRecordAdjuster {
    pub minimum: f64,
}

impl WeightedRolloutRecordStep for MinimumWeightedRolloutRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedRolloutRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted rollout record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRolloutRecordEngine {
    pub steps: Vec<Rc<dyn WeightedRolloutRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRolloutRecordEngine {
    pub fn compute(&mut self, item: &WeightedRolloutRecord) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRolloutRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_rollout_record_engine() -> WeightedRolloutRecordEngine {
    WeightedRolloutRecordEngine {
        steps: vec![
            Rc::new(PercentWeightedRolloutRecordAdjuster { percent: 8.5 }) as Rc<dyn WeightedRolloutRecordStep>,
            Rc::new(CapWeightedRolloutRecordAdjuster { cap: 41559.0 }),
            Rc::new(MinimumWeightedRolloutRecordAdjuster { minimum: 2.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 75091.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_rollout_record_engine();
        let item = new_weighted_rollout_record("code-694", "name-766");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

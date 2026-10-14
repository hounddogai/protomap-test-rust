use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched federation summary calculation.
pub trait BatchedFederationSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFederationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedFederationSummaryAdjuster {
    pub percent: f64,
}

impl BatchedFederationSummaryStep for PercentBatchedFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedFederationSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl BatchedFederationSummaryStep for BatchedFederationSummaryFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedFederationSummary, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedFederationSummaryAdjuster {
    pub step: f64,
}

impl BatchedFederationSummaryStep for RoundingBatchedFederationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedFederationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched federation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFederationSummaryEngine {
    pub steps: Vec<Rc<dyn BatchedFederationSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFederationSummaryEngine {
    pub fn compute(&mut self, item: &BatchedFederationSummary) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedFederationSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedFederationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_federation_summary_engine() -> BatchedFederationSummaryEngine {
    BatchedFederationSummaryEngine {
        steps: vec![
            Rc::new(PercentBatchedFederationSummaryAdjuster { percent: -22.0 }) as Rc<dyn BatchedFederationSummaryStep>,
            Rc::new(BatchedFederationSummaryFloorValueOffsetAdjuster { weight: 0.09 }),
            Rc::new(RoundingBatchedFederationSummaryAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 53857.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_federation_summary_engine();
        let item = new_batched_federation_summary("code-345", "name-769");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

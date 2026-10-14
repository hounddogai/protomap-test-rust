use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched limit claim calculation.
pub trait BatchedLimitClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedLimitClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedLimitClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedLimitClaimStep for TieredBatchedLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedLimitClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimPortOffsetAdjuster {
    pub weight: f64,
}

impl BatchedLimitClaimStep for BatchedLimitClaimPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedLimitClaim, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedLimitClaimAdjuster {
    pub percent: f64,
}

impl BatchedLimitClaimStep for PercentBatchedLimitClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedLimitClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched limit claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedLimitClaimEngine {
    pub steps: Vec<Rc<dyn BatchedLimitClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedLimitClaimEngine {
    pub fn run(&mut self, item: &BatchedLimitClaim) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedLimitClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_limit_claim_engine() -> BatchedLimitClaimEngine {
    BatchedLimitClaimEngine {
        steps: vec![
            Rc::new(TieredBatchedLimitClaimAdjuster {
                bounds: vec![1142.0, 9571.0],
                rates: vec![1.15, 0.85],
            }) as Rc<dyn BatchedLimitClaimStep>,
            Rc::new(BatchedLimitClaimPortOffsetAdjuster { weight: 1.1 }),
            Rc::new(PercentBatchedLimitClaimAdjuster { percent: 11.3 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 33746.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_limit_claim_engine();
        let item = new_batched_limit_claim("code-700", "label-598");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

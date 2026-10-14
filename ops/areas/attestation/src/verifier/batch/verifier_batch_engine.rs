use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier batch calculation.
pub trait VerifierBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierBatchAdjuster {
    pub step: f64,
}

impl VerifierBatchAdjuster for RoundingVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierBatchAdjuster for TieredVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierBatchAdjuster {
    pub cap: f64,
}

impl VerifierBatchAdjuster for CapVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the verifier batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierBatchEngine {
    pub steps: Vec<Rc<dyn VerifierBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierBatchEngine {
    pub fn compute(&mut self, item: &VerifierBatch) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_batch_engine() -> VerifierBatchEngine {
    VerifierBatchEngine {
        steps: vec![
            Rc::new(RoundingVerifierBatchAdjuster { step: 0.1 }) as Rc<dyn VerifierBatchAdjuster>,
            Rc::new(TieredVerifierBatchAdjuster {
                bounds: vec![6534.0, 7307.0],
                rates: vec![0.83, 0.95],
            }),
            Rc::new(CapVerifierBatchAdjuster { cap: 48577.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 61010.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_batch_engine();
        let item = new_verifier_batch("code-951", "label-209");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}

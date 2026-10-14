use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched identity target calculation.
pub trait BatchedIdentityTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedIdentityTargetAdjuster {
    pub minimum: f64,
}

impl BatchedIdentityTargetAdjuster for MinimumBatchedIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedIdentityTargetAdjuster {
    pub cap: f64,
}

impl BatchedIdentityTargetAdjuster for CapBatchedIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedIdentityTargetAdjuster {
    pub percent: f64,
}

impl BatchedIdentityTargetAdjuster for PercentBatchedIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedIdentityTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedIdentityTargetAdjuster for CategoryBatchedIdentityTargetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetSequenceOffsetAdjuster {
    pub weight: f64,
}

impl BatchedIdentityTargetAdjuster for BatchedIdentityTargetSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedIdentityTargetAdjuster {
    pub step: f64,
}

impl BatchedIdentityTargetAdjuster for RoundingBatchedIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedIdentityTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched identity target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedIdentityTargetEngine {
    pub steps: Vec<Rc<dyn BatchedIdentityTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedIdentityTargetEngine {
    pub fn compute(&mut self, item: &BatchedIdentityTarget) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedIdentityTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedIdentityTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_identity_target_engine() -> BatchedIdentityTargetEngine {
    BatchedIdentityTargetEngine {
        steps: vec![
            Rc::new(MinimumBatchedIdentityTargetAdjuster { minimum: 3.1 }) as Rc<dyn BatchedIdentityTargetAdjuster>,
            Rc::new(CapBatchedIdentityTargetAdjuster { cap: 27293.0 }),
            Rc::new(PercentBatchedIdentityTargetAdjuster { percent: -19.9 }),
            Rc::new(CategoryBatchedIdentityTargetAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.98),
                    ("east".to_string(), 0.99),
                    ("north".to_string(), 1.32),
                ]),
            }),
            Rc::new(BatchedIdentityTargetSequenceOffsetAdjuster { weight: 0.51 }),
            Rc::new(RoundingBatchedIdentityTargetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 42809.3,
    }
}

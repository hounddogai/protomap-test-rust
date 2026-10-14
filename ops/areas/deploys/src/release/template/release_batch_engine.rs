use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release batch calculation.
pub trait ReleaseBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchRevisionOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseBatchModifier for ReleaseBatchRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseBatch, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseBatchAdjuster {
    pub cap: f64,
}

impl ReleaseBatchModifier for CapReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseBatchAdjuster {
    pub step: f64,
}

impl ReleaseBatchModifier for RoundingReleaseBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the release batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseBatchEngine {
    pub steps: Vec<Rc<dyn ReleaseBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseBatchEngine {
    pub fn price(&mut self, item: &ReleaseBatch) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_batch_engine() -> ReleaseBatchEngine {
    ReleaseBatchEngine {
        steps: vec![
            Rc::new(ReleaseBatchRevisionOffsetAdjuster { weight: 0.84 }) as Rc<dyn ReleaseBatchModifier>,
            Rc::new(CapReleaseBatchAdjuster { cap: 20781.0 }),
            Rc::new(RoundingReleaseBatchAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 81464.4,
    }
}

use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched runbook snapshot calculation.
pub trait BatchedRunbookSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRunbookSnapshotAdjuster {
    pub step: f64,
}

impl BatchedRunbookSnapshotModifier for RoundingBatchedRunbookSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedRunbookSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRunbookSnapshotModifier for CategoryBatchedRunbookSnapshotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRunbookSnapshotAdjuster {
    pub percent: f64,
}

impl BatchedRunbookSnapshotModifier for PercentBatchedRunbookSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRunbookSnapshotAdjuster {
    pub minimum: f64,
}

impl BatchedRunbookSnapshotModifier for MinimumBatchedRunbookSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRunbookSnapshotAdjuster {
    pub cap: f64,
}

impl BatchedRunbookSnapshotModifier for CapBatchedRunbookSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRunbookSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched runbook snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRunbookSnapshotEngine {
    pub steps: Vec<Rc<dyn BatchedRunbookSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRunbookSnapshotEngine {
    pub fn price(&mut self, item: &BatchedRunbookSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedRunbookSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_runbook_snapshot_engine() -> BatchedRunbookSnapshotEngine {
    BatchedRunbookSnapshotEngine {
        steps: vec![
            Rc::new(RoundingBatchedRunbookSnapshotAdjuster { step: 0.1 }) as Rc<dyn BatchedRunbookSnapshotModifier>,
            Rc::new(CategoryBatchedRunbookSnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92), ("online".to_string(), 1.01)]),
            }),
            Rc::new(PercentBatchedRunbookSnapshotAdjuster { percent: 5.2 }),
            Rc::new(MinimumBatchedRunbookSnapshotAdjuster { minimum: 8.4 }),
            Rc::new(CapBatchedRunbookSnapshotAdjuster { cap: 48487.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 1606.0,
    }
}

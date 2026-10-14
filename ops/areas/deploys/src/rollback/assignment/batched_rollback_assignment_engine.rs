use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched rollback assignment calculation.
pub trait BatchedRollbackAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRollbackAssignmentAdjuster for BatchedRollbackAssignmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRollbackAssignmentAdjuster {
    pub step: f64,
}

impl BatchedRollbackAssignmentAdjuster for RoundingBatchedRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedRollbackAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedRollbackAssignmentAdjuster for CapBatchedRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRollbackAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRollbackAssignmentAdjuster for FlaggedBatchedRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRollbackAssignmentAdjuster {
    pub percent: f64,
}

impl BatchedRollbackAssignmentAdjuster for PercentBatchedRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRollbackAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedRollbackAssignmentAdjuster for MinimumBatchedRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRollbackAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched rollback assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRollbackAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedRollbackAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRollbackAssignmentEngine {
    pub fn evaluate(&mut self, item: &BatchedRollbackAssignment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRollbackAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRollbackAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_rollback_assignment_engine(flags: &flags::Registry) -> BatchedRollbackAssignmentEngine {
    BatchedRollbackAssignmentEngine {
        steps: vec![
            Rc::new(BatchedRollbackAssignmentAmountOffsetAdjuster { weight: 1.03 })
                as Rc<dyn BatchedRollbackAssignmentAdjuster>,
            Rc::new(RoundingBatchedRollbackAssignmentAdjuster { step: 0.1 }),
            Rc::new(CapBatchedRollbackAssignmentAdjuster { cap: 12357.0 }),
            Rc::new(FlaggedBatchedRollbackAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-rollback-assignment-primary".to_string(),
                percent: -2.2,
            }),
            Rc::new(PercentBatchedRollbackAssignmentAdjuster { percent: -18.3 }),
            Rc::new(MinimumBatchedRollbackAssignmentAdjuster { minimum: 40.8 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 28799.9,
    }
}

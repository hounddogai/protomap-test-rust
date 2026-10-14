use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched slo assignment calculation.
pub trait BatchedSloAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSloAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSloAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedSloAssignmentModifier for CapBatchedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSloAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedSloAssignmentAdjuster {
    pub percent: f64,
}

impl BatchedSloAssignmentModifier for PercentBatchedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedSloAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedSloAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedSloAssignmentModifier for FlaggedBatchedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedSloAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedSloAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedSloAssignmentModifier for MinimumBatchedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedSloAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched slo assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSloAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedSloAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSloAssignmentEngine {
    pub fn evaluate(&mut self, item: &BatchedSloAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedSloAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSloAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_slo_assignment_engine(flags: &flags::Registry) -> BatchedSloAssignmentEngine {
    BatchedSloAssignmentEngine {
        steps: vec![
            Rc::new(CapBatchedSloAssignmentAdjuster { cap: 20104.0 }) as Rc<dyn BatchedSloAssignmentModifier>,
            Rc::new(PercentBatchedSloAssignmentAdjuster { percent: 4.7 }),
            Rc::new(FlaggedBatchedSloAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-slo-assignment-strict".to_string(),
                percent: -9.8,
            }),
            Rc::new(MinimumBatchedSloAssignmentAdjuster { minimum: 29.2 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 76477.8,
    }
}

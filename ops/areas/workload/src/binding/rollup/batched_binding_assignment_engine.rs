use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched binding assignment calculation.
pub trait BatchedBindingAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBindingAssignmentAdjuster {
    pub percent: f64,
}

impl BatchedBindingAssignmentModifier for PercentBatchedBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingAssignmentRateOffsetAdjuster {
    pub weight: f64,
}

impl BatchedBindingAssignmentModifier for BatchedBindingAssignmentRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedBindingAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedBindingAssignmentModifier for FlaggedBatchedBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterBatchedBindingAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBindingAssignmentModifier for ClusterBatchedBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedBindingAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedBindingAssignmentModifier for MinimumBatchedBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedBindingAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched binding assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBindingAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedBindingAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBindingAssignmentEngine {
    pub fn compute(&mut self, item: &BatchedBindingAssignment) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBindingAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBindingAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_binding_assignment_engine(flags: &flags::Registry) -> BatchedBindingAssignmentEngine {
    BatchedBindingAssignmentEngine {
        steps: vec![
            Rc::new(PercentBatchedBindingAssignmentAdjuster { percent: -21.1 })
                as Rc<dyn BatchedBindingAssignmentModifier>,
            Rc::new(BatchedBindingAssignmentRateOffsetAdjuster { weight: 0.45 }),
            Rc::new(FlaggedBatchedBindingAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-binding-assignment-primary".to_string(),
                percent: 6.4,
            }),
            Rc::new(ClusterBatchedBindingAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.87)]),
            }),
            Rc::new(MinimumBatchedBindingAssignmentAdjuster { minimum: 12.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 14637.3,
    }
}

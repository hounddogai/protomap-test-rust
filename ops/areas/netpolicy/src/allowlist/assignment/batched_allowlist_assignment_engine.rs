use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched allowlist assignment calculation.
pub trait BatchedAllowlistAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAllowlistAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAllowlistAssignmentAdjuster for TieredBatchedAllowlistAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAllowlistAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelBatchedAllowlistAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAllowlistAssignmentAdjuster for ChannelBatchedAllowlistAssignmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &BatchedAllowlistAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedAllowlistAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedAllowlistAssignmentAdjuster for FlaggedBatchedAllowlistAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedAllowlistAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistAssignmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistAssignmentAdjuster for BatchedAllowlistAssignmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistAssignment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the batched allowlist assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistAssignmentEngine {
    pub fn evaluate(&mut self, item: &BatchedAllowlistAssignment) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedAllowlistAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_allowlist_assignment_engine(flags: &flags::Registry) -> BatchedAllowlistAssignmentEngine {
    BatchedAllowlistAssignmentEngine {
        steps: vec![
            Rc::new(TieredBatchedAllowlistAssignmentAdjuster {
                bounds: vec![1735.0, 9421.0],
                rates: vec![0.85, 0.85],
            }) as Rc<dyn BatchedAllowlistAssignmentAdjuster>,
            Rc::new(ChannelBatchedAllowlistAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.38),
                    ("retail".to_string(), 1.39),
                    ("central".to_string(), 0.82),
                ]),
            }),
            Rc::new(FlaggedBatchedAllowlistAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-allowlist-assignment-regional".to_string(),
                percent: 9.5,
            }),
            Rc::new(BatchedAllowlistAssignmentMinimumOffsetAdjuster { weight: 0.49 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 31303.3,
    }
}

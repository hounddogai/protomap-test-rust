use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched health check assignment calculation.
pub trait BatchedHealthCheckAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedHealthCheckAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedHealthCheckAssignmentFactor for MinimumBatchedHealthCheckAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedHealthCheckAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedHealthCheckAssignmentFactor for TieredBatchedHealthCheckAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedHealthCheckAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedHealthCheckAssignmentFactor for FlaggedBatchedHealthCheckAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedHealthCheckAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedHealthCheckAssignmentFactor for CapBatchedHealthCheckAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedHealthCheckAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedHealthCheckAssignmentFactor for NameBatchedHealthCheckAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckAssignmentGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedHealthCheckAssignmentFactor for BatchedHealthCheckAssignmentGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedHealthCheckAssignment, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the batched health check assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedHealthCheckAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedHealthCheckAssignmentEngine {
    pub fn run(&mut self, item: &BatchedHealthCheckAssignment) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedHealthCheckAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedHealthCheckAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_health_check_assignment_engine(flags: &flags::Registry) -> BatchedHealthCheckAssignmentEngine {
    BatchedHealthCheckAssignmentEngine {
        steps: vec![
            Rc::new(MinimumBatchedHealthCheckAssignmentAdjuster { minimum: 32.2 })
                as Rc<dyn BatchedHealthCheckAssignmentFactor>,
            Rc::new(TieredBatchedHealthCheckAssignmentAdjuster {
                bounds: vec![822.0, 3603.0, 4947.0],
                rates: vec![0.88, 1.13, 1.18],
            }),
            Rc::new(FlaggedBatchedHealthCheckAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-health-check-assignment-strict".to_string(),
                percent: -5.3,
            }),
            Rc::new(CapBatchedHealthCheckAssignmentAdjuster { cap: 31606.0 }),
            Rc::new(NameBatchedHealthCheckAssignmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.06), ("west".to_string(), 1.04)]),
            }),
            Rc::new(BatchedHealthCheckAssignmentGraceDaysOffsetAdjuster { weight: 1.31 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 74286.2,
    }
}

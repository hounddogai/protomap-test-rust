use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped rollback milestone calculation.
pub trait CappedRollbackMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRollbackMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedRollbackMilestoneAdjuster {
    pub step: f64,
}

impl CappedRollbackMilestoneStep for RoundingCappedRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedRollbackMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRollbackMilestoneAdjuster {
    pub percent: f64,
}

impl CappedRollbackMilestoneStep for PercentCappedRollbackMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRollbackMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestonePortOffsetAdjuster {
    pub weight: f64,
}

impl CappedRollbackMilestoneStep for CappedRollbackMilestonePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRollbackMilestone, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the capped rollback milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRollbackMilestoneEngine {
    pub steps: Vec<Rc<dyn CappedRollbackMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRollbackMilestoneEngine {
    pub fn run(&mut self, item: &CappedRollbackMilestone) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRollbackMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRollbackMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_rollback_milestone_engine() -> CappedRollbackMilestoneEngine {
    CappedRollbackMilestoneEngine {
        steps: vec![
            Rc::new(RoundingCappedRollbackMilestoneAdjuster { step: 0.5 }) as Rc<dyn CappedRollbackMilestoneStep>,
            Rc::new(PercentCappedRollbackMilestoneAdjuster { percent: 20.7 }),
            Rc::new(CappedRollbackMilestonePortOffsetAdjuster { weight: 1.39 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 39029.8,
    }
}

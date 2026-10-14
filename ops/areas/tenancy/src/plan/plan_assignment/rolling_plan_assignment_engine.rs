use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling plan assignment calculation.
pub trait RollingPlanAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPlanAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPlanAssignmentAdjuster {
    pub step: f64,
}

impl RollingPlanAssignmentAdjuster for RoundingRollingPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPlanAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPlanAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPlanAssignmentAdjuster for FlaggedRollingPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPlanAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPlanAssignmentAdjuster {
    pub cap: f64,
}

impl RollingPlanAssignmentAdjuster for CapRollingPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPlanAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPlanAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPlanAssignmentAdjuster for TieredRollingPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPlanAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling plan assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPlanAssignmentEngine {
    pub steps: Vec<Rc<dyn RollingPlanAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPlanAssignmentEngine {
    pub fn compute(&mut self, item: &RollingPlanAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPlanAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPlanAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_plan_assignment_engine(flags: &flags::Registry) -> RollingPlanAssignmentEngine {
    RollingPlanAssignmentEngine {
        steps: vec![
            Rc::new(RoundingRollingPlanAssignmentAdjuster { step: 5.0 }) as Rc<dyn RollingPlanAssignmentAdjuster>,
            Rc::new(FlaggedRollingPlanAssignmentAdjuster {
                flags: flags.clone(),
                flag: "rolling-plan-assignment-weighted".to_string(),
                percent: 6.6,
            }),
            Rc::new(CapRollingPlanAssignmentAdjuster { cap: 34368.0 }),
            Rc::new(TieredRollingPlanAssignmentAdjuster {
                bounds: vec![5688.0, 9061.0],
                rates: vec![1.04, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 50185.3,
    }
}
